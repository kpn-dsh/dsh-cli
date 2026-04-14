//! # Dsh directory structure
//!
//! This module defines the directory structure used by the dsh tool to store settings, tokens
//! and certificates. The default root of this directory structure is `$HOME/.dsh_cli`,
//! but this can be overridden via the environment variable `DSH_CLI_HOME`.
//!
//! ```
//! $HOME/.dsh_cli/
//!  ├── targets/
//!  │    ├── platform1/
//!  │    │    ├── tenant1/
//!  │    │    │    └── bundles/
//!  │    │    │         └── proxy1/
//!  │    │    │              ├── ca.key
//!  │    │    │              ├── ca.pem
//!  │    │    │              ├── client.key
//!  │    │    │              ├── client.pem
//!  │    │    │              ├── server.key
//!  │    │    │              └── server.key
//!  │    │    ├── tenant2/
//!  │    │    │     ...
//!  │    │    └── refresh-token.encrypted
//!  │    └── platform2/
//!  │        ...
//!  └── settings.toml
//! ```
//!
//! The location of the dsh directory is determined by the first match of the potential directory
//! locations listed below.
//! 1. If the environment variable `DSH_CLI_HOME` is set to the empty string, no dsh directory
//!    will be available. In this case the dsh tool will not be able to store any settings, but
//!    that might be the desired behavior in some use cases, e.g. in a CI/CD environment.
//! 1. If the environment variable `DSH_CLI_HOME` is set to a non-empty value, that value is used
//!    as the location.
//! 1. If the environment variable `HOME` is set its value will be used.
//! 1. Else the application should terminate reporting the error.
//!
//! Note that the environment variables `DSH_CLI_HOME` and `HOME` must be regular environment
//! variables and cannot be specified via the command line `--environment-variable` argument.

use crate::certificates::ProxyCertificateBundle;
use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_HOME};
use crate::settings::Settings;
use crate::{err, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, DshCliResult};
use dsh_api::platform::DshPlatform;
use homedir::my_home;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use rcgen::KeyPair;
use std::fs;
use std::fs::{File, Permissions};
use std::io::{ErrorKind, Write};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const PROXY_CERTIFICATE_BUNDLES_SUBDIRECTORY: &str = "bundles";
const DEFAULT_SETTINGS_FILENAME: &str = "settings.toml";
const DEFAULT_USER_DSH_CLI_DIRECTORY: &str = ".dsh_cli";
const REFRESH_TOKEN_FILENAME: &str = "refresh-token.encrypted";
const TARGETS_SUBDIRECTORY: &str = "targets";

const MODE_U_RW: u32 = 0o600;

/// # Initialise the dsh directory
///
/// This function initialises the dsh directory. If it does not already exist the directory
/// will be created, together with required subdirectories.
///
/// This function must be called before any function that depends on the dsh directory is called.
/// The modules `refresh_token_store`, `settings` and `targets` depend on the dsh directory.
///
/// See this modules doc comments for an explanation how the dsh directory location is
/// determined.
///
/// ## Returns
/// * `Ok<()>` - Initialization was successful.
/// * `Err<DshCliError>` - Initialization failed and the application must report the error and
///   terminate.
#[allow(clippy::single_element_loop)]
pub(crate) fn init_dsh_directory() -> DshCliResult<()> {
  match root_dsh_directory_pathbuf() {
    Ok(Some(dsh_directory)) => {
      if dsh_directory.exists() {
        if dsh_directory.is_dir() {
          debug!("dsh directory '{}' already exists", dsh_directory.display());
          Ok(())
        } else {
          err!(
            "dsh directory initialization failed because '{}' already exists but is not a directory",
            dsh_directory.display()
          )
        }
      } else {
        fs::create_dir_all(&dsh_directory)?;
        info!("dsh directory {} created", dsh_directory.display());
        for dsh_subdirectory_name in [TARGETS_SUBDIRECTORY] {
          let dsh_subdirectory = dsh_directory.join(dsh_subdirectory_name);
          fs::create_dir_all(&dsh_subdirectory)?;
          info!("dsh subdirectory '{}' created", dsh_subdirectory.display());
        }
        Ok(())
      }
    }
    Ok(None) => Ok(()),
    Err(error) => err!("dsh directory initialization failed ({})", error),
  }
}

