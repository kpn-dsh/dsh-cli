use lazy_static::lazy_static;

use trifonius_dsh_api::types::AllocationStatus;

use crate::formatters::notifications_to_string;

enum AllocationStatusLabel {
  DerivedFrom,
  Notifications,
  Provisioned,
  Target,
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

pub(crate) fn allocation_status_to_table_row(target_id: &str, allocation_status: Option<&AllocationStatus>) -> Vec<String> {
  match allocation_status {
    Some(status) => DEFAULT_ALLOCATION_STATUS_TABLE_LABELS
      .iter()
      .map(|column| allocation_status_value(target_id, status, column))
      .collect::<Vec<String>>(),
    None => vec![target_id.to_string(), "".to_string(), "".to_string(), "".to_string()],
  }
}

pub(crate) fn allocation_status_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_ALLOCATION_STATUS_TABLE_LABELS
    .iter()
    .map(|column| column.label(target).to_string())
    .collect()
}

pub(crate) fn _allocation_status_table_row_labels(target: &str) -> Vec<String> {
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
    AllocationStatusLabel::DerivedFrom => allocation_status.derived_from.clone().unwrap_or_default(),
    AllocationStatusLabel::Notifications => {
      if allocation_status.notifications.is_empty() {
        "none".to_string()
      } else {
        notifications_to_string(&allocation_status.notifications)
      }
    }
    AllocationStatusLabel::Provisioned => allocation_status.provisioned.to_string(),
    AllocationStatusLabel::Target => target.to_string(),
  }
}
