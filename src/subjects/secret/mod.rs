pub(crate) mod capabilities;
pub(crate) mod labels;

use crate::arguments::secret_id_argument;
use crate::capability::{
  Capability, COPY_COMMAND, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS,
  UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::flags::FlagType;
use crate::global_options::expiration_option;
use crate::issues::Issue;
use crate::modifier_flags::ModifierFlagType;
use crate::secret_metadata::SecretMetadata;
use crate::subject::Subject;
use crate::subjects::secret::capabilities::{
  SecretCopy, SecretCreate, SecretDelete, SecretList, SecretListAllocationStatus, SecretListCertificates, SecretListErrors, SecretListIds, SecretListIssues, SecretListKeys,
  SecretListSystem, SecretListUsage, SecretShow, SecretShowAllocationStatus, SecretShowUsage, SecretShowValue, SecretUpdate,
};
use async_trait::async_trait;
use dsh_api::secret::SecretInjection;
use dsh_api::types::AllocationStatus;
use dsh_api::Dependant;
use serde::Serialize;
use std::sync::LazyLock;

pub struct SecretSubject {}

const SECRET_SUBJECT_TARGET: &str = "secret";

pub(crate) static SECRET_SUBJECT: LazyLock<Box<dyn Subject + Send + Sync>> = LazyLock::new(|| Box::new(SecretSubject {}));

#[async_trait]
impl Subject for SecretSubject {
  fn subject(&self) -> &'static str {
    SECRET_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH secrets.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list secrets used by the services and apps on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("s")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      COPY_COMMAND => Some(SECRET_COPY_CAPABILITY.as_ref()),
      CREATE_COMMAND => Some(SECRET_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(SECRET_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(SECRET_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(SECRET_SHOW_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(SECRET_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &SECRET_CAPABILITIES
  }
}

static SECRET_COPY_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> =
  LazyLock::new(|| Box::new(CapabilityBuilder::new(COPY_COMMAND, None, &SecretCopy {}, "Copy secret to clipboard").add_target_argument(secret_id_argument().required(true))));

static SECRET_CREATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &SecretCreate {}, "Create new secret")
      .set_long_about("Create a new secret.")
      .add_target_argument(secret_id_argument().required(true))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  )
});
static SECRET_DELETE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &SecretDelete {}, "Delete secret")
      .set_long_about("Delete a secret.")
      .add_target_argument(secret_id_argument().required(true)),
  )
});
static SECRET_LIST_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &SecretList {}, "List secrets")
      .set_long_about("Lists all secrets used by the services and apps on the DSH.")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &SecretListAllocationStatus {}, None),
        (FlagType::Certificates, &SecretListCertificates {}, None),
        (FlagType::Errors, &SecretListErrors {}, None),
        (FlagType::Ids, &SecretListIds {}, None),
        (FlagType::Issues, &SecretListIssues {}, None),
        (FlagType::Keys, &SecretListKeys {}, None),
        (FlagType::System, &SecretListSystem {}, None),
        (FlagType::Usage, &SecretListUsage {}, None),
      ])
      .add_extra_argument(expiration_option()),
  )
});
static SECRET_SHOW_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &SecretShow {}, "Show secret details")
      .add_command_executor(
        FlagType::AllocationStatus,
        &SecretShowAllocationStatus {},
        Some("Show secret allocation status".to_string()),
      )
      .add_command_executor(FlagType::Usage, &SecretShowUsage {}, Some("Show where the secret is used".to_string()))
      .add_command_executor(FlagType::Value, &SecretShowValue {}, Some("Show the secret value".to_string()))
      .add_target_argument(secret_id_argument().required(true))
      .add_extra_argument(expiration_option()),
  )
});
static SECRET_UPDATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &SecretUpdate {}, "Update secret")
      .set_long_about("Update a secret.")
      .add_target_argument(secret_id_argument().required(true))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  )
});
static SECRET_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
    SECRET_COPY_CAPABILITY.as_ref(),
    SECRET_CREATE_CAPABILITY.as_ref(),
    SECRET_DELETE_CAPABILITY.as_ref(),
    SECRET_LIST_CAPABILITY.as_ref(),
    SECRET_SHOW_CAPABILITY.as_ref(),
    SECRET_UPDATE_CAPABILITY.as_ref(),
  ]
});

/// Secret with metadata
///
/// # Fields
///
/// * `name` - `String`
/// * `id` - `Option<String>`
/// * `metadata` - `SecretMetadata`
/// * `allocation_status` - `Option<AllocationStatus>`
/// * `dependants` - `Vec<Dependant<SecretInjection>>`
#[derive(Clone, Serialize)]
pub(crate) struct SecretWithMetadata {
  pub(crate) name: String,
  pub(crate) id: Option<String>,
  pub(crate) metadata: SecretMetadata,
  pub(crate) allocation_status: Option<AllocationStatus>,
  pub(crate) dependants: Vec<Dependant<SecretInjection>>,
}

/// Check if a secret has issues.
///
/// # Parameters
/// * `secret` - `SecretWithMetadata` - Secret metadata.
/// * `days` - Number of days until expiration.
///
/// # Returns
/// * `Some(Vec<Issue>)` - List of found issues (at least one).
/// * `None` - No issues where found.
pub(crate) fn has_issues(secret: &SecretWithMetadata, days: Option<u64>) -> Option<Vec<Issue>> {
  let mut issues = issues_from_secret_metadata(&secret.metadata, days);
  if secret.id.is_none() && secret.dependants.is_empty() {
    issues.push(Issue::NotUsed)
  }
  if let Some(allocation_status) = &secret.allocation_status {
    if !allocation_status.provisioned {
      issues.push(Issue::NotProvisioned)
    }
    for notification in &allocation_status.notifications {
      if notification.remove {
        issues.push(Issue::RemovalNotification { notification: notification.clone() });
      } else {
        issues.push(Issue::CreationUpdateNotification { notification: notification.clone() });
      }
    }
  }
  if issues.is_empty() {
    None
  } else {
    Some(issues)
  }
}

/// Find issues in secret metadata.
///
/// # Parameters
/// * `secret_metadata` - `SecretMetadata` - Secret metadata.
/// * `days` - Number of days until expiration.
///
/// # Returns
/// * `Vec<Issue>` - List of found issues, possible empty.
pub(crate) fn issues_from_secret_metadata(secret_metadata: &SecretMetadata, days: Option<u64>) -> Vec<Issue> {
  let mut issues: Vec<Issue> = vec![];
  match &secret_metadata {
    SecretMetadata::Certificate { parts, .. } => {
      for part in parts {
        if let Some(issue) = Issue::timestamp_expired(part.not_after as i64, days) {
          issues.push(issue);
        }
        if let Some(issue) = Issue::timestamp_before(part.not_before as i64) {
          issues.push(issue);
        }
      }
    }
    SecretMetadata::Empty => {
      issues.push(Issue::Empty);
    }
    SecretMetadata::Error { message } => {
      issues.push(Issue::IncorrectValue { explanation: message.clone() });
    }
    SecretMetadata::Misconfiguration { message } => {
      issues.push(Issue::Misconfiguration { explanation: message.clone() });
    }
    SecretMetadata::NotFound { message } => {
      issues.push(Issue::Misconfiguration { explanation: message.clone().unwrap_or_default().to_string() });
    }
    SecretMetadata::Pki { .. } => {}
    SecretMetadata::Regular { .. } => {}
    SecretMetadata::Settings { .. } => {}
  }
  issues
}