/// # Returns whether the dsh directory is supported
///
/// This function returns whether the dsh directory is available or not. If this directory does
/// not exist the dsh tool will not be able to store any settings, but that might be the desired
/// behavior in some use cases, e.g. in a CI/CD environment.
///
/// ## Returns
/// * `true` - A valid dsh directory is available.
/// * `false` - No valid dsh directory available, either by configuration or by an earlier error.
pub(crate) fn supports_dsh_directory() -> bool {
  match &*DSH_DIRECTORY {
    Ok(Some(_)) => true,
    Ok(None) => false,
    Err(_) => false,
  }
}

/// # Delete stored refresh token
///
/// ## Parameters
/// * `platform` - Platform for which the token must be deleted.
///
/// ## Returns
/// * `Ok<true>` - Refresh token was deleted.
/// * `Ok<false>` - Refresh token does not exist.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn delete_refresh_token(platform: &DshPlatform) -> DshCliResult<bool> {
  match refresh_token_pathbuf(platform)? {
    Some(refresh_token_pathbuf) => match fs::remove_file(&refresh_token_pathbuf) {
      Ok(()) => {
        debug!("refresh token '{}' deleted", refresh_token_pathbuf.display());
        Ok(true)
      }
      Err(error) => match error.kind() {
        ErrorKind::NotFound => Ok(false),
        _ => err!("error deleting refresh token '{}' ({})", refresh_token_pathbuf.display(), error),
      },
    },
    None => err!("dsh directory disabled, refresh token cannot be deleted"),
  }
}

/// Read stored refresh token
///
/// ## Parameters
/// * `platform` - Platform for which the token is requested.
///
/// ## Returns
/// * `Ok<Some<String>>` - Refresh token.
/// * `Ok<None>` - Refresh token does not exist.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn read_refresh_token(platform: &DshPlatform) -> DshCliResult<Option<String>> {
  match refresh_token_pathbuf(platform)? {
    Some(refresh_token_pathbuf) => match fs::read_to_string(&refresh_token_pathbuf) {
      Ok(refresh_token_string) => {
        debug!("refresh token for platform '{}' read from file {}", platform, refresh_token_pathbuf.display());
        Ok(Some(refresh_token_string))
      }
      Err(error) => match error.kind() {
        ErrorKind::NotFound => {
          debug!("refresh token for platform '{}' not found", platform);
          Ok(None)
        }
        _ => err!(
          "error reading refresh token '{}' for platform '{}' ({})",
          refresh_token_pathbuf.display(),
          platform,
          error
        ),
      },
    },
    None => err!("dsh directory disabled, refresh token cannot be read"),
  }
}

/// Store refresh token
///
/// ## Parameters
/// * `platform` - Platform for which the token must be stored.
/// * `refresh_token` - Refresh token that must be stored.
///
/// ## Returns
/// * `Ok<()>` - If storing was successful.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn write_refresh_token(platform: &DshPlatform, refresh_token: &str) -> DshCliResult<()> {
  match refresh_token_pathbuf(platform)? {
    Some(refresh_token_pathbuf) => {
      create_parent_directories(&refresh_token_pathbuf)?;
      debug!("write refresh token for platform '{}' to '{}'", platform, refresh_token_pathbuf.display());
      write_with_mode(refresh_token_pathbuf, refresh_token, Some(MODE_U_RW))?;
      Ok(())
    }
    None => err!("dsh directory disabled, refresh token cannot be stored"),
  }
}

const CA_KEY_FILENAME: &str = "CA_KEY";
const CA_CERTIFICATE_FILENAME: &str = "CA_PEM";
const CLIENT_KEY_FILENAME: &str = "CLIENT_KEY";
const CLIENT_CERTIFICATE_FILENAME: &str = "CLIENT_PEM";
const SERVER_KEY_FILENAME: &str = "SERVER_KEY";
const SERVER_CERTIFICATE_FILENAME: &str = "SERVER_PEM";

