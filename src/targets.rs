use crate::directory::delete_target_directory;
use crate::{DshCliResult, APPLICATION_NAME};
use dsh_api::platform::DshPlatform;
use log::debug;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};

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
  pub(crate) certificates: Vec<(String, String)>,
}

impl Target {
  pub(crate) fn new(platform: DshPlatform, tenant: String, password: Option<String>, certificates: Vec<(String, String)>) -> Self {
    Self { platform, tenant, password, certificates }
  }
}

impl Display for Target {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}@{}", self.tenant, self.platform)
  }
}

/// # Delete target
///
/// This function will delete a target directory (if it exists) and the matching target password
/// from the keyring.
/// Note that this function is not transaction safe in the sense that when
/// deleting the target directory is successful but deleting the password in the keyring is not,
/// the deletion of the target directory will not be rolled back.
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
pub(crate) fn delete_target(platform: &DshPlatform, tenant: &str) -> DshCliResult<()> {
  if delete_target_directory(platform, tenant)? {
    match delete_target_password_from_keyring(platform, tenant) {
      Ok(_) => {
        debug!("directory and keyring entry for target '{}@{}' successfully deleted", platform, tenant);
        Ok(())
      }
      Err(keyring_error) => {
        debug!(
          "directory for '{}@{}' successfully deleted, but deleting keyring entry resulted in an error ({})",
          tenant, platform, keyring_error
        );
        Err(keyring_error)
      }
    }
  } else {
    match delete_target_password_from_keyring(platform, tenant) {
      Ok(_) => {
        debug!("keyring entry for target '{}@{}' successfully deleted", platform, tenant);
        Ok(())
      }
      Err(keyring_error) => {
        debug!(
          "deleting keyring entry for target '{}@{}' resulted in an error ({})",
          tenant, platform, keyring_error
        );
        Err(keyring_error)
      }
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
pub(crate) fn get_target_password_from_keyring(platform: &DshPlatform, tenant: &str) -> DshCliResult<Option<String>> {
  let user = format!("{}.{}", platform, tenant);
  let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
  match entry.get_password() {
    Ok(password) => {
      debug!("target password for '{}@{}' read from keyring", tenant, platform);
      Ok(Some(password))
    }
    Err(_) => {
      debug!("target password for '{}@{}' could not be read from keyring", tenant, platform);
      Ok(None)
    }
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
pub(crate) fn upsert_password_to_keyring(password: &str, platform: &DshPlatform, tenant: &str) -> DshCliResult<()> {
  let user = format!("{}.{}", platform, tenant);
  let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
  entry.set_password(password)?;
  debug!("target password for '{}@{}' written to keyring", tenant, platform);
  Ok(())
}

/// # Delete target password from the keyring
///
/// ## Parameters
/// * `platform` - platform of the target
/// * `tenant` - tenant of the target
///
/// ## Returns
/// * `Ok(())` - if the password entry was successfully deleted from the keyring
pub(crate) fn delete_target_password_from_keyring(platform: &DshPlatform, tenant: &str) -> DshCliResult<()> {
  let user = format!("{}.{}", platform, tenant);
  let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
  entry.delete_credential()?;
  debug!("password for target '{}@{}' deleted from keyring", tenant, platform);
  Ok(())
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
