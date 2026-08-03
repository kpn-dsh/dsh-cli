use crate::formatters::{ColumnAlignment, Value};
use crate::formatters::{Label, SubjectFormatter};
use crate::issues::{Issue, IssueLabel};
use crate::secret_metadata::{CertificateSecretMetadata, PkiSecretMetadata, SecretMetadata};
use crate::subjects::secret::SecretWithMetadata;
use itertools::Itertools;
use serde::Serialize;
use std::cmp::PartialEq;

/// Secret metadata plus issue
///
/// # Fields
/// * `metadata` - `SecretMetadata`
/// * `issue` - `Issue`
#[derive(Clone, Serialize)]
pub(crate) struct SecretMetadataIssue {
  pub(crate) metadata: SecretMetadata,
  pub(crate) issue: Issue,
}

impl SubjectFormatter<IssueLabel> for SecretMetadataIssue {
  fn value(&self, label: &IssueLabel, target_id: &str) -> Value {
    let SecretMetadataIssue { metadata, issue } = self;
    match label {
      IssueLabel::IssueDetails => issue.value(label, target_id),
      IssueLabel::IssueKind => issue.value(label, target_id),
      IssueLabel::SubjectDescription => Value::some_or_hide(metadata.value_description()),
      IssueLabel::SubjectKind => Value::some_or_hide(metadata.kind()),
      IssueLabel::Target => Value::target(target_id),
      IssueLabel::DependencyName => Value::not_applicable(),
      IssueLabel::DependencySubject => Value::not_applicable(),
      IssueLabel::DependencyValue => Value::not_applicable(),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum SecretLabel {
  Bytes,
  Dependants,
  DerivedFrom,
  Description,
  Format,
  Issues,
  Kind,
  Label,
  Notifications,
  Provisioned,
  SecretId,
  SecretName,
  Size,
  System,
}

impl Label for SecretLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Bytes => "bytes",
      Self::Dependants => "dependants",
      Self::DerivedFrom => "derived from",
      Self::Description => "description",
      Self::Format => "format",
      Self::Issues => "issues",
      Self::Kind => "kind",
      Self::Label => "label",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::SecretId => "secret id",
      Self::SecretName => "secret name",
      Self::Size => "size",
      Self::System => "system",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretName)
  }

  fn column_alignment(&self) -> ColumnAlignment {
    match self {
      Self::Bytes => ColumnAlignment::Right,
      _ => ColumnAlignment::default(),
    }
  }
}

impl SubjectFormatter<SecretLabel> for (SecretWithMetadata, Option<u64>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    (&self.0, self.1).value(label, target_id)
  }
}

impl SubjectFormatter<SecretLabel> for (&SecretWithMetadata, Option<u64>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    let (metadata, expiration_days) = self;
    match label {
      SecretLabel::DerivedFrom => match metadata.allocation_status.clone().and_then(|allocation_status| allocation_status.derived_from) {
        Some(derived_from) => Value::plain(derived_from),
        None => Value::hide(),
      },
      SecretLabel::Notifications => match &metadata.allocation_status {
        Some(allocation_status) if !allocation_status.notifications.is_empty() => {
          Value::warn(allocation_status.notifications.iter().map(|notification| notification.to_string()).join("\n"))
        }
        _ => Value::hide(),
      },
      SecretLabel::Provisioned => match &metadata.allocation_status {
        Some(allocation_status) => {
          if allocation_status.provisioned {
            Value::plain("yes")
          } else {
            Value::plain("no")
          }
        }
        None => Value::hide(),
      },
      SecretLabel::SecretId => match &metadata.id {
        Some(secret_id) => Value::target(secret_id),
        None => Value::hide(),
      },
      SecretLabel::SecretName => Value::target(target_id),
      SecretLabel::System => {
        if metadata.id.is_some() {
          Value::plain("yes")
        } else {
          Value::plain("no")
        }
      }
      _ => SecretMetadataExpirationDays::new(metadata.metadata.clone(), *expiration_days).value(label, target_id),
    }
  }
}

impl SubjectFormatter<SecretLabel> for (SecretWithMetadata, Option<u64>, Vec<Issue>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    (&self.0, self.1, &self.2).value(label, target_id)
  }
}

/// * `SecretMetadata` - Secret metadata.
/// * `Option<u64>` - Expiration days.
/// * `Issues` - Possible issues.
impl SubjectFormatter<SecretLabel> for (&SecretWithMetadata, Option<u64>, &Vec<Issue>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    let (SecretWithMetadata { id, metadata, allocation_status, dependants, .. }, expiration_days, issues) = self;
    match label {
      SecretLabel::Dependants => Value::non_empty_or_hide(&dependants.iter().map(|dependant| format!("{:9}{}", dependant.kind(), dependant)).collect_vec()),
      SecretLabel::DerivedFrom => match allocation_status.clone().and_then(|allocation_status| allocation_status.derived_from) {
        Some(derived_from) => Value::plain(derived_from),
        None => Value::hide(),
      },
      SecretLabel::Issues => {
        if issues.is_empty() {
          Value::hide()
        } else {
          Value::vec(issues.iter().map(|issue| Value::issue(issue, expiration_days)).collect_vec())
        }
      }
      SecretLabel::Notifications => match allocation_status {
        Some(allocation_status) if !allocation_status.notifications.is_empty() => {
          Value::warn(allocation_status.notifications.iter().map(|notification| notification.to_string()).join("\n"))
        }
        _ => Value::hide(),
      },
      SecretLabel::Provisioned => match allocation_status {
        Some(allocation_status) => {
          if allocation_status.provisioned {
            Value::plain("yes")
          } else {
            Value::plain("no")
          }
        }
        None => Value::hide(),
      },
      SecretLabel::SecretId => match id {
        Some(secret_id) => Value::target(secret_id),
        None => Value::hide(),
      },
      SecretLabel::SecretName => {
        if id.is_some() {
          Value::target(format!("{}*", target_id))
        } else {
          Value::target(target_id)
        }
      }
      SecretLabel::System => {
        if id.is_some() {
          Value::plain("yes")
        } else {
          Value::plain("no")
        }
      }
      _ => SecretMetadataExpirationDays::new(metadata.clone(), None).value(label, target_id),
    }
  }
}