/// Read stored certificate bundle
///
/// ## Parameters
/// * `platform` - Platform for which the certificate bundle is requested.
/// * `tenant` - Tenant for which the certificate bundle is requested.
/// * `proxy_prefix` - Proxy prefix for the requested certificate bundle.
///
/// ## Returns
/// * `Ok<Some<CertBundle>>` - Certificate bundle.
/// * `Ok<None>` - Certificate bundle does not exist.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn read_certificate_bundle(platform: &DshPlatform, tenant: &str, proxy_prefix: &str) -> DshCliResult<Option<ProxyCertificateBundle>> {
  match certificate_bundle_pathbuf(platform, tenant, proxy_prefix)? {
    Some(certificate_bundle_directory_pathbuf) => {
      let mut ca_key_file_path = certificate_bundle_directory_pathbuf.clone();
      ca_key_file_path.push(CA_KEY_FILENAME);
      let _ca_key = KeyPair::from_pem(&fs::read_to_string(ca_key_file_path)?)?;

      let mut ca_certificate_file_path = certificate_bundle_directory_pathbuf.clone();
      ca_certificate_file_path.push(CA_CERTIFICATE_FILENAME);
      // let ca_certificate = Certificate::fr(&fs::read_to_string(ca_certificate_file_path)?)?;

      Ok(None)
    } // TODO
    None => err!("dsh directory disabled, certificate bundle cannot be read"),
  }
}

/// Store certificate bundle
///
/// ## Parameters
/// * `platform` - Platform for which the certificate bundle must be stored.
/// * `tenant` - Tenant for which the certificate bundle must be stored.
/// * `proxy_prefix` - Proxy prefix for the certificate bundle.
/// * `certificate_bundle` - Certificate bundle that must be stored.
///
/// ## Returns
/// * `Ok<String>` - If storing was successful, the directory name where the bundle was stored
///   will be returned.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn store_certificate_bundle(platform: &DshPlatform, tenant: &str, proxy_prefix: &str, certificate_bundle: &ProxyCertificateBundle) -> DshCliResult<String> {
  match certificate_bundle_pathbuf(platform, tenant, proxy_prefix)? {
    Some(certificate_bundle_directory_pathbuf) => {
      let pems = vec![
        (CA_CERTIFICATE_FILENAME, certificate_bundle.ca_certificate.certificate.pem(), Some(MODE_U_RW)),
        (CA_KEY_FILENAME, certificate_bundle.ca_certificate.key_pair.serialize_pem(), Some(MODE_U_RW)),
        (
          CLIENT_CERTIFICATE_FILENAME,
          certificate_bundle.client_certificate.certificate.pem(),
          Some(MODE_U_RW),
        ),
        (CLIENT_KEY_FILENAME, certificate_bundle.client_certificate.key_pair.serialize_pem(), Some(MODE_U_RW)),
        (
          SERVER_CERTIFICATE_FILENAME,
          certificate_bundle.server_certificate.certificate.pem(),
          Some(MODE_U_RW),
        ),
        (SERVER_KEY_FILENAME, certificate_bundle.server_certificate.key_pair.serialize_pem(), Some(MODE_U_RW)),
      ];
      for (filename, pem, mode) in pems {
        let mut file_path = certificate_bundle_directory_pathbuf.clone();
        file_path.push(filename);
        debug!("write certificate bundle file '{}'", file_path.display());
        write_with_mode(file_path, &pem, mode)?;
      }
      Ok(certificate_bundle_directory_pathbuf.display().to_string())
    }
    None => err!("dsh directory disabled, certificate bundle cannot be stored"),
  }
}

