pub(crate) mod capabilities;
pub(crate) mod labels;

use crate::arguments::certificate_id_argument;
use crate::capability::{Capability, DELETE_COMMAND, DELETE_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::flags::FlagType;
use crate::formatters::vec_to_table;
use crate::global_options::expiration_option;
use crate::issues::{Issue, IssueDescription, Severity};
use crate::secret_metadata::SecretMetadata;
use crate::subject::Subject;
use crate::subjects::certificate::capabilities::{
  CertificateDelete, CertificateList, CertificateListAllocationStatus, CertificateListConfiguration, CertificateListErrors, CertificateListIds, CertificateListIssues,
  CertificateListUsage, CertificateShow, CertificateShowAllocationStatus, CertificateShowUsage,
};
use crate::subjects::secret;
use crate::subjects::secret::SecretWithMetadata;
use async_trait::async_trait;
use dsh_api::error::DshApiResult;
use dsh_api::types::CertificateStatus;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::LazyLock;

struct CertificateSubject {}

const CERTIFICATE_SUBJECT_TARGET: &str = "certificate";

lazy_static! {
  pub(crate) static ref CERTIFICATE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(CertificateSubject {});
}

#[async_trait]
impl Subject for CertificateSubject {
  fn subject(&self) -> &'static str {
    CERTIFICATE_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH certificates.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list certificates deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("c")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(CERTIFICATE_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(CERTIFICATE_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(CERTIFICATE_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &CERTIFICATE_CAPABILITIES
  }
}

static CERTIFICATE_DELETE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(
      DELETE_COMMAND,
      Some(DELETE_COMMAND_ALIAS),
      &CertificateDelete {},
      "Delete certificate configuration",
    )
    .add_target_argument(certificate_id_argument().required(true)),
  )
});
static CERTIFICATE_LIST_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &CertificateList {}, "List certificates")
      .set_long_about("Lists all available certificates.")
      .add_extra_argument(expiration_option())
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateListAllocationStatus {}, None),
        (FlagType::Configuration, &CertificateListConfiguration {}, None),
        (FlagType::Errors, &CertificateListErrors {}, None),
        (FlagType::Ids, &CertificateListIds {}, None),
        (FlagType::Issues, &CertificateListIssues {}, None),
        (FlagType::Usage, &CertificateListUsage {}, None),
      ]),
  )
});
static CERTIFICATE_SHOW_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &CertificateShow {}, "Show certificate configuration")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateShowAllocationStatus {}, None),
        (FlagType::Usage, &CertificateShowUsage {}, None),
      ])
      .add_target_argument(certificate_id_argument().required(true))
      .add_extra_argument(expiration_option()),
  )
});
static CERTIFICATE_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> =
  LazyLock::new(|| vec![CERTIFICATE_DELETE_CAPABILITY.as_ref(), CERTIFICATE_LIST_CAPABILITY.as_ref(), CERTIFICATE_SHOW_CAPABILITY.as_ref()]);

/// Check if a certificate has issues.
///
/// # Parameters
/// * `certificate_status`
/// * `secrets` - List of [`SecretTuple`]s describing all secrets. Each tuple consists of:
///   * `String` - Secret name.
///   * `Option<String>` - Secret id when it is a system secret.
///   * `SecretMetadata` - Secret metadata.
///   * `Option<AllocationStatus>` - Secret allocation status.
///   * `Vec<Dependant<SecretInjection>>` - List of apps, applications and proxies that depend
///     on the secret.
/// * `days` - Number of days until expiration.
/// * `only_errors` - If `true` only issues with severity level `Severity::Error` will be returned.
///
/// # Returns
/// * `Some(Vec<IssueDescription>)` - List of tuples describing the issues found
///   (at least one).
/// * `None` - No issues where found.
fn has_issues(certificate_status: DshApiResult<CertificateStatus>, secrets: &[SecretWithMetadata], days: Option<u64>, only_errors: bool) -> Option<Vec<IssueDescription<'_>>> {
  let certificate_status = match certificate_status {
    Ok(certificate_status) => certificate_status,
    Err(_) => return Some(vec![(None, Issue::Unexpected { message: "could not get certificate status".to_string() })]),
  };

  let mut issues: Vec<IssueDescription> = vec![];

  if !certificate_status.status.provisioned {
    issues.push((None, Issue::NotProvisioned))
  }
  for notification in &certificate_status.status.notifications {
    if notification.remove {
      issues.push((None, Issue::RemovalNotification { notification: notification.clone() }));
    } else {
      issues.push((None, Issue::CreationUpdateNotification { notification: notification.clone() }));
    }
  }

  if let Some(actual_certificate) = certificate_status.actual {
    let mut cert_chain_secret_issues = secret_issues(&actual_certificate.cert_chain_secret, days, only_errors, "cert chain", secrets);
    if !cert_chain_secret_issues.iter().any(|(_, issue)| matches!(issue, Issue::Expired { .. })) {
      if let Some(issue) = Issue::datetime_expired(&actual_certificate.not_after, days) {
        if !only_errors || issue.severity() == Severity::Error {
          issues.push((None, issue));
        }
      }
    }

    if !cert_chain_secret_issues.iter().any(|(_, issue)| matches!(issue, Issue::Before { .. })) {
      if let Some(issue) = Issue::datetime_before(&actual_certificate.not_before) {
        if !only_errors || issue.severity() == Severity::Error {
          issues.push((None, issue));
        }
      }
    }
    if secret_is_certificate(&actual_certificate.cert_chain_secret, secrets).is_some_and(|a| !a) {
      cert_chain_secret_issues.push((
        Some(("cert chain", actual_certificate.cert_chain_secret.to_string(), "secret")),
        Issue::Misconfiguration { explanation: "".to_string() },
      ))
    }
    issues.append(&mut cert_chain_secret_issues);

    issues.append(&mut secret_issues(&actual_certificate.key_secret, days, only_errors, "key", secrets));

    if let Some(passphrase_secret) = actual_certificate.passphrase_secret {
      issues.append(&mut secret_issues(&passphrase_secret, days, only_errors, "passphrase", secrets));
    }
  }

  if only_errors {
    issues.retain(|(_, issue)| issue.severity() == Severity::Error);
  }
  if issues.is_empty() {
    None
  } else {
    Some(issues)
  }
}