/// Secret metadata plus expiration days
///
/// # Fields
/// * `metadata` - `SecretMetadata`
/// * `days` - `Option<u64>`
#[derive(Clone, Serialize)]
pub(crate) struct SecretMetadataExpirationDays {
  pub(crate) metadata: SecretMetadata,
  pub(crate) days: Option<u64>,
}

impl SecretMetadataExpirationDays {
  pub(crate) fn new(metadata: SecretMetadata, days: Option<u64>) -> Self {
    Self { metadata, days }
  }
}

// TODO Expiration days is not used
impl SubjectFormatter<SecretLabel> for SecretMetadataExpirationDays {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    let SecretMetadataExpirationDays { metadata, .. } = self;
    match label {
      SecretLabel::Bytes => Value::some_or_hide(metadata.secret_size().map(|size| size.number_of_characters)),
      SecretLabel::Description => match metadata.kind() {
        Some("error") => Value::error(metadata.additional_info().map(|info| info.to_string()).unwrap_or_default()),
        _ => Value::some_or_hide(metadata.additional_info()),
      },
      SecretLabel::Format => Value::plain(metadata.format()),
      SecretLabel::Kind => match metadata.kind() {
        Some("error") => Value::error("ERROR"),
        _ => Value::some_or_hide(metadata.kind()),
      },
      SecretLabel::Label => match metadata {
        SecretMetadata::Pki { metadata, .. } => Value::plain(metadata.labels.join("/")),
        _ => Value::hide(),
      },
      SecretLabel::SecretName => Value::target(target_id),
      SecretLabel::Size => Value::some_or_hide(metadata.secret_size()),
      _ => Value::unreachable(),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum CertificateSecretLabel {
  Issuer,
  IssuerDistinguishedName,
  NotAfter,
  NotBefore,
  SecretName,
  SerialNumber,
  SubjectCommonName,
  SubjectDistinguishedName,
}

impl Label for CertificateSecretLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Issuer => "issuer",
      Self::IssuerDistinguishedName => "issuer",
      Self::NotAfter => "not after",
      Self::NotBefore => "not before",
      Self::SecretName => "secret",
      Self::SerialNumber => "serial number",
      Self::SubjectCommonName => "subject common name",
      Self::SubjectDistinguishedName => "subject",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretName)
  }
}

/// * `CertificateSecretMetadata` - Certificate secret metadata.
/// * `Option<u64>` - Expiration days.
impl SubjectFormatter<CertificateSecretLabel> for (&CertificateSecretMetadata, Option<u64>) {
  fn value(&self, label: &CertificateSecretLabel, target_id: &str) -> Value {
    let (metadata, expiration_days) = self;
    match label {
      CertificateSecretLabel::Issuer => {
        if metadata.is_self_issued() {
          Value::warn("self-issued")
        } else {
          Value::plain(metadata.issuer_common_name.clone())
        }
      }
      CertificateSecretLabel::IssuerDistinguishedName => {
        if metadata.is_self_issued() {
          Value::warn("self-issued")
        } else {
          Value::distinguished_name(metadata.issuer_distinguished_name.clone())
        }
      }
      CertificateSecretLabel::NotAfter => Value::timestamp_seconds_expired(metadata.not_after as i64, *expiration_days),
      CertificateSecretLabel::NotBefore => Value::timestamp_seconds_not_before(metadata.not_before as i64),
      CertificateSecretLabel::SecretName => Value::target(target_id),
      CertificateSecretLabel::SerialNumber => Value::plain(metadata.serial_number.clone()),
      CertificateSecretLabel::SubjectCommonName => Value::plain(metadata.subject_common_name.clone()),
      CertificateSecretLabel::SubjectDistinguishedName => Value::distinguished_name(metadata.subject_distinguished_name.clone()),
    }
  }
}

impl SubjectFormatter<CertificateSecretLabel> for (CertificateSecretMetadata, Option<u64>) {
  fn value(&self, label: &CertificateSecretLabel, target_id: &str) -> Value {
    (&self.0, self.1).value(label, target_id)
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum PkiSecretLabel {
  Algorithm,
  Labels,
  Private,
}

impl Label for PkiSecretLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Algorithm => "algorithm",
      Self::Labels => "labels",
      Self::Private => "private",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

impl SubjectFormatter<PkiSecretLabel> for &PkiSecretMetadata {
  fn value(&self, label: &PkiSecretLabel, _: &str) -> Value {
    match label {
      PkiSecretLabel::Algorithm => match &self.algorithm {
        Some(algorithm) => Value::plain(algorithm),
        None => Value::warn("missing algorithm"),
      },
      PkiSecretLabel::Labels => Value::plain(self.labels.join("\n")),
      PkiSecretLabel::Private => {
        if self.private {
          Value::plain("private")
        } else {
          Value::plain("private/public")
        }
      }
    }
  }
}