fn write_with_mode<T>(file_path: T, data: &str, mode: Option<u32>) -> DshCliResult<()>
where
  T: AsRef<Path>,
{
  create_parent_directories(file_path.as_ref())?;
  debug!("write certificate bundle file '{}'", file_path.as_ref().display());
  let mut file = File::create(file_path)?;
  file.write_all(data.as_bytes())?;
  if let Some(mode) = mode {
    #[cfg(target_family = "unix")]
    file.set_permissions(Permissions::from_mode(mode))?;
    #[cfg(not(target_family = "unix"))]
    debug!("permissions on file '{}' can only be set on unix", file_path.display());
  }
  Ok(())
}

/// # Returns the settings file
///
/// Create the [PathBuf] for the settings file. The file name will be
/// "$HOME/.dsh_cli/settings.toml".
///
/// ## Returns
/// * `Ok<Some<PathBuf>>` - Pathbuf of the settings file.
/// * `Ok<None>` - Dsh tool does not support storing state and settings.
/// * `Err<DshCliError>` - Settings file could not be created.
pub(crate) fn settings_file_pathbuf() -> DshCliResult<Option<PathBuf>> {
  dsh_directory_pathbuf(DEFAULT_SETTINGS_FILENAME)
}

pub(crate) fn get_settings() -> DshCliResult<(Settings, String)> {
  match settings_file_pathbuf()? {
    Some(settings_file) => match read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(settings_file.clone()))? {
      Some(settings_from_default_file) => Ok((
        Settings { file_name: Some(settings_file.display().to_string()), ..settings_from_default_file },
        format!("read settings from '{}'", settings_file.display()),
      )),
      None => Ok((Settings::default(), "no settings file found, using default settings".to_string())),
    },
    None => Ok((Settings::default(), "dsh directory disabled, using default settings".to_string())),
  }
}

pub(crate) fn write_settings(settings: Settings) -> DshCliResult<()> {
  match settings_file_pathbuf()? {
    Some(settings_file) => {
      debug!("write settings to default file '{}'", settings_file.display());
      serialize_and_write_to_toml_file(settings_file, &settings)
    }
    None => err!("dsh directory disabled, settings cannot be stored"),
  }
}

/// # Returns the root dsh directory pathbuf
///
/// This function returns a `Pathbuf` pointing at the root dsh directory, if it is available.
///
/// ## Returns
/// * `Ok<Some<PathBuf>>` - Pathbuf of the root dsh directory.
/// * `Ok<None>` - Dsh tool does not support storing state and settings.
/// * `Err<DshCliError>` - When the dsh directory could not be determined.
fn root_dsh_directory_pathbuf() -> DshCliResult<Option<PathBuf>> {
  match &*DSH_DIRECTORY {
    Ok(Some(path_buf)) => Ok(Some(path_buf.clone())),
    Ok(None) => Ok(None),
    Err(error) => Err(error.clone()),
  }
}

/// # Returns dsh subdirectory pathbuf
///
/// This function returns a `Pathbuf` from the root dsh directory joined with the provided
/// `subdirectory`.
///
/// ## Parameters
/// * `subdirectory` - Requested subdirectory of the root dsh directory.
///
/// ## Returns
/// * `Ok<Some<PathBuf>>` - Pathbuf of the dsh directory.
/// * `Ok<None>` - Dsh tool does not support storing state and settings.
/// * `Err<DshCliError>` -  Dsh directory could not be determined.
pub(crate) fn dsh_directory_pathbuf(subdirectory: &str) -> DshCliResult<Option<PathBuf>> {
  root_dsh_directory_pathbuf().map(|pathbuf| pathbuf.map(|root_directory| root_directory.join(subdirectory)))
}

/// Returns `PathBuf` for refresh token file
///
/// Return the [PathBuf] for a refresh token file for the provided `platform`. The filename will
/// be "$HOME/.dsh_cli/targets/\[platform.name\]/refresh-token.encrypted".
///
/// ## Parameters
/// * `platform` - Platform for which the [PathBuf] will be created.
///
/// ## Returns
/// * `Ok<Some<PathBuf>>` - Pathbuf of the refresh token file for `platform`.
/// * `Ok<None>` - Dsh tool does not support storing state and settings.
/// * `Err<DshCliError>` -  Ssh directory could not be determined.
fn refresh_token_pathbuf(platform: &DshPlatform) -> DshCliResult<Option<PathBuf>> {
  dsh_directory_pathbuf(&format!("{}/{}/{}", TARGETS_SUBDIRECTORY, platform.name(), REFRESH_TOKEN_FILENAME))
}

