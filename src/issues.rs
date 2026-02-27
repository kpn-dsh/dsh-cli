use crate::formatters::{timestamp_to_string, Label, SubjectFormatter, Value};
use chrono::{DateTime, Days, Utc};
use dsh_api::types::Notification;
use itertools::Itertools;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) enum Issue {
  /// Configuration item is not yet valid
  ///
  /// # Fields
  /// * `not_before` - Timestamp at which the configuration item becomes valid in seconds
  ///   since epoch.
  Before { not_before: i64 },

  /// Configuration item has a creation/update notification
  ///
  /// # Fields
  /// * `notification` - Creation/update notification.
  CreationUpdateNotification { notification: Notification },

  /// Configuration item references a dependency that has a creation/update notification
  ///
  /// # Fields
  /// * `subject` - Subject of the notified dependency.
  /// * `id` - Value of the configuration item that identifies the notified dependency.
  /// * `notification` - Creation/update notification.
  DependencyCreationUpdateNotification { subject: &'static str, id: String, notification: Notification },

  /// Configuration item references a dependency that cannot be found
  ///
  /// # Fields
  /// * `subject` - Subject of the missing dependency.
  /// * `id` - Value of the configuration item that identifies the missing dependency.
  DependencyNotFound { subject: &'static str, id: String },

  /// Configuration item references a dependency that is not provisioned
  ///
  /// # Fields
  /// * `subject` - Subject of the unprovisioned dependency.
  /// * `id` - Value of the configuration item that identifies the unprovisioned dependency.
  DependencyNotProvisioned { subject: &'static str, id: String },

  /// Configuration item references a dependency that has a removal notification
  ///
  /// # Fields
  /// * `subject` - Subject of the notified dependency.
  /// * `id` - Value of the configuration item that identifies the notified dependency.
  /// * `notification` - Removal notification.
  DependencyRemovalNotification { subject: &'static str, id: String, notification: Notification },

  /// Mandatory configuration item is empty/missing
  Empty,

  /// Configuration item has expired
  ///
  /// # Fields
  /// * `not_after` - Not after timestamp value of the configuration item in seconds since epoch.
  Expired { not_after: i64 },

  /// Configuration item is about to expire
  ///
  /// # Fields
  /// * `not_after` - Not after timestamp value of the configuration item in seconds since epoch.
  ExpirationOncoming { not_after: i64 },

  /// Configuration item has an incorrect/illegal value
  ///
  /// # Fields
  /// * `explanation` - Additional explanatory text.
  IncorrectValue { explanation: String },

  /// Configuration item is not properly configured
  ///
  /// # Fields
  /// * `explanation` - Additional explanatory text.
  Misconfiguration { explanation: String },

  /// Configuration item is not provisioned
  NotProvisioned,

  /// Configuration item is not used
  NotUsed,

  /// Configuration item has a removal notification
  ///
  /// # Fields
  /// * `notification` - Removal notification.
  RemovalNotification { notification: Notification },
}

#[derive(Debug, PartialEq)]
pub(crate) enum Severity {
  Error,
  Ignore,
  Warning,
}

impl Issue {
  /// Returns the severity of the issue
  pub(crate) fn severity(&self) -> Severity {
    match self {
      Self::Before { .. }
      | Self::DependencyNotFound { .. }
      | Self::DependencyNotProvisioned { .. }
      | Self::Expired { .. }
      | Self::IncorrectValue { .. }
      | Self::Misconfiguration { .. }
      | Self::NotProvisioned => Severity::Error,
      Self::Empty | Self::NotUsed => Severity::Ignore,
      Self::CreationUpdateNotification { .. }
      | Self::DependencyCreationUpdateNotification { .. }
      | Self::DependencyRemovalNotification { .. }
      | Self::ExpirationOncoming { .. }
      | Self::RemovalNotification { .. } => Severity::Warning,
    }
  }

  /// Returns the issue kind
  pub(crate) fn issue_kind(&self) -> &'static str {
    match self {
      Self::Before { .. } => "before",
      Self::CreationUpdateNotification { .. } => "creation/update notification",
      Self::DependencyCreationUpdateNotification { .. } => "dependency creation/update notification",
      Self::DependencyNotFound { .. } => "dependency not found",
      Self::DependencyNotProvisioned { .. } => "dependency not provisioned",
      Self::DependencyRemovalNotification { .. } => "dependency removal notification",
      Self::Empty { .. } => "empty",
      Self::ExpirationOncoming { .. } => "expiration oncoming",
      Self::Expired { .. } => "expired",
      Self::IncorrectValue { .. } => "incorrect value",
      Self::Misconfiguration { .. } => "misconfiguration",
      Self::NotProvisioned => "not provisioned",
      Self::NotUsed => "not used",
      Self::RemovalNotification { .. } => "removal notification",
    }
  }

  pub(crate) fn details(&self) -> Option<String> {
    match self {
      Self::Before { not_before, .. } => Some(format!("not before {}", timestamp_to_string(*not_before))),
      Self::CreationUpdateNotification { notification } => Some(notification_to_string(notification)),
      Self::DependencyCreationUpdateNotification { notification, .. } => Some(notification_to_string(notification)),
      Self::DependencyRemovalNotification { notification, .. } => Some(notification_to_string(notification)),
      Self::ExpirationOncoming { not_after } => Some(format!("will expire at {}", timestamp_to_string(*not_after))),
      Self::Expired { not_after } => Some(format!("not after {}", timestamp_to_string(*not_after))),
      Self::IncorrectValue { explanation } => Some(explanation.clone()),
      Self::Misconfiguration { explanation } => Some(explanation.clone()),
      Self::RemovalNotification { notification } => Some(notification_to_string(notification)),
      _ => None,
    }
  }

  pub(crate) fn dependency_subject(&self) -> Option<&'static str> {
    match self {
      Self::DependencyCreationUpdateNotification { subject, .. } => Some(subject),
      Self::DependencyNotFound { subject, .. } => Some(subject),
      Self::DependencyNotProvisioned { subject, .. } => Some(subject),
      Self::DependencyRemovalNotification { subject, .. } => Some(subject),
      _ => None,
    }
  }

  pub(crate) fn dependency_id(&self) -> Option<&String> {
    match self {
      Self::DependencyCreationUpdateNotification { id, .. } => Some(id),
      Self::DependencyNotFound { id, .. } => Some(id),
      Self::DependencyNotProvisioned { id, .. } => Some(id),
      Self::DependencyRemovalNotification { id, .. } => Some(id),
      _ => None,
    }
  }

  pub(crate) fn datetime_before(not_before: &DateTime<Utc>) -> Option<Self> {
    if not_before > &Utc::now() {
      Some(Self::Before { not_before: not_before.timestamp() })
    } else {
      None
    }
  }

  pub(crate) fn datetime_expired(not_after: &DateTime<Utc>, days: Option<u64>) -> Option<Self> {
    if not_after < &Utc::now() {
      Some(Self::Expired { not_after: not_after.timestamp() })
    } else {
      match days {
        Some(warning_days) => match Utc::now().checked_add_days(Days::new(warning_days)) {
          Some(expiration_warning_date) => {
            if not_after < &expiration_warning_date {
              Some(Self::ExpirationOncoming { not_after: not_after.timestamp() })
            } else {
              None
            }
          }
          None => Some(Self::IncorrectValue { explanation: format!("could not determine oncoming expiration ({}/{})", not_after, warning_days) }),
        },
        None => None,
      }
    }
  }

  pub(crate) fn timestamp_before(not_before: i64) -> Option<Self> {
    match DateTime::from_timestamp_secs(not_before) {
      Some(not_before_datetime) => Self::datetime_before(&not_before_datetime),
      None => Some(Self::IncorrectValue { explanation: format!("could not convert {} to valid datetime", not_before) }),
    }
  }

  pub(crate) fn timestamp_expired(not_after: i64, days: Option<u64>) -> Option<Self> {
    match DateTime::from_timestamp_secs(not_after) {
      Some(not_after_datetime) => Self::datetime_expired(&not_after_datetime, days),
      None => Some(Self::IncorrectValue { explanation: format!("could not convert {} to valid datetime", not_after) }),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum IssueLabel {
  ConfigurationItem,
  IssueDetails,
  IssueKind,
  Subject,
  SubjectKind,
  Target,
  Value,
}

impl Label for IssueLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::ConfigurationItem => "configuration item",
      Self::IssueDetails => "issue details",
      Self::IssueKind => "issue",
      Self::Subject => "subject",
      Self::SubjectKind => "subject kind",
      Self::Target => "target id",
      Self::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<IssueLabel> for (&str, Issue) {
  fn value(&self, label: &IssueLabel, target_id: &str) -> Value {
    let (configuration_item, issue) = self;
    match label {
      IssueLabel::ConfigurationItem => Value::plain(configuration_item),
      _ => issue.value(label, target_id),
    }
  }
}

impl SubjectFormatter<IssueLabel> for Issue {
  fn value(&self, label: &IssueLabel, target_id: &str) -> Value {
    match label {
      IssueLabel::ConfigurationItem => Value::unreachable(),
      IssueLabel::IssueDetails => Value::option(self.details()),
      IssueLabel::IssueKind => match self.severity() {
        Severity::Error => Value::error(self.issue_kind()),
        Severity::Ignore => Value::ignore(self.issue_kind()),
        Severity::Warning => Value::warn(self.issue_kind()),
      },
      IssueLabel::Subject => Value::option(self.dependency_id()),
      IssueLabel::SubjectKind => Value::option(self.dependency_subject()),
      IssueLabel::Target => Value::target(target_id),
      IssueLabel::Value => Value::unreachable(),
    }
  }
}

pub(crate) fn notification_to_string(notification: &Notification) -> String {
  if notification.args.is_empty() {
    notification.message.to_string()
  } else {
    format!(
      "{}\n{}",
      notification.message,
      notification.args.iter().map(|(key, value)| format!("{}:{}", key, value)).join("\n"),
    )
  }
}

pub(crate) static _ISSUE_LABELS_LIST: [IssueLabel; 6] =
  [IssueLabel::Target, IssueLabel::ConfigurationItem, IssueLabel::Value, IssueLabel::Subject, IssueLabel::IssueKind, IssueLabel::IssueDetails];
