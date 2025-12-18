//! # Dsh directory structure
//!
//! This module defines the directory structure used by the dsh tool to store settings, tokens
//! and certificates. The default root of this directory structure is `$HOME/.dsh_cli`,
//! but this can be overridden via the environment variable `DSH_CLI_HOME`.
//!
//! ```
//! $HOME/.dsh_cli/
//!  ├─ targets/
//!  │   ├─ platform1/
//!  │   │   ├─ tenant1/
//!  │   │   │   └─ certificates/
//!  │   │   │       ├─ broker-ca.pem
//!  │   │   │       ├─ broker-client.key
//!  │   │   │       └─ broker-client.pem
//!  │   │   ├─ tenant2/
//!  │   │   │    ...
//!  │   │   └─ refresh-token.encrypted
//!  │   └─ platform2/
//!  │        ...
//!  └─ settings.toml
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
use crate::targets::{upsert_password_to_keyring, Target};
use crate::{error, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, DshCliResult};
use dsh_api::platform::DshPlatform;
use homedir::my_home;
use lazy_static::lazy_static;
use log::debug;
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
/// * `Ok<()>` - Initialisation was successful.
/// * `Err<DshCliError>` - Initialisation failed and the application must report the error and
///   terminate.
#[allow(clippy::single_element_loop)]
pub(crate) fn init_dsh_directory() -> DshCliResult<()> {
  match root_dsh_directory_pathbuf() {
    Ok(Some(dsh_directory)) => {
      if dsh_directory.exists() {
        if dsh_directory.is_dir() {
          log::debug!("dsh directory already {} exists", dsh_directory.display());
          Ok(())
        } else {
          Err(error!(
            "initialization failed because {} already exists but is not a directory",
            dsh_directory.display()
          ))
        }
      } else {
        _ = &fs::create_dir_all(&dsh_directory)?;
        log::info!("dsh directory {} created", dsh_directory.display());
        for dsh_subdirectory_name in [TARGETS_SUBDIRECTORY] {
          let dsh_subdirectory = dsh_directory.join(dsh_subdirectory_name);
          fs::create_dir_all(&dsh_subdirectory)?;
          log::info!("dsh subdirectory {} created", dsh_subdirectory.display());
        }
        Ok(())
      }
    }
    Ok(None) => Ok(()),
    Err(error) => Err(error!("initialization failed {})", error)),
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
        log::debug!("refresh token {} deleted", refresh_token_pathbuf.display());
        Ok(true)
      }
      Err(error) => match error.kind() {
        ErrorKind::NotFound => Ok(false),
        _ => Err(error!("error deleting refresh token {} ({})", refresh_token_pathbuf.display(), error)),
      },
    },
    None => Err(error!("dsh directory disabled, refresh token cannot be deleted")),
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
  // let refresh_token_pathbuf = refresh_token_pathbuf(platform)?;
  match refresh_token_pathbuf(platform)? {
    Some(refresh_token_pathbuf) => match fs::read_to_string(&refresh_token_pathbuf) {
      Ok(refresh_token_string) => {
        log::debug!("refresh token for platform '{}' read from file {}", platform, refresh_token_pathbuf.display());
        Ok(Some(refresh_token_string))
      }
      Err(error) => match error.kind() {
        ErrorKind::NotFound => {
          log::debug!("refresh token for platform '{}' not found", platform);
          Ok(None)
        }
        _ => Err(error!("error reading refresh token {} ({})", refresh_token_pathbuf.display(), error)),
      },
    },
    None => Err(error!("dsh directory disabled, refresh token cannot be read")),
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
pub(crate) fn write_refresh_token(platform: &DshPlatform, refresh_token: String) -> DshCliResult<()> {
  match refresh_token_pathbuf(platform)? {
    Some(refresh_token_pathbuf) => {
      create_parent_directories(&refresh_token_pathbuf)?;
      fs::write(refresh_token_pathbuf, refresh_token)?;
      Ok(())
    }
    None => Err(error!("dsh directory disabled, refresh token cannot be stored")),
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
    None => Err(error!("dsh directory disabled, settings cannot be stored")),
  }
}

/// # Delete target directory
///
/// This function will delete a target directory (if it exists).
///
/// ## Parameters
/// * `platform` - Target platform.
/// * `tenant` - Target tenant name.
///
/// ## Returns
/// * `Ok(())` - Indicates that deleting the target directory was successful.
pub(crate) fn delete_target_directory(platform: &DshPlatform, tenant: &str) -> DshCliResult<bool> {
  match target_directory_pathbuf(platform, tenant)? {
    Some(target_directory) => {
      if target_directory.exists() {
        if target_directory.is_dir() {
          fs::remove_dir_all(target_directory)?;
          Ok(true)
        } else {
          Err(error!("{} is not a directory", target_directory.display()))
        }
      } else {
        Ok(false)
      }
    }
    None => Err(error!("dsh directory disabled, target directory cannot be deleted")),
  }
}

/// # Read target directory
///
/// This function will read the target parameters from the target directory (if it exists).
/// The `password` field of the returned `Target` will always be `None`.
///
/// ## Parameters
/// * `platform` - target platform
/// * `tenant` - target tenant name
///
/// ## Returns
/// * `Ok(Some(target))` - If the target directory exists a `Target` will be returned.
/// * `Ok(None)` - If the target directory does not exist.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn read_target(platform: &DshPlatform, tenant: &str) -> DshCliResult<Option<Target>> {
  match target_directory_pathbuf(platform, tenant)? {
    Some(target_directory) => {
      if target_directory.exists() {
        if target_directory.is_dir() {
          Ok(Some(Target::new(platform.clone(), tenant.to_string(), None, vec![])))
        } else {
          Err(error!("{} is not a directory", target_directory.display()))
        }
      } else {
        Ok(None)
      }
    }
    None => Err(error!("dsh directory disabled, target cannot be read")),
  }
}