/// Returns `PathBuf` for certificate bundle
///
/// Return the [PathBuf] for a certificate bundle directory for the provided `platform`, `tenant`
/// and `proxy_prefix`. The directory name will be
///
/// $HOME/.dsh_cli/targets/\[platform.name\]/\[tenant\]/bundles/\[proxy_prefix\]
///
/// ## Parameters
/// * `platform` - Platform for which the [PathBuf] will be created.
/// * `tenant` - Tenant for which the [PathBuf] will be created.
/// * `proxy_prefix` - Proxy prefix for which the [PathBuf] will be created.
///
/// ## Returns
/// * `Ok<Some<PathBuf>>` - Pathbuf of the certificate bundle directory.
/// * `Ok<None>` - Dsh tool does not support storing state and settings.
/// * `Err<DshCliError>` -  Ssh directory could not be determined.
fn certificate_bundle_pathbuf(platform: &DshPlatform, tenant: &str, proxy_prefix: &str) -> DshCliResult<Option<PathBuf>> {
  dsh_directory_pathbuf(&format!(
    "{}/{}/{}/{}/{}",
    TARGETS_SUBDIRECTORY,
    platform.name(),
    tenant,
    PROXY_CERTIFICATE_BUNDLES_SUBDIRECTORY,
    proxy_prefix
  ))
}

/// Create parent directory
///
/// Create the parent directory or directories for the provided `path`. If the parent directory
/// already exists, nothing will happen.
///
/// ## Parameters
/// * `path` - Path for which the parent directory or directories will be created.
///
/// ## Returns
/// * `Ok<()>` - Parent directory already exists or was successfully created.
/// * `Err<DshCliError>` - Pasrent directory or directories could not be created.
fn create_parent_directories<T>(path: T) -> DshCliResult<()>
where
  T: AsRef<Path>,
{
  match path.as_ref().parent() {
    Some(parent) => {
      if parent.exists() {
        if parent.is_dir() {
          Ok(())
        } else {
          err!("parent '{}' already exists but is not a directory", parent.display())
        }
      } else {
        fs::create_dir_all(parent)?;
        debug!("parent directory '{}' created", parent.display());
        Ok(())
      }
    }
    None => err!("'{}' has no parent", path.as_ref().display()),
  }
}

lazy_static! {
  static ref DSH_DIRECTORY: DshCliResult<Option<PathBuf>> = {
    match environment_variable(ENV_VAR_DSH_CLI_HOME, None) {
      Ok(Some(dsh_directory_from_env_var)) => {
        if dsh_directory_from_env_var.is_empty() {
          warn!("environment variable '{}' is set but empty", ENV_VAR_DSH_CLI_HOME);
          Ok(None)
        } else {
          debug!(
            "dsh directory '{}' set from environment variable '{}'",
            dsh_directory_from_env_var, ENV_VAR_DSH_CLI_HOME
          );
          Ok(Some(PathBuf::new().join(dsh_directory_from_env_var)))
        }
      }
      Ok(None) => match my_home() {
        Ok(Some(user_home_directory)) => {
          let dsh_directory = user_home_directory.join(DEFAULT_USER_DSH_CLI_DIRECTORY);
          debug!("dsh directory '{}' in user home directory", dsh_directory.display());
          Ok(Some(dsh_directory))
        }
        Ok(None) => err!(
          "could not determine dsh cli directory, check environment variable '{}' or 'HOME'",
          ENV_VAR_DSH_CLI_HOME
        ),
        Err(error) => err!(
          "error determining user home directory, check environment variable '{}' or 'HOME' ({})",
          ENV_VAR_DSH_CLI_HOME,
          error
        ),
      },
      Err(_) => unreachable!(),
    }
  };
}
