use crate::arguments::{LogLevel, Verbosity};
use crate::context::MatchingStyle;
use crate::formatters::OutputFormat;
use crate::APPLICATION_NAME;
use dsh_api::platform::DshPlatform;
use homedir::my_home;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};

const DSH_CLI_DIRECTORY_ENV_VAR: &str = "DSH_CLI_HOME";
const DEFAULT_USER_DSH_CLI_DIRECTORY: &str = ".dsh_cli";
const TARGETS_SUBDIRECTORY: &str = "targets";
const DEFAULT_DSH_CLI_SETTINGS_FILENAME: &str = "settings.toml";
const TOML_FILENAME_EXTENSION: &str = "toml";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Settings {
  #[serde(rename = "csv-quote", skip_serializing_if = "Option::is_none")]
  pub(crate) csv_quote: Option<char>,
  #[serde(rename = "csv-separator", skip_serializing_if = "Option::is_none")]
  pub(crate) csv_separator: Option<String>,
  #[serde(rename = "default-platform", skip_serializing_if = "Option::is_none")]
  pub(crate) default_platform: Option<String>,
  #[serde(rename = "dry-run", skip_serializing_if = "Option::is_none")]
  pub(crate) dry_run: Option<bool>,
  #[serde(rename = "default-tenant", skip_serializing_if = "Option::is_none")]
  pub(crate) default_tenant: Option<String>,
  #[serde(rename = "log-level", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level: Option<LogLevel>,
  #[serde(rename = "log-level-api", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level_api: Option<LogLevel>,
  #[serde(rename = "matching-style", skip_serializing_if = "Option::is_none")]
  pub(crate) matching_style: Option<MatchingStyle>,
  #[serde(rename = "no-escape", skip_serializing_if = "Option::is_none")]
  pub(crate) no_escape: Option<bool>,
  #[serde(rename = "output-format", skip_serializing_if = "Option::is_none")]
  pub(crate) output_format: Option<OutputFormat>,
  #[serde(rename = "quiet", skip_serializing_if = "Option::is_none")]
  pub(crate) quiet: Option<bool>,
  #[serde(rename = "show-execution-time", skip_serializing_if = "Option::is_none")]
  pub(crate) show_execution_time: Option<bool>,
  #[serde(rename = "terminal-width", skip_serializing_if = "Option::is_none")]
  pub(crate) terminal_width: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) verbosity: Option<Verbosity>,
  #[serde(skip_serializing)]
  pub(crate) file_name: Option<String>,
}

/// # Identifies the application's target
///
/// * `platform` target's platform
/// * `tenant` target's tenant name
/// * `group_user_id` - target's group and user id
/// * `password` - target's password, which will not be stored in the target settings file,
///   but instead in the keyring
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Target {
  #[serde(serialize_with = "dsh_platform_to_name", deserialize_with = "dsh_platform_from_name")]
  pub(crate) platform: DshPlatform,
  pub(crate) tenant: String,
  #[serde(rename = "group-user-id")]
  pub(crate) group_user_id: u16,
  #[serde(skip_serializing)]
  pub(crate) password: Option<String>,
}

impl Target {
  pub(crate) fn new(platform: DshPlatform, tenant: String, group_user_id: u16, password: Option<String>) -> Result<Self, String> {
    Ok(Self { platform, tenant, group_user_id, password })
  }
}

impl Display for Target {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}@{}", self.tenant, self.group_user_id, self.platform)
  }
}

pub(crate) fn read_settings(explicit_settings_filename: Option<&str>) -> Result<Option<Settings>, String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("read settings from explicit file '{}'", explicit_name);
      let settings = read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(explicit_name))?;
      Ok(settings.map(|settings| Settings { file_name: Some(explicit_name.to_string()), ..settings }))
    }
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      log::debug!("read settings from default file '{}'", default_settings_file.to_string_lossy());
      let settings = read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(default_settings_file.clone()))?;
      Ok(settings.map(|settings| Settings { file_name: Some(default_settings_file.to_string_lossy().to_string()), ..settings }))
    }
  }
}

