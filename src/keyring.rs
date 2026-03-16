use crate::error::DshCliError;
use crate::{err, DshCliResult, APPLICATION_NAME};
use dsh_api::platform::DshPlatform;
use itertools::Itertools;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maps platform name to `PlatformEntry`
type KeyringEntry = HashMap<String, PlatformEntry>;

#[derive(Deserialize, Serialize)]
struct PlatformEntry {
  /// Maps tenant name to `TenantEntry`
  tenants: HashMap<String, TenantEntry>,
}

#[derive(Deserialize, Serialize)]
struct TenantEntry {
  /// Maps secret names to their value
  secrets: HashMap<String, String>,
}

/// # Get secret from keyring
///
/// ## Parameters
/// * `platform` - Platform for which the secret is requested.
/// * `tenant_name` - Tenant for which the secret is requested.
/// * `secret_name` - Name of the requested secret.
///
/// ## Returns
/// * `Ok(Some(secret))` - If the secret was found in the keyring.
/// * `Ok(None)` - If the secret could not be found in the keyring.
pub(crate) fn get_secret_from_keyring(platform: &DshPlatform, tenant_name: &str, secret_name: &str) -> DshCliResult<Option<String>> {
  let target = format!("{}@{}", tenant_name, platform);
  match get_keyring_entry()? {
    Some(keyring_entry) => match keyring_entry.get(platform.name()) {
      Some(platform_entry) => match platform_entry.tenants.get(tenant_name) {
        Some(tenant_entry) => match tenant_entry.secrets.get(secret_name) {
          Some(secret) => {
            debug!("secret '{}' for '{}' read from keyring", secret_name, target);
            Ok(Some(secret.to_string()))
          }
          None => {
            debug!("secret '{}' for '{}' not found in keyring", secret_name, target);
            Ok(None)
          }
        },
        None => {
          debug!(
            "secret '{}' for '{}' not found in keyring (no entry for tenant '{}')",
            secret_name, target, tenant_name
          );
          Ok(None)
        }
      },
      None => {
        debug!(
          "secret '{}' for '{}' not found in keyring (no entry for platform '{}')",
          secret_name, target, platform
        );
        Ok(None)
      }
    },
    None => {
      debug!("secret '{}' for '{}' not found keyring (keyring does not contain secrets)", secret_name, target);
      Ok(None)
    }
  }
}

/// # Get secret targets
///
/// ## Returns
/// * `Ok(Vec<(String, Vec(String))>)` - List of tuples (sorted by platform name) where
///   each tuple consists of
///   * `String` - Platform name.
///   * `Vec(String)` - Sorted list of tenants for the platform.
pub(crate) fn get_secret_targets() -> DshCliResult<Vec<(String, Vec<String>)>> {
  match get_keyring_entry()? {
    Some(keyring_entry) => {
      let mut secret_targets: Vec<(String, Vec<String>)> = keyring_entry
        .into_iter()
        .flat_map(|(platform_name, platform_entry)| {
          if platform_entry.tenants.is_empty() {
            None
          } else {
            let mut platform_tenants: Vec<String> = platform_entry
              .tenants
              .into_iter()
              .flat_map(|(tenant_name, tenant_entry)| if tenant_entry.secrets.is_empty() { None } else { Some(tenant_name) })
              .collect_vec();
            platform_tenants.sort();
            Some((platform_name, platform_tenants))
          }
        })
        .collect_vec();
      secret_targets.sort_by(|(platform_name_a, _), (platform_name_b, _)| platform_name_a.cmp(platform_name_b));
      debug!("secret targets read from keyring");
      Ok(secret_targets)
    }
    None => {
      debug!("secret targets could not be read from keyring (keyring does not contain secrets)");
      Ok(vec![])
    }
  }
}

/// # Create or update secret in the keyring
///
/// ## Parameters
/// * `platform` - Platform for the secret.
/// * `tenant_name` - Tenant for the secret.
/// * `secret_name` - Name of the secret to add to the keyring.
/// * `secret_value` - Value of the secret to add to the keyring.
///
/// ## Returns
/// * `Ok(())` - If the secret was successfully written to the keyring.
pub(crate) fn upsert_secret_to_keyring(platform: &DshPlatform, tenant_name: &str, secret_name: &str, secret_value: &str) -> DshCliResult<()> {
  let target = format!("{}@{}", tenant_name, platform);
  match get_keyring_entry()? {
    Some(mut keyring_entry) => {
      let platform_entry = keyring_entry.entry(platform.name().to_string());
      platform_entry.and_modify(|pe| {
        let tenant_entry = pe.tenants.entry(tenant_name.to_string());
        tenant_entry.and_modify(|tenant| {
          _ = tenant.secrets.insert(secret_name.to_string(), secret_value.to_string());
        });
      });
      debug!("secret '{}' for '{}' upserted in keyring", secret_name, target);
      set_keyring_entry(&keyring_entry)
    }
    None => {
      let keyring_entry: KeyringEntry = HashMap::from([(
        platform.name().to_string(),
        PlatformEntry {
          tenants: HashMap::from([(
            tenant_name.to_string(),
            TenantEntry { secrets: HashMap::from([(secret_name.to_string(), secret_value.to_string())]) },
          )]),
        },
      )]);
      set_keyring_entry(&keyring_entry)?;
      debug!("secret '{}' for '{}' written to newly created keyring entry", secret_name, target);
      Ok(())
    }
  }
}