/// # Create or update target
///
/// This function will create a target settings file if it does not already exist,
/// or it will update it if it is already there.
/// If the `Target` has a non-empty `password` field, the password will be stored in the keyring.
/// Note that this function is not transaction safe in the sense that when
/// upserting the settings file is successful but storing the password in the keyring is not,
/// the settings file will not be rolled back.
/// The function will return an `Err` in this case, describing the situation.
/// If upserting the settings file fails, the password will not be stored in the keyring.
///
/// ## Parameters
/// * `target` - target to create or update a settings file for
///
/// ## Returns
/// * `Ok(())` - if the target's setting file was successfully created or updated
/// * `Err(message)` - if an error occurred in either upserting the target's settings file
///   or the password in the keyring
pub(crate) fn upsert_target(target: &Target) -> DshCliResult<()> {
  match target_directory_pathbuf(&target.platform, &target.tenant)? {
    Some(target_directory_pathbuf) => {
      let target_file = target_directory_pathbuf.join(format!("{}.{}.toml", &target.platform, &target.tenant));
      serialize_and_write_to_toml_file(&target_file, target)?;
      match target.password {
        Some(ref password) => match upsert_password_to_keyring(password, &target.platform, &target.tenant) {
          Ok(_) => {
            debug!("target file '{}' and keyring upserted with target '{}'", target_file.display(), target);
            Ok(())
          }
          Err(keyring_error) => {
            debug!(
              "target file '{}' upserted with target '{}', but keyring update failed ({})",
              target_file.display(),
              target,
              keyring_error
            );
            Err(keyring_error)
          }
        },
        None => {
          debug!("target file '{}' upserted with target '{}', but password is empty", target_file.display(), target);
          Ok(())
        }
      }
    }
    None => Err(error!("dsh directory disabled, target cannot be upserted")),
  }
}

