use crate::{dsh_directory, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, APPLICATION_NAME, TARGETS_SUBDIRECTORY, TOML_FILENAME_EXTENSION};
use dsh_api::platform::DshPlatform;
use log::{debug, error};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;

/// # Identifies the `dsh` tool's target
///
/// * `platform` target's platform
/// * `tenant` target's tenant name
/// * `password` - target's password, which will not be stored in the target settings file,
///   but instead in the keyring
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Target {
  #[serde(serialize_with = "dsh_platform_to_name", deserialize_with = "dsh_platform_from_name")]
  pub(crate) platform: DshPlatform,
  pub(crate) tenant: String,
  #[serde(skip_serializing)]
  pub(crate) password: Option<String>,
}

impl Target {
  pub(crate) fn new(platform: DshPlatform, tenant: String, password: Option<String>) -> Result<Self, String> {
    Ok(Self { platform, tenant, password })
  }
}

impl Display for Target {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}@{}", self.tenant, self.platform)
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
    Ok(_) => match delete_target_password_from_keyring(platform, tenant) {
      Ok(_) => {
        debug!("target file '{}' and keyring entry successfully deleted", target_file.to_string_lossy());
        Ok(())
      }
      Err(keyring_error) => {
        debug!(
          "target file '{}' successfully deleted but deleting the password from the keyring resulted in an error ({})",
          target_file.to_string_lossy(),
          keyring_error
        );
        Err(keyring_error)
      }
    },
    Err(target_file_error) => match delete_target_password_from_keyring(platform, tenant) {
      Ok(_) => {
        debug!(
          "keyring entry successfully deleted but deleting the target file '{}' resulted in an error ({})",
          target_file.to_string_lossy(),
          target_file_error
        );
        Err(target_file_error)
      }
      Err(keyring_error) => {
        debug!(
          "deleting the target file resulted in an error ({}), as well as deleting the password from the keyring ({})",
          target_file_error, keyring_error
        );
        Err(format!("{} / {}", target_file_error, keyring_error))
      }
    },
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
      debug!("target '{}' read (file '{}')", target, target_file.to_string_lossy());
      Ok(Some(Target { password: None, ..target }))
    }
    None => {
      debug!("target could not read target file '{}'", target_file.to_string_lossy());
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
        debug!("target file '{}' and keyring upserted with target '{}'", target_file.to_string_lossy(), target);
        Ok(())
      }
      Err(keyring_error) => {
        debug!(
          "target file '{}' upserted with target '{}', but keyring update failed ({})",
          target_file.to_string_lossy(),
          target,
          keyring_error
        );
        Err(keyring_error)
      }
    },
    None => {
      debug!(
        "target file '{}' upserted with target '{}', but password is empty",
        target_file.to_string_lossy(),
        target
      );
      Ok(())
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

/// # Get target password from keyring
///
/// ## Parameters
/// * `platform` - platform of the target
/// * `tenant` - tenant of the target
///
/// ## Returns
/// * `Ok(Some(password))` - if the password entry was found in the keyring
/// * `Ok(None)` - if the password entry could not be found in the keyring
/// * `Err<String>` - if an error occurred
pub(crate) fn get_target_password_from_keyring(platform: &DshPlatform, tenant: &str) -> Result<Option<String>, String> {
  let user = format!("{}.{}", platform, tenant);
  match keyring::Entry::new(APPLICATION_NAME, &user) {
    Ok(entry) => match entry.get_password() {
      Ok(password) => {
        debug!("target password for '{}@{}' read from keyring", tenant, platform);
        Ok(Some(password))
      }
      Err(_) => {
        debug!("target password for '{}@{}' could not be read from keyring", tenant, platform);
        Ok(None)
      }
    },
    Err(keyring_error) => Err(keyring_error.to_string()),
  }
}

/// # Create or update target password in the keyring
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
      Ok(_) => {
        debug!("target password for '{}@{}' written to keyring", tenant, platform);
        Ok(())
      }
      Err(keyring_error) => {
        error!("could not set password for '{}@{}': {}", tenant, platform, keyring_error);
        Err(keyring_error.to_string())
      }
    },
    Err(keyring_error) => {
      error!("could not get keyring entry for '{}@{}': {}", tenant, platform, keyring_error);
      Err(keyring_error.to_string())
    }
  }
}

/// # Delete target password from the keyring
///
/// ## Parameters
/// * `platform` - platform of the target
/// * `tenant` - tenant of the target
///
/// ## Returns
/// * `Ok(())` - if the password entry was successfully deleted from the keyring
/// * `Err<String>` - if an error occurred
pub(crate) fn delete_target_password_from_keyring(platform: &DshPlatform, tenant: &str) -> Result<(), String> {
  let user = format!("{}.{}", platform, tenant);
  match keyring::Entry::new(APPLICATION_NAME, &user) {
    Ok(entry) => match entry.delete_credential() {
      Ok(_) => {
        debug!("password for target '{}@{}' deleted from keyring", tenant, platform);
        Ok(())
      }
      Err(keyring_error) => Err(keyring_error.to_string()),
    },
    Err(keyring_error) => Err(keyring_error.to_string()),
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