pub(crate) fn write_settings(explicit_settings_filename: Option<&str>, settings: Settings) -> Result<(), String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("write settings to explicit file '{}'", explicit_name);
      serialize_and_write_to_toml_file::<Settings>(PathBuf::new().join(explicit_name), &settings)
    }
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      log::debug!("write settings to default file '{}'", default_settings_file.to_string_lossy());
      serialize_and_write_to_toml_file(default_settings_file, &settings)
    }
  }
}

pub(crate) fn _upsert_settings(explicit_settings_filename: Option<&str>, settings: &Settings) -> Result<(), String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("upsert explicit settings file '{}'", explicit_name);
      serialize_and_write_to_toml_file::<Settings>(PathBuf::new().join(explicit_name), settings)
    }
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      log::debug!("upsert default settings file '{}'", default_settings_file.to_string_lossy());
      serialize_and_write_to_toml_file::<Settings>(default_settings_file, settings)
    }
  }
}

pub(crate) fn all_targets() -> Result<Vec<Target>, String> {
  let mut targets = vec![];
  for target_file in fs::read_dir(targets_directory()?).map_err(|error| error.to_string())? {
    if let Ok(Some(target)) = read_and_deserialize_from_toml_file(target_file.map_err(|error| error.to_string())?.path()) {
      targets.push(target);
    }
  }
  targets.sort();
  Ok(targets)
}

/// # Delete target
///
/// This function will delete a target settings file (if it exists)
/// and the matching target password from the keyring.
/// Note that this function is not transaction safe in the sense that when
/// deleting the settings file is successful but deleting the password in the keyring is not,
/// the deletion of the settings file will not be rolled back.
/// The function will return an `Err` in this case, describing the situation.
/// On the other hand, if deleting the settings file fails,
/// the password __will__ be deleted from the keyring.
/// This situation will also return an `Err`, describing the situation.
///
/// ## Parameters
/// * `platform` - target platform
/// * `tenant` - target tenant name
///
/// ## Returns
/// * `Ok(())` - indicates that deleting the target's settings file and the password was successful
/// * `Err(message)` - if an error occurred
pub(crate) fn delete_target(platform: &DshPlatform, tenant: &str) -> Result<(), String> {
  let target_file = target_file(platform, tenant)?;
  match delete_file(&target_file) {
    Ok(_) => match delete_password_from_keyring(platform, tenant) {
      Ok(_) => {
        log::debug!("target file '{}' and keyring entry successfully deleted", target_file.to_string_lossy());
        Ok(())
      }
      Err(keyring_error) => {
        log::debug!(
          "target file '{}' successfully deleted but deleting the password from the keyring resulted in an error ({})",
          target_file.to_string_lossy(),
          keyring_error
        );
        Err(keyring_error)
      }
    },
    Err(target_file_error) => match delete_password_from_keyring(platform, tenant) {
      Ok(_) => {
        log::debug!(
          "keyring entry successfully deleted but deleting the target file '{}' resulted in an error ({})",
          target_file.to_string_lossy(),
          target_file_error
        );
        Err(target_file_error)
      }
      Err(keyring_error) => {
        log::debug!(
          "deleting the target file resulted in an error ({}), as well as deleting the password from the keyring ({})",
          target_file_error,
          keyring_error
        );
        Err(format!("{} / {}", target_file_error, keyring_error))
      }
    },
  }
}

/// # Read target and password
///
/// This function will read the target parameters from the target settings file (if it exists)
/// and the target password from the keyring.
/// If the target settings could be read, but the password entry from the keyring is not present,
/// a [`Target`] will be returned with an empty `password` field.
///
/// ## Parameters
/// * `platform` - target platform
/// * `tenant` - target tenant name
///
/// ## Returns
/// * `Ok(Some(target))` - if the target setting was available a `Target` will be returned,
///   but the `password` field can be empty if there was no matching keyring entry
/// * `Ok(None)` - if the target setting was not available
/// * `Err(message)` - if an error occurred
pub(crate) fn read_target_and_password(platform: &DshPlatform, tenant: &str) -> Result<Option<Target>, String> {
  let target_file = target_file(platform, tenant)?;
  match read_and_deserialize_from_toml_file::<Target>(&target_file)? {
    Some(target) => {
      log::debug!("read target file '{}'", target_file.to_string_lossy());
      Ok(Some(Target { password: get_password_from_keyring(platform, tenant)?, ..target }))
    }
    None => {
      log::debug!("could not read target file '{}'", target_file.to_string_lossy());
      Ok(None)
    }
  }
}