/// # List target directories
///
/// This function will read the target parameters from the target directory (if it exists).
/// The `password` field of the returned `Target` will always be `None`.
///
/// ## Parameters
/// * `platform` - target platform
/// * `tenant` - target tenant name
///
/// ## Returns
/// * `Ok<Vec<(platform, directory)>>` - If the target directory exists a `Target` will be returned.
/// * `Err<DshCliError>` - Dsh tool does not support dsh directory or was unable to determine it.
pub(crate) fn list_targets() -> DshCliResult<Vec<(DshPlatform, String)>> {
  match dsh_directory_pathbuf(TARGETS_SUBDIRECTORY)? {
    Some(targets_directory) => {
      let mut targets = vec![];
      if targets_directory.exists() && targets_directory.is_dir() {
        for target_directory_entry in targets_directory.read_dir()?.flatten() {
          if let Ok(platform) = DshPlatform::try_from(target_directory_entry.file_name().to_string_lossy().to_string().as_str()) {
            let platform_directory = targets_directory.join(platform.name());
            if platform_directory.exists() && platform_directory.is_dir() {
              for tenant_directory_entry in platform_directory.read_dir()?.flatten() {
                if tenant_directory_entry.path().is_dir() {
                  targets.push((platform.clone(), tenant_directory_entry.file_name().into_string().unwrap()))
                }
              }
            }
          }
        }
      }
      targets.sort();
      Ok(targets)
    }
    None => Err(error!("dsh directory disabled, targets cannot be listed")),
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
fn dsh_directory_pathbuf(subdirectory: &str) -> DshCliResult<Option<PathBuf>> {
  root_dsh_directory_pathbuf().map(|pathbuf| pathbuf.map(|root_directory| root_directory.join(subdirectory)))
  // match root_dsh_directory_pathbuf()? {
  //   Some(root_directory) => Ok(Some(root_directory.join(subdirectory))),
  //   None => Ok(None)
  // }
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

/// Returns `PathBuf` for target directory
///
/// Return the [PathBuf] for the target directory for the provided `platform` and `tenant`.
/// The directory name will be "$HOME/.dsh_cli/targets/\[platform.name\]/\[target\]".
///
/// * `platform` - Target platform.
/// * `tenant` - Target tenant name.
///
/// ## Returns
/// * `Ok<Some<PathBuf>>` - Pathbuf of the target directory for `platform` and `tenant`.
/// * `Ok<PathBuf>` - Dsh tool does not support storing state and settings.
/// * `Err<DshCliError>` -  Dsh directory could not be determined.
fn target_directory_pathbuf(platform: &DshPlatform, tenant: &str) -> DshCliResult<Option<PathBuf>> {
  dsh_directory_pathbuf(&format!("{}/{}/{}", TARGETS_SUBDIRECTORY, platform.name(), tenant))
}

/// Create parent directory
///
/// Create the parent directory or directories for the provided `path`. If the parent directory
/// already exists, nothing will happen.
///
/// # Parameters
/// * `path` - Path for which the parent directory or directories will be created.
///
/// # Returns
/// * `Ok<()>` - Parent directory already exists or was successfully created.
/// * `Err<DshCliError>` - Pasrent directory or directories could not be created.
fn create_parent_directories(path: &Path) -> DshCliResult<()> {
  match path.parent() {
    Some(parent) => {
      if parent.exists() {
        if parent.is_dir() {
          Ok(())
        } else {
          Err(error!("parent {} already exists but is not a directory", parent.display()))
        }
      } else {
        fs::create_dir_all(parent)?;
        log::debug!("parent directory {} created", parent.display());
        Ok(())
      }
    }
    None => Err(error!("{} has no parent", path.display())),
  }
}

lazy_static! {
  static ref DSH_DIRECTORY: DshCliResult<Option<PathBuf>> = {
    match environment_variable(ENV_VAR_DSH_CLI_HOME, None) {
      Ok(Some(dsh_directory_from_env_var)) => {
        if dsh_directory_from_env_var.is_empty() {
          Ok(None)
        } else {
          Ok(Some(PathBuf::new().join(dsh_directory_from_env_var)))
        }
      }
      Ok(None) => match my_home() {
        Ok(Some(user_home_directory)) => Ok(Some(user_home_directory.join(DEFAULT_USER_DSH_CLI_DIRECTORY))),
        Ok(None) => Err(error!(
          "could not determine dsh cli directory, check environment variable '{}' or 'HOME'",
          ENV_VAR_DSH_CLI_HOME
        )),
        Err(error) => Err(error!(
          "could not determine dsh cli directory, check environment variable '{}' or 'HOME' ({})",
          ENV_VAR_DSH_CLI_HOME, error
        )),
      },
      Err(_) => unreachable!(),
    }
  };
}
