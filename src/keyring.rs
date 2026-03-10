use crate::error::DshCliError;
use crate::{err, DshCliResult, APPLICATION_NAME};
use dsh_api::platform::DshPlatform;
use log::{debug, error};
use std::collections::HashMap;

/// # Get robot secret from keyring
///
/// ## Parameters
/// * `platform` - Platform for which the robot secret is requested.
/// * `tenant` - Tenant for which the robot secret is requested.
///
/// ## Returns
/// * `Ok(Some(secret))` - If the robot secret was found in the keyring.
/// * `Ok(None)` - If the robot secret could not be found in the keyring.
pub(crate) fn get_robot_secret_from_keyring(platform: &DshPlatform, tenant: &str) -> DshCliResult<Option<String>> {
  let target = format!("{}@{}", tenant, platform);
  match get_keyring_entry()? {
    Some(passwords) => match passwords.get(&target) {
      Some(secret) => {
        debug!("robot secret for '{}' read from keyring", target);
        Ok(Some(secret.to_string()))
      }
      None => {
        debug!("robot secret for '{}' not found in keyring", target);
        Ok(None)
      }
    },
    None => {
      debug!("robot secret for '{}' not found in keyring (keyring does not contain robot secrets)", target);
      Ok(None)
    }
  }
}

/// # Get robot secret targets
///
/// ## Returns
/// * `Ok(Vec(target))` - Sorted list of targets for which the keyring contains robot secrets.
pub(crate) fn get_robot_secret_targets() -> DshCliResult<Vec<String>> {
  match get_keyring_entry()? {
    Some(passwords) => {
      let mut targets: Vec<String> = passwords.keys().map(|target| target.to_string()).collect();
      targets.sort();
      debug!("robot secret targets read from keyring");
      Ok(targets)
    }
    None => {
      debug!("robot secret targets could not be read from keyring (keyring does not contain robot secrets)");
      Ok(vec![])
    }
  }
}

/// # Create or update robot secret in the keyring
///
/// ## Parameters
/// * `robot_secret` - Robot secret to add to the keyring.
/// * `platform` - Platform for the robot secret.
/// * `tenant` - Tenant for the robot secret.
///
/// ## Returns
/// * `Ok(())` - If the password entry was successfully written to the keyring.
pub(crate) fn upsert_robot_secret_to_keyring(robot_secret: &str, platform: &DshPlatform, tenant: &str) -> DshCliResult<()> {
  let target = format!("{}@{}", tenant, platform);
  match get_keyring_entry()? {
    Some(mut passwords) => match passwords.insert(target.clone(), robot_secret.to_string()) {
      Some(_) => {
        set_keyring_entry(&passwords)?;
        debug!("robot secret for '{}' updated in keyring", target);
        Ok(())
      }
      None => {
        set_keyring_entry(&passwords)?;
        debug!("robot secret for '{}' written to keyring", target);
        Ok(())
      }
    },
    None => {
      let mut passwords: HashMap<String, String> = HashMap::new();
      passwords.insert(target.clone(), robot_secret.to_string());
      set_keyring_entry(&passwords)?;
      debug!("robot secret for '{}' written to newly created entry in keyring", target);
      Ok(())
    }
  }
}

/// # Delete target password from the keyring
///
/// ## Parameters
/// * `platform` - Platform for which the robot secret will be deleted.
/// * `tenant` - Tenant for which the robot secret will be deleted.
///
/// ## Returns
/// * `Ok(())` - if the password entry was successfully deleted from the keyring
pub(crate) fn delete_robot_secret_from_keyring(platform: &DshPlatform, tenant: &str) -> DshCliResult<()> {
  let target = format!("{}@{}", tenant, platform);
  match get_keyring_entry()? {
    Some(mut passwords) => match passwords.remove(&target) {
      Some(_) => {
        set_keyring_entry(&passwords)?;
        debug!("robot secret for '{}' removed from keyring", target);
        Ok(())
      }
      None => {
        debug!("robot secret for '{}' not found in keyring (not removed)", target);
        Ok(())
      }
    },
    None => {
      debug!("robot secret for '{}' not removed from keyring (keyring does not contain robot secrets)", target);
      Ok(())
    }
  }
}

fn get_keyring_entry() -> DshCliResult<Option<HashMap<String, String>>> {
  let user = whoami::username()?;
  let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
  match entry.get_password() {
    Ok(secrets_json) => match serde_json::from_str::<HashMap<String, String>>(&secrets_json) {
      Ok(secrets) => Ok(Some(secrets)),
      Err(_) => err!("keyring content corrupted"),
    },
    Err(keyring_error) => match keyring_error {
      keyring::Error::NoEntry => Ok(None),
      _ => {
        error!("keyring returned an error while reading passwords ({})", keyring_error);
        Err(DshCliError::from(keyring_error))
      }
    },
  }
}

fn set_keyring_entry(passwords: &HashMap<String, String>) -> DshCliResult<()> {
  match serde_json::to_string(&passwords) {
    Ok(passwords_json) => {
      let user = whoami::username()?;
      let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
      entry.set_password(&passwords_json)?;
      Ok(())
    }
    Err(_) => err!("could not serialize passwords"),
  }
}
