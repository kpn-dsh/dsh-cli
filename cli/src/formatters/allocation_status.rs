use lazy_static::lazy_static;

use trifonius_dsh_api::types::{AllocationStatus, Notification};

enum AllocationStatusLabel {
  Target,
  DerivedFrom,
  Notifications,
  Provisioned,
}

lazy_static! {
  static ref DEFAULT_ALLOCATION_STATUS_TABLE_LABELS: Vec<AllocationStatusLabel> =
    vec![AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::DerivedFrom, AllocationStatusLabel::Notifications,];
}

pub(crate) fn allocation_status_to_table(target: &str, target_id: &str, allocation_status: &AllocationStatus) -> Vec<Vec<String>> {
  DEFAULT_ALLOCATION_STATUS_TABLE_LABELS
    .iter()
    .map(|column| vec![column.label(target).to_string(), allocation_status_value(target_id, allocation_status, column)])
    .collect()
}

pub(crate) fn allocation_status_to_table_row(target_id: &str, allocation_status: &AllocationStatus) -> Vec<String> {
  DEFAULT_ALLOCATION_STATUS_TABLE_LABELS
    .iter()
    .map(|column| allocation_status_value(target_id, allocation_status, column))
    .collect()
}

pub(crate) fn allocation_status_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_ALLOCATION_STATUS_TABLE_LABELS
    .iter()
    .map(|column| column.label(target).to_string())
    .collect()
}

pub(crate) fn allocation_status_table_row_labels(target: &str) -> Vec<String> {
  DEFAULT_ALLOCATION_STATUS_TABLE_LABELS
    .iter()
    .map(|column| column.label(target).to_string())
    .collect()
}

impl AllocationStatusLabel {
  fn label<'a>(&'a self, target: &'a str) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => target,
    }
  }
}

fn allocation_status_value(target: &str, allocation_status: &AllocationStatus, column: &AllocationStatusLabel) -> String {
  match column {
    AllocationStatusLabel::Target => target.to_string(),
    AllocationStatusLabel::DerivedFrom => allocation_status.derived_from.clone().unwrap_or_default(),
    AllocationStatusLabel::Notifications => {
      if allocation_status.notifications.is_empty() {
        "none".to_string()
      } else {
        allocation_status
          .notifications
          .iter()
          .map(notification_to_string)
          .collect::<Vec<String>>()
          .join(", ")
      }
    }
    AllocationStatusLabel::Provisioned => allocation_status.provisioned.to_string(),
  }
}

fn notification_to_string(notification: &Notification) -> String {
  format!(
    "{}, {}, {}",
    if notification.remove { "remove".to_string() } else { "create/update".to_string() },
    notification.message,
    notification
      .args
      .iter()
      .map(|(key, value)| format!("{}:{}", key, value))
      .collect::<Vec<String>>()
      .join(", "),
  )
}
