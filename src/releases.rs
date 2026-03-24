use crate::directory::dsh_directory_pathbuf;
use crate::{cli_error, err, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, DshCliResult};
use dsh_api::version::Version;
use openidconnect::reqwest::blocking::ClientBuilder;
use openidconnect::reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use openidconnect::reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering::Greater;
use std::io::Read;
use std::str::FromStr;
use std::time::SystemTime;
use tokio::task::spawn_blocking;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Release {
  pub(crate) html_url: String,
  pub(crate) id: u64,
  pub(crate) tag_name: String,
  pub(crate) target_commitish: String,
  pub(crate) name: String,
  pub(crate) draft: bool,
  pub(crate) immutable: bool,
  pub(crate) prerelease: bool,
  pub(crate) created_at: String,
  pub(crate) updated_at: String,
  pub(crate) published_at: String,
  pub(crate) assets: Vec<ReleaseAsset>,
  pub(crate) version: Option<Version>,
  pub(crate) body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ReleaseAsset {
  pub(crate) url: String,
  pub(crate) name: String,
  pub(crate) content_type: String,
  pub(crate) state: String,
  pub(crate) size: u64,
  pub(crate) created_at: String,
  pub(crate) updated_at: String,
  pub(crate) browser_download_url: String,
}

/// Returns newer release
///
/// Return the latest release when it is newer than the current version.
///
/// ## Parameters
/// * `current_version` - Version number of the current application.
///
/// ## Returns
/// * `Some(Release)` - When a newer release than the current version is available.
/// * `None` - When the current version is the latest (or newer than the latest version).
pub(crate) async fn newer_release(current_version: &Version) -> DshCliResult<Option<Release>> {
  let (latest_release_version, latest_release, _) = latest_release().await?;
  if current_version < &latest_release_version {
    Ok(Some(latest_release))
  } else {
    Ok(None)
  }
}

/// Returns newer release notification
///
/// Return the latest release when it is newer than the current version, but only when the latest
/// release information is not from the cache but from GitHub. This can be used to notify the
/// user that a newer release exists, but only once per day.
///
/// ## Parameters
/// * `current_version` - Version number of the current application.
///
/// ## Returns
/// * `Some(Release)` - When the user should be notified that there is a newer version.
/// * `None` - No newer release is available or the user should not be notified.
pub(crate) async fn newer_release_notification(current_version: &Version) -> Option<Release> {
  match latest_release().await {
    Ok((latest_release_version, latest_release, cached)) => {
      if !cached && current_version < &latest_release_version {
        Some(latest_release)
      } else {
        None
      }
    }
    Err(_) => None,
  }
}

const NUMBER_OF_SECONDS_IN_A_DAY: u64 = 24 * 60 * 60;

/// Get the latest release of the tool
///
/// ## Returns
/// Tuple `(version, release, cached)` consisting of:
/// * `version` - Version of the latest release.
/// * `release` - Latest release.
/// * `cached` - Whether the latest release information was cached or freshly retrieved fromGitHub.
async fn latest_release() -> DshCliResult<(Version, Release, bool)> {
  let (latest_release, cached) = if let Some((latest_release_from_file, modified)) = latest_release_from_file()? {
    if modified.elapsed()?.cmp(&core::time::Duration::from_secs(NUMBER_OF_SECONDS_IN_A_DAY)) == Greater {
      // Latest release file exists but is more than a day old
      (latest_release_from_github().await?, false)
    } else {
      // Latest release file exists and is less than a day old
      (Some(latest_release_from_file), true)
    }
  } else {
    // Latest release file does not exist
    (latest_release_from_github().await?, false)
  };
  match latest_release {
    Some(latest_release) => Ok((
      latest_release.version.clone().ok_or(cli_error!("latest release version could not be parsed"))?,
      latest_release,
      cached,
    )),
    None => err!("latest release is not available"),
  }
}

const LATEST_RELEASE_FILENAME: &str = "latest-release.toml";

/// Store latest release
fn write_latest_release_to_file(latest_release: &Release) -> DshCliResult<()> {
  if let Some(pathbuf) = dsh_directory_pathbuf(LATEST_RELEASE_FILENAME)? {
    serialize_and_write_to_toml_file(&pathbuf, latest_release)?;
  }
  Ok(())
}

/// Get stored latest release
///
/// ## Returns
/// * `Some((Release, SystemTime))` - When stored latest release information was found, it is
///   returned together with the timestamp when it was created.
/// * `None` - No stored latest release information was found.
fn latest_release_from_file() -> DshCliResult<Option<(Release, SystemTime)>> {
  if let Some(pathbuf) = dsh_directory_pathbuf(LATEST_RELEASE_FILENAME)? {
    match read_and_deserialize_from_toml_file::<Release>(&pathbuf)? {
      Some(release) => Ok(Some((release, pathbuf.metadata()?.modified()?))),
      None => Ok(None),
    }
  } else {
    Ok(None)
  }
}

/// Get latest release from GitHub
///
/// When the latest release is found, as a side effect it will be stored in the
/// "latest-release.toml" file.
///
/// ## Returns
/// * `Some((Release, SystemTime))` - When stored latest release information was found, it is
///   returned together with the timestamp when it was created.
/// * `None` - No stored latest release information was found.
async fn latest_release_from_github() -> DshCliResult<Option<Release>> {
  let mut releases = releases_from_github().await?;
  releases.reverse();
  match releases
    .into_iter()
    .find(|release| release.version.is_some() && !release.prerelease && !release.draft)
  {
    Some(latest_release_from_github) => {
      write_latest_release_to_file(&latest_release_from_github)?;
      Ok(Some(latest_release_from_github))
    }
    None => Ok(None),
  }
}

/// Get releases information from GitHub
///
/// ## Returns
/// * `Vec<Release>` - Vector describing all releases of the application found at GitHub.
async fn releases_from_github() -> DshCliResult<Vec<Release>> {
  spawn_blocking(move || {
    const DSH_CLI_RELEASES_ENDPOINT_USER_AGENT: &str = "kpn-dsh/dsh-cli";
    const DSH_CLI_RELEASES_ENDPOINT: &str = "https://api.github.com/repos/kpn-dsh/dsh-cli/releases";
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DSH_CLI_RELEASES_ENDPOINT_USER_AGENT));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github+json"));
    let http_client = ClientBuilder::new().redirect(Policy::none()).default_headers(headers).build()?;
    let mut response = http_client.get(DSH_CLI_RELEASES_ENDPOINT).send()?;
    let mut body = String::new();
    response.read_to_string(&mut body)?;
    let mut releases: Vec<Release> = serde_json::from_str(&body)?;
    for release in releases.iter_mut() {
      if let Some(version) = parse_version(&release.name) {
        release.version = Some(version);
      } else if let Some(version) = parse_version(&release.tag_name) {
        release.version = Some(version);
      }
    }
    releases.sort_by(|release_a, release_b| release_a.version.cmp(&release_b.version));
    Ok(releases)
  })
  .await?
}

fn parse_version(version_string: &str) -> Option<Version> {
  if let Some(stripped_version_string) = version_string.strip_prefix("v") {
    Version::from_str(stripped_version_string).ok()
  } else {
    Version::from_str(version_string).ok()
  }
}
