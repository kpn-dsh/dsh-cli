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
//!  │    │    │    └── certificates/
//!  │    │    │         ├── broker-ca.pem
//!  │    │    │         ├── broker-client.key
//!  │    │    │         └── broker-client.pem
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

use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_HOME};
use crate::settings::Settings;
use crate::{err, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, DshCliResult};
use dsh_api::platform::DshPlatform;
use homedir::my_home;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const _CERTIFICATES_SUBDIRECTORY: &str = "certificates";
const DEFAULT_SETTINGS_FILENAME: &str = "settings.toml";
const DEFAULT_USER_DSH_CLI_DIRECTORY: &str = ".dsh_cli";
const REFRESH_TOKEN_FILENAME: &str = "refresh-token.encrypted";
const TARGETS_SUBDIRECTORY: &str = "targets";

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
        _ = &fs::create_dir_all(&dsh_directory)?;
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
      fs::write(refresh_token_pathbuf, refresh_token)?;
      Ok(())
    }
    None => err!("dsh directory disabled, refresh token cannot be stored"),
  }
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
fn create_parent_directories(path: &Path) -> DshCliResult<()> {
  match path.parent() {
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
    None => err!("'{}' has no parent", path.display()),
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