/// # Remove target secret from the keyring
///
/// If the tenant entry is empty after the removal of the secret, the tenants entry itself will
/// also be removed. Similar, if the platform entry is empty after the removal of the tenant
/// entry, the platform entry itself will also be removed.
///
/// ## Parameters
/// * `platform` - Platform for which the secret will be removed.
/// * `tenant_name` - Tenant for which the secret will be removed.
/// * `secret_name` - Name of the secret that will be removed.
///
/// ## Returns
/// * `Ok(())` - If the secret entry was successfully removed from the keyring.
pub(crate) fn remove_secret_from_keyring(platform: &DshPlatform, tenant_name: &str, secret_name: &str) -> DshCliResult<()> {
  let target = format!("{}@{}", tenant_name, platform);
  match get_keyring_entry()? {
    Some(mut keyring_entry) => match keyring_entry.get_mut(platform.name()) {
      Some(platform_entry) => match platform_entry.tenants.get_mut(tenant_name) {
        Some(tenant_entry) => match tenant_entry.secrets.remove(secret_name) {
          Some(_) => {
            debug!("secret '{}' for '{}' removed from keyring", secret_name, target);
            if tenant_entry.secrets.is_empty() && platform_entry.tenants.remove(tenant_name).is_some() {
              debug!("entry for '{}' removed from keyring (did not contain any secrets)", target);
              if platform_entry.tenants.is_empty() && keyring_entry.remove(platform.name()).is_some() {
                debug!("entry for platform '{}' removed from keyring (did not contain any tenants)", platform);
              }
            }
            set_keyring_entry(&keyring_entry)?;
            Ok(())
          }
          None => {
            debug!("secret '{}' for '{}' not found in keyring (not removed)", secret_name, target);
            Ok(())
          }
        },
        None => {
          debug!(
            "secret '{}' for '{}' not removed from keyring (keyring does not contain entries for tenant '{}')",
            secret_name, target, tenant_name
          );
          Ok(())
        }
      },
      None => {
        debug!(
          "secret '{}' for '{}' not removed from keyring (keyring does not contain entries for platform '{}')",
          secret_name, target, platform
        );
        Ok(())
      }
    },
    None => {
      debug!(
        "secret '{}' for '{}' not removed from keyring (keyring does not contain secrets)",
        secret_name, target
      );
      Ok(())
    }
  }
}

/// # Remove tenant entry from the keyring
///
/// If the platform entry is empty after the removal of the tenant entry, the platform entry
/// itself will also be removed.
///
/// ## Parameters
/// * `platform` - Platform for which the tenant entry will be removed.
/// * `tenant_name` - Tenant for which the entry will be removed.
///
/// ## Returns
/// * `Ok(())` - If the tenant entry was successfully deleted from the keyring.
pub(crate) fn _remove_tenant_from_keyring(platform: &DshPlatform, tenant_name: &str) -> DshCliResult<()> {
  let target = format!("{}@{}", tenant_name, platform);
  match get_keyring_entry()? {
    Some(mut keyring_entry) => match keyring_entry.get_mut(platform.name()) {
      Some(platform_entry) => match platform_entry.tenants.remove(tenant_name) {
        Some(_) => {
          debug!("entry for '{}' removed from keyring", target);
          if platform_entry.tenants.is_empty() && keyring_entry.remove(platform.name()).is_some() {
            debug!("entry for platform '{}' removed from keyring (did not contain any tenants)", platform);
          }
          set_keyring_entry(&keyring_entry)?;
          Ok(())
        }
        None => {
          debug!("entry for '{}' not found in keyring (not removed)", target);
          Ok(())
        }
      },
      None => {
        debug!(
          "entry for '{}' not removed from keyring (keyring does not contain entries for platform '{}')",
          target, platform
        );
        Ok(())
      }
    },
    None => {
      debug!("entry for '{}' not removed from keyring (keyring does not contain entry)", target);
      Ok(())
    }
  }
}

/// # Remove platform entry from the keyring
///
/// ## Parameters
/// * `platform` - Platform for which the entry will be removed.
///
/// ## Returns
/// * `Ok(())` - If the tenant entry was successfully deleted from the keyring.
pub(crate) fn _remove_platform_from_keyring(platform: &DshPlatform) -> DshCliResult<()> {
  match get_keyring_entry()? {
    Some(mut keyring_entry) => match keyring_entry.remove(platform.name()) {
      Some(_) => {
        set_keyring_entry(&keyring_entry)?;
        debug!("entry for platform '{}' removed from keyring", platform);
        Ok(())
      }
      None => {
        debug!("entry for platform '{}' not found in keyring (not removed)", platform);
        Ok(())
      }
    },
    None => {
      debug!("entry for platform '{}' not removed from keyring (keyring does not contain entry)", platform);
      Ok(())
    }
  }
}

fn get_keyring_entry() -> DshCliResult<Option<KeyringEntry>> {
  let user = whoami::username()?;
  let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
  match entry.get_password() {
    Ok(secrets_json) => match serde_json::from_str::<KeyringEntry>(&secrets_json) {
      Ok(secrets) => Ok(Some(secrets)),
      Err(_) => err!("keyring entry corrupted"),
    },
    Err(keyring_error) => match keyring_error {
      keyring::Error::NoEntry => Ok(None),
      _ => {
        error!("keyring returned an error while reading entry ({})", keyring_error);
        Err(DshCliError::from(keyring_error))
      }
    },
  }
}

fn set_keyring_entry(keyring_entry: &KeyringEntry) -> DshCliResult<()> {
  match serde_json::to_string(&keyring_entry) {
    Ok(keyring_entry_json) => {
      let user = whoami::username()?;
      let entry = keyring::Entry::new(APPLICATION_NAME, &user)?;
      entry.set_password(&keyring_entry_json)?;
      Ok(())
    }
    Err(_) => err!("could not serialize keyring entry"),
  }
}
