use lazy_static::lazy_static;

use trifonius_dsh_api::types::{BucketStatus, Notification};

enum BucketStatusLabel {
  Target,
  DerivedFrom,
  Notifications,
  Provisioned,
  Encrypted,
  Versioned,
}

lazy_static! {
  static ref DEFAULT_BUCKET_STATUS_TABLE_LABELS: Vec<BucketStatusLabel> = vec![
    BucketStatusLabel::Target,
    BucketStatusLabel::Provisioned,
    BucketStatusLabel::DerivedFrom,
    BucketStatusLabel::Notifications,
    BucketStatusLabel::Encrypted,
    BucketStatusLabel::Versioned
  ];
}

pub(crate) fn bucket_status_to_table(target: &str, target_id: &str, bucket_status: &BucketStatus) -> Vec<Vec<String>> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS
    .iter()
    .map(|column| vec![column.label(target).to_string(), bucket_status_value(target_id, bucket_status, column)])
    .collect()
}

pub(crate) fn bucket_status_to_table_row(target_id: &str, bucket_status: &BucketStatus) -> Vec<String> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS
    .iter()
    .map(|column| bucket_status_value(target_id, bucket_status, column))
    .collect()
}

pub(crate) fn bucket_status_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS.iter().map(|column| column.label(target).to_string()).collect()
}

pub(crate) fn bucket_status_table_row_labels(target: &str) -> Vec<String> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS.iter().map(|column| column.label(target).to_string()).collect()
}

impl BucketStatusLabel {
  fn label<'a>(&'a self, target: &'a str) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => target,
      BucketStatusLabel::Encrypted => "encrypted",
      BucketStatusLabel::Versioned => "versioned",
    }
  }
}

fn bucket_status_value(target: &str, bucket_status: &BucketStatus, column: &BucketStatusLabel) -> String {
  match column {
    BucketStatusLabel::Target => target.to_string(),
    BucketStatusLabel::DerivedFrom => bucket_status.status.derived_from.clone().unwrap_or_default(),
    BucketStatusLabel::Notifications => {
      if bucket_status.status.notifications.is_empty() {
        "none".to_string()
      } else {
        bucket_status
          .status
          .notifications
          .iter()
          .map(notification_to_string)
          .collect::<Vec<String>>()
          .join(", ")
      }
    }
    BucketStatusLabel::Provisioned => bucket_status.status.provisioned.to_string(),
    BucketStatusLabel::Encrypted => bucket_status.configuration.as_ref().map(|bs| bs.encrypted.to_string()).unwrap_or_default(),
    BucketStatusLabel::Versioned => bucket_status.configuration.as_ref().map(|bs| bs.versioned.to_string()).unwrap_or_default(),
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