fn secret_issues<'a>(secret: &str, days: Option<u64>, only_errors: bool, secret_attribute: &'static str, secrets: &[SecretWithMetadata]) -> Vec<IssueDescription<'a>> {
  let mut issues: Vec<IssueDescription<'_>> = vec![];
  match secrets.iter().find(|SecretWithMetadata { name, .. }| name == secret) {
    Some(secret_tuple) => {
      if let Some(cert_chain_secret_issues) = secret::has_issues(secret_tuple, days) {
        for issue in cert_chain_secret_issues {
          if !only_errors || issue.severity() == Severity::Error {
            issues.push((Some((secret_attribute, secret.to_string(), "secret")), issue));
          }
        }
      }
    }
    None => {
      issues.push((Some((secret_attribute, secret.to_string(), "secret")), Issue::NotFound {}));
    }
  }
  issues
}

fn secret_is_certificate(secret: &str, secrets: &[SecretWithMetadata]) -> Option<bool> {
  secrets
    .iter()
    .find(|SecretWithMetadata { name, .. }| *name == secret)
    .map(|SecretWithMetadata { metadata, .. }| matches!(metadata, SecretMetadata::Certificate { .. }))
}

pub(crate) fn format_distinguished_name(distinguished_name: &str) -> String {
  const ATTRIBUTES: [&str; 8] = ["CN", "O", "OU", "L", "S", "SP", "ST", "C"];
  let map = distinguished_name_to_map(distinguished_name);
  let mut attribute_value_pairs = vec![];
  for attribute in ATTRIBUTES {
    if let Some(value) = map.get(attribute) {
      attribute_value_pairs.push((attribute.to_string(), vec![value.to_string()]))
    }
  }
  for (attribute, value) in map {
    if !ATTRIBUTES.contains(&attribute) {
      attribute_value_pairs.push((attribute.to_string(), vec![value.to_string()]))
    }
  }
  vec_to_table(&attribute_value_pairs)
}

/// Get relative distinguished name from distinguished name string.
///
/// ## Examples
/// ```
/// let common_name = get_relative_distinguished_name(
///   "CN=tenant.org,O=Tenant Organization,L=City,ST=State,C=NL",
///   "CN",
/// );
/// assert_eq!(common_name, Some("tenant.org"));
/// ```
///
/// ## Parameters
/// * `distinguished_name` - Haystack distinguished name.
/// * `target_rdn_type` - Case sensitive target relative distinguished name type (e.g. "CN").
pub(crate) fn get_relative_distinguished_name<'a>(distinguished_name: &'a str, target_rdn_type: &str) -> Option<&'a str> {
  distinguished_name.split(",").find_map(|rdn_pair| {
    rdn_pair
      .split("=")
      .collect_array()
      .and_then(|[rdn_type, rdn_value]| if rdn_type == target_rdn_type { Some(rdn_value) } else { None })
  })
}

pub(crate) fn distinguished_name_to_map(distinguished_name: &str) -> HashMap<&str, &str> {
  distinguished_name
    .split(",")
    .flat_map(|rdn_pair| rdn_pair.split("=").collect_tuple())
    .collect::<HashMap<_, _>>()
}

// pub(crate) fn distinguished_name_to_map(distinguished_name: &str) -> HashMap<String, String> {
//   distinguished_name
//     .split(",")
//     .map(|rdn_pair| {
//       let attribute_value = rdn_pair.split("=").map(|s| s.trim().to_string()).collect_vec();
//       (
//         attribute_value.first().cloned().unwrap_or_default().to_string(),
//         attribute_value.get(1).cloned().unwrap_or_default().to_string(),
//       )
//     })
//     .collect::<HashMap<_, _>>()
// }

#[test]
fn test_get_relative_distinguished_name() {
  const DISTINGUISHED_NAME: &str = "CN=tenant.org,O=Tenant Organization,L=City,ST=State,C=NL";
  assert_eq!(get_relative_distinguished_name(DISTINGUISHED_NAME, "CN"), Some("tenant.org"));
  assert_eq!(get_relative_distinguished_name(DISTINGUISHED_NAME, "O"), Some("Tenant Organization"));
  assert_eq!(get_relative_distinguished_name(DISTINGUISHED_NAME, "L"), Some("City"));
  assert_eq!(get_relative_distinguished_name(DISTINGUISHED_NAME, "ST"), Some("State"));
  assert_eq!(get_relative_distinguished_name(DISTINGUISHED_NAME, "C"), Some("NL"));
  assert!(get_relative_distinguished_name(DISTINGUISHED_NAME, "OU").is_none());
}
