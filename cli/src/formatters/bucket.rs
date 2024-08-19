use lazy_static::lazy_static;

use trifonius_dsh_api::types::{Bucket, BucketStatus};

use crate::formatters::notifications_to_string;

enum BucketStatusLabel {
  Target,
  DerivedFrom,
  Notifications,
  Provisioned,
  Empty,
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
  static ref EMPTY_BUCKET_STATUS_TABLE_LABELS: Vec<BucketStatusLabel> =
    vec![BucketStatusLabel::Target, BucketStatusLabel::Empty, BucketStatusLabel::Empty, BucketStatusLabel::Empty, BucketStatusLabel::Empty, BucketStatusLabel::Empty];
  static ref DEFAULT_BUCKET_TABLE_LABELS: Vec<BucketStatusLabel> = vec![BucketStatusLabel::Target, BucketStatusLabel::Encrypted, BucketStatusLabel::Versioned];
  static ref EMPTY_BUCKET_TABLE_LABELS: Vec<BucketStatusLabel> = vec![BucketStatusLabel::Target, BucketStatusLabel::Empty, BucketStatusLabel::Empty, BucketStatusLabel::Empty];
}

pub(crate) fn _bucket_status_to_table(target: &str, target_id: &str, bucket_status: &BucketStatus) -> Vec<Vec<String>> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS
    .iter()
    .map(|column| vec![column.label(target).to_string(), bucket_status_value(target_id, bucket_status, column)])
    .collect()
}

pub(crate) fn bucket_status_to_table_row(target_id: &str, bucket_status: Option<&BucketStatus>) -> Vec<String> {
  match bucket_status {
    Some(bs) => DEFAULT_BUCKET_STATUS_TABLE_LABELS
      .iter()
      .map(|column| bucket_status_value(target_id, bs, column))
      .collect(),
    None => vec![target_id.to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()],
  }
}

pub(crate) fn bucket_to_table_row(target_id: &str, bucket: Option<&Bucket>) -> Vec<String> {
  match bucket {
    Some(bucket) => DEFAULT_BUCKET_TABLE_LABELS.iter().map(|column| bucket_value(target_id, bucket, column)).collect(),
    None => vec![target_id.to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()],
  }
}

pub(crate) fn bucket_status_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS.iter().map(|column| column.label(target).to_string()).collect()
}

pub(crate) fn bucket_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_BUCKET_TABLE_LABELS.iter().map(|column| column.label(target).to_string()).collect()
}

pub(crate) fn _bucket_status_table_row_labels(target: &str) -> Vec<String> {
  DEFAULT_BUCKET_STATUS_TABLE_LABELS.iter().map(|column| column.label(target).to_string()).collect()
}

impl BucketStatusLabel {
  fn label<'a>(&'a self, target: &'a str) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Empty => "",
      Self::Encrypted => "encrypted",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => target,
      Self::Versioned => "versioned",
    }
  }
}

fn bucket_status_value(target: &str, bucket_status: &BucketStatus, column: &BucketStatusLabel) -> String {
  match column {
    BucketStatusLabel::DerivedFrom => bucket_status.status.derived_from.clone().unwrap_or_default(),
    BucketStatusLabel::Empty => "".to_string(),
    BucketStatusLabel::Encrypted => bucket_status.configuration.as_ref().map(|bs| bs.encrypted.to_string()).unwrap_or_default(),
    BucketStatusLabel::Notifications => {
      if bucket_status.status.notifications.is_empty() {
        "none".to_string()
      } else {
        notifications_to_string(&bucket_status.status.notifications)
      }
    }
    BucketStatusLabel::Provisioned => bucket_status.status.provisioned.to_string(),
    BucketStatusLabel::Target => target.to_string(),
    BucketStatusLabel::Versioned => bucket_status.configuration.as_ref().map(|bs| bs.versioned.to_string()).unwrap_or_default(),
  }
}

fn bucket_value(target: &str, bucket: &Bucket, column: &BucketStatusLabel) -> String {
  match column {
    BucketStatusLabel::Encrypted => bucket.encrypted.to_string(),
    BucketStatusLabel::Target => target.to_string(),
    BucketStatusLabel::Versioned => bucket.versioned.to_string(),
    _ => "".to_string(),
  }
}