/// # Read target
///
/// This function will read the target parameters from the target settings file (if it exists).
/// The `password` field of the returned `Target` will always be `None`.
///
/// ## Parameters
/// * `platform` - target platform
/// * `tenant` - target tenant name
///
/// ## Returns
/// * `Ok(Some(target))` - if the target setting was available a `Target` will be returned.
/// * `Ok(None)` - if the target setting was not available
/// * `Err(message)` - if an error occurred
pub(crate) fn read_target(platform: &DshPlatform, tenant: &str) -> Result<Option<Target>, String> {
  let target_file = target_file(platform, tenant)?;
  match read_and_deserialize_from_toml_file::<Target>(&target_file)? {
    Some(target) => {
      log::debug!("read target file '{}'", target_file.to_string_lossy());
      Ok(Some(Target { password: None, ..target }))
    }
    None => {
      log::debug!("could not read target file '{}'", target_file.to_string_lossy());
      Ok(None)
    }
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
pub(crate) fn upsert_target(target: &Target) -> Result<(), String> {
  let target_file = target_file(&target.platform, &target.tenant)?;
  serialize_and_write_to_toml_file(&target_file, target)?;
  match target.password {
    Some(ref password) => match upsert_password_to_keyring(password, &target.platform, &target.tenant) {
      Ok(_) => {
        log::debug!("target file '{}' and keyring upserted", target_file.to_string_lossy());
        Ok(())
      }
      Err(keyring_error) => {
        log::debug!(
          "target file '{}' upserted, but keyring update failed ({})",
          target_file.to_string_lossy(),
          keyring_error
        );
        Err(keyring_error)
      }
    },
    None => {
      log::debug!("target file '{}' upserted, but password is empty", target_file.to_string_lossy());
      Ok(())
    }
  }
}

/// # Returns the dsh application directory
///
/// This function returns the application directory.
/// If it doesn't already exist the directory (and possibly its parent directories)
/// will be created.
///
/// To determine the directory, first the environment variable DSH_CLI_HOME will be checked.
/// If this variable is not defined, `${HOME}/.dsh` will be used as the application directory.
fn dsh_directory() -> Result<PathBuf, String> {
  let dsh_directory = match env::var(DSH_CLI_DIRECTORY_ENV_VAR) {
    Ok(dsh_directory) => PathBuf::new().join(dsh_directory),
    Err(_) => match my_home() {
      Ok(Some(user_home_directory)) => user_home_directory.join(DEFAULT_USER_DSH_CLI_DIRECTORY),
      _ => {
        let message = format!(
          "could not determine dsh cli directory name (check environment variable {})",
          DSH_CLI_DIRECTORY_ENV_VAR
        );
        log::error!("{}", &message);
        return Err(message);
      }
    },
  };
  match fs::create_dir_all(&dsh_directory) {
    Ok(_) => match fs::create_dir_all(dsh_directory.join(TARGETS_SUBDIRECTORY)) {
      Ok(_) => Ok(dsh_directory),
      Err(io_error) => {
        let message = format!(
          "could not create dsh targets directory '{}' ({})",
          dsh_directory.join(TARGETS_SUBDIRECTORY).to_string_lossy(),
          io_error
        );
        log::error!("{}", &message);
        Err(message)
      }
    },
    Err(io_error) => {
      let message = format!("could not create dsh directory '{}' ({})", dsh_directory.to_string_lossy(), io_error);
      log::error!("{}", &message);
      Err(message)
    }
  }
}

fn targets_directory() -> Result<PathBuf, String> {
  Ok(dsh_directory()?.join(TARGETS_SUBDIRECTORY))
}

fn target_file(platform: &DshPlatform, tenant: &str) -> Result<PathBuf, String> {
  Ok(targets_directory()?.join(format!("{}.{}.{}", platform, tenant, TOML_FILENAME_EXTENSION)))
}

fn delete_file(toml_file: &PathBuf) -> Result<(), String> {
  match fs::remove_file(toml_file) {
    Ok(_) => Ok(()),
    Err(io_error) => {
      let message = format!("could not delete file '{}' ({})", toml_file.to_string_lossy(), io_error);
      log::error!("{}", &message);
      Err(message)
    }
  }
}

/// # Get password from keyring
///
/// ## Parameters
/// * `platform` - platform of the target
/// * `tenant` - tenant of the target
///
/// ## Returns
/// * `Ok(Some(password))` - if the password entry was found in the keyring
/// * `Ok(None)` - if the password entry could not be found in the keyring
/// * `Err<String>` - if an error occurred
pub(crate) fn get_password_from_keyring(platform: &DshPlatform, tenant: &str) -> Result<Option<String>, String> {
  let user = format!("{}.{}", platform, tenant);
  match keyring::Entry::new(APPLICATION_NAME, &user) {
    Ok(entry) => match entry.get_password() {
      Ok(password) => Ok(Some(password)),
      Err(_) => Ok(None),
    },
    Err(keyring_error) => Err(keyring_error.to_string()),
  }
}

/// # Create or update password in the keyring
///
/// ## Parameters
/// * `password` - password to add to the keyring
/// * `platform` - platform of the target
/// * `tenant` - tenant of the target
///
/// ## Returns
/// * `Ok(())` - if the password entry was successfully written to the keyring
/// * `Err<String>` - if an error occurred
pub(crate) fn upsert_password_to_keyring(password: &str, platform: &DshPlatform, tenant: &str) -> Result<(), String> {
  let user = format!("{}.{}", platform, tenant);
  match keyring::Entry::new(APPLICATION_NAME, &user) {
    Ok(entry) => match entry.set_password(password) {
      Ok(_) => Ok(()),
      Err(keyring_error) => Err(keyring_error.to_string()),
    },
    Err(keyring_error) => Err(keyring_error.to_string()),
  }
}

/// # Delete password from the keyring
///
/// ## Parameters
/// * `platform` - platform of the target
/// * `tenant` - tenant of the target
///
/// ## Returns
/// * `Ok(())` - if the password entry was successfully deleted from the keyring
/// * `Err<String>` - if an error occurred
pub(crate) fn delete_password_from_keyring(platform: &DshPlatform, tenant: &str) -> Result<(), String> {
  let user = format!("{}.{}", platform, tenant);
  match keyring::Entry::new(APPLICATION_NAME, &user) {
    Ok(entry) => match entry.delete_credential() {
      Ok(_) => Ok(()),
      Err(keyring_error) => Err(keyring_error.to_string()),
    },
    Err(keyring_error) => Err(keyring_error.to_string()),
  }
}

fn read_and_deserialize_from_toml_file<T>(toml_file: impl AsRef<Path>) -> Result<Option<T>, String>
where
  T: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(&toml_file) {
    Ok(toml_string) => match toml::from_str::<T>(&toml_string) {
      Ok(deserialized_toml) => Ok(Some(deserialized_toml)),
      Err(de_error) => {
        let message = format!("could not deserialize file '{}' ({})", toml_file.as_ref().to_string_lossy(), de_error.message());
        log::error!("{}", &message);
        Err(message)
      }
    },
    Err(io_error) => match io_error.kind() {
      NotFound => Ok(None),
      _ => {
        let message = format!("could not read file '{}'", toml_file.as_ref().to_string_lossy());
        log::error!("{}", &message);
        Err(message)
      }
    },
  }
}

fn serialize_and_write_to_toml_file<T>(toml_file: impl AsRef<Path>, data: &T) -> Result<(), String>
where
  T: Serialize,
{
  match toml::to_string(data) {
    Ok(toml_string) => match fs::write(&toml_file, toml_string) {
      Ok(_) => Ok(()),
      Err(io_error) => {
        let message = format!("could not write file '{}' ({})", toml_file.as_ref().to_string_lossy(), io_error);
        log::error!("{}", &message);
        Err(message)
      }
    },
    Err(ser_error) => {
      let message = format!("could not serialize data ({})", ser_error);
      log::error!("{}", &message);
      Err(message)
    }
  }
}

fn dsh_platform_from_name<'de, D>(deserializer: D) -> Result<DshPlatform, D::Error>
where
  D: Deserializer<'de>,
{
  DshPlatform::try_from(String::deserialize(deserializer)?.as_str()).map_err(Error::custom)
}

fn dsh_platform_to_name<S>(platform: &DshPlatform, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(platform.name())
}
