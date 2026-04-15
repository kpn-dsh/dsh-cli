use crate::formatters::{timestamp_to_string, Label, SubjectFormatter, Value};
use chrono::{DateTime, Days, Utc};
use dsh_api::types::Notification;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) enum Issue {
  /// Configuration item is not yet valid
  ///
  /// ## Fields
  /// * `not_before` - Timestamp at which the configuration item becomes valid in seconds
  ///   since epoch.
  Before { not_before: i64 },

  /// Configuration item has a creation/update notification
  ///
  /// ## Fields
  /// * `notification` - Creation/update notification.
  CreationUpdateNotification { notification: Notification },

  /// Mandatory configuration item is empty/missing
  Empty,

  /// Configuration item has expired
  ///
  /// ## Fields
  /// * `not_after` - Not after timestamp value of the configuration item in seconds since epoch.
  Expired { not_after: i64 },

  /// Configuration item is about to expire
  ///
  /// ## Fields
  /// * `not_after` - Not after timestamp value of the configuration item in seconds since epoch.
  ExpirationUpcoming { not_after: i64 },

  /// Configuration item has an incorrect/illegal value
  ///
  /// ## Fields
  /// * `explanation` - Additional explanatory text.
  IncorrectValue { explanation: String },

  /// Configuration item is not properly configured
  ///
  /// ## Fields
  /// * `explanation` - Additional explanatory text.
  Misconfiguration { explanation: String },

  /// Configuration item could not be found
  NotFound,

  /// Configuration item is not provisioned
  NotProvisioned,

  /// Configuration item is not used
  NotUsed,

  /// Configuration item has a removal notification
  ///
  /// ## Fields
  /// * `notification` - Removal notification.
  RemovalNotification { notification: Notification },

  /// Something unexpected happened
  ///
  /// ## Fields
  /// * `message` - Describes what happened.
  Unexpected { message: String },
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
      | Self::Expired { .. }
      | Self::IncorrectValue { .. }
      | Self::Misconfiguration { .. }
      | Self::NotFound
      | Self::NotProvisioned
      | Self::Unexpected { .. } => Severity::Error,
      Self::Empty | Self::NotUsed => Severity::Ignore,
      Self::CreationUpdateNotification { .. } | Self::ExpirationUpcoming { .. } | Self::RemovalNotification { .. } => Severity::Warning,
    }
  }

  /// Returns the issue kind
  pub(crate) fn issue_kind(&self) -> &'static str {
    match self {
      Self::Before { .. } => "before",
      Self::CreationUpdateNotification { .. } => "creation/update notification",
      Self::Empty { .. } => "empty",
      Self::ExpirationUpcoming { .. } => "expiration upcoming",
      Self::Expired { .. } => "expired",
      Self::IncorrectValue { .. } => "incorrect value",
      Self::Misconfiguration { .. } => "misconfiguration",
      Self::NotFound => "not found",
      Self::NotProvisioned => "not provisioned",
      Self::NotUsed => "not used",
      Self::RemovalNotification { .. } => "removal notification",
      Self::Unexpected { .. } => "unexpected error",
    }
  }

  pub(crate) fn details(&self) -> Option<String> {
    match self {
      Self::Before { not_before, .. } => Some(format!("not before {}", timestamp_to_string(*not_before))),
      Self::CreationUpdateNotification { notification } => Some(notification.render_message()),
      Self::Empty => None,
      Self::ExpirationUpcoming { not_after } => Some(format!("will expire at {}", timestamp_to_string(*not_after))),
      Self::Expired { not_after } => Some(format!("not after {}", timestamp_to_string(*not_after))),
      Self::IncorrectValue { explanation } => Some(explanation.clone()),
      Self::Misconfiguration { explanation } => Some(explanation.clone()),
      Self::NotFound => None,
      Self::NotProvisioned => None,
      Self::NotUsed => None,
      Self::RemovalNotification { notification } => Some(notification.render_message()),
      Self::Unexpected { message } => Some(message.clone()),
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
              Some(Self::ExpirationUpcoming { not_after: not_after.timestamp() })
            } else {
              None
            }
          }
          None => Some(Self::IncorrectValue { explanation: format!("could not determine upcoming expiration ({}/{})", not_after, warning_days) }),
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
  DependencyName,
  DependencySubject,
  DependencyValue,
  IssueDetails,
  IssueKind,
  SubjectDescription,
  SubjectKind,
  Target,
}

impl Label for IssueLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DependencyName => "dependency",
      Self::DependencySubject => "dependency kind",
      Self::DependencyValue => "dependency value",
      Self::IssueDetails => "issue details",
      Self::IssueKind => "issue",
      Self::SubjectDescription => "subject description",
      Self::SubjectKind => "subject kind",
      Self::Target => "target id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

/// Tuple that describes an issue:
/// * `Option<(&str, String)>` - Determines what caused the issue:
///   * `Some<(&str, &str, String)>` - Issue concerns a dependency of the subject:
///     * `&str` - Dependency name.
///     * `String`  - Dependency value.
///     * `&str` - Dependency subject.
///   * `None` - Issue concerns the subject itself.
/// * `Issue` - Describes the issue.
pub(crate) type IssueDescription<'a> = (Option<(&'a str, String, &'a str)>, Issue);

impl SubjectFormatter<IssueLabel> for IssueDescription<'_> {
  fn value(&self, label: &IssueLabel, target_id: &str) -> Value {
    let (attribute, issue): &(Option<(&str, String, &str)>, Issue) = self;
    match attribute {
      Some((dependency_name, dependency_value, dependency_subject)) => match label {
        IssueLabel::DependencyName => Value::plain(dependency_name),
        IssueLabel::DependencySubject => Value::plain(dependency_subject),
        IssueLabel::DependencyValue => Value::plain(dependency_value),
        IssueLabel::SubjectDescription => Value::empty(),
        IssueLabel::SubjectKind => Value::empty(),
        _ => issue.value(label, target_id),
      },
      None => match label {
        IssueLabel::DependencyName => Value::empty(),
        IssueLabel::DependencySubject => Value::empty(),
        IssueLabel::DependencyValue => Value::empty(),
        IssueLabel::SubjectDescription => Value::empty(),
        IssueLabel::SubjectKind => Value::empty(),
        _ => issue.value(label, target_id),
      },
    }
  }
}

impl SubjectFormatter<IssueLabel> for Issue {
  fn value(&self, label: &IssueLabel, target_id: &str) -> Value {
    match label {
      IssueLabel::DependencyName => Value::unreachable(),
      IssueLabel::DependencySubject => Value::unreachable(),
      IssueLabel::DependencyValue => Value::unreachable(),
      IssueLabel::IssueDetails => Value::option(self.details()),
      IssueLabel::IssueKind => match self.severity() {
        Severity::Error => Value::error(self.issue_kind()),
        Severity::Ignore => Value::ignore(self.issue_kind()),
        Severity::Warning => Value::warn(self.issue_kind()),
      },
      IssueLabel::SubjectDescription => Value::unreachable(),
      IssueLabel::SubjectKind => Value::unreachable(),
      IssueLabel::Target => Value::target(target_id),
    }
  }
}

const _ALL_ISSUE_LABELS: [IssueLabel; 8] = [
  IssueLabel::Target,
  IssueLabel::IssueKind,
  IssueLabel::IssueDetails,
  IssueLabel::SubjectKind,
  IssueLabel::SubjectDescription,
  IssueLabel::DependencyName,
  IssueLabel::DependencySubject,
  IssueLabel::DependencyValue,
];
