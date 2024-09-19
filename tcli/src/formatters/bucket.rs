use trifonius_dsh_api::types::{Bucket, BucketStatus};

use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::notifications_to_string;

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum BucketLabel {
  DerivedFrom,
  Encrypted,
  Notifications,
  Provisioned,
  Target,
  Versioned,
}

impl Label for BucketLabel {
  fn label_show(&self) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Encrypted => "encrypted",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => "bucket id",
      Self::Versioned => "versioned",
    }
  }
}

impl SubjectFormatter<BucketLabel> for BucketStatus {
  fn value(&self, column: &BucketLabel, target_id: &str) -> String {
    match column {
      BucketLabel::DerivedFrom => self.status.derived_from.clone().unwrap_or_default(),
      BucketLabel::Encrypted => self.configuration.as_ref().map(|bs| bs.encrypted.to_string()).unwrap_or_default(),
      BucketLabel::Notifications => {
        if self.status.notifications.is_empty() {
          "none".to_string()
        } else {
          notifications_to_string(&self.status.notifications)
        }
      }
      BucketLabel::Provisioned => self.status.provisioned.to_string(),
      BucketLabel::Target => target_id.to_string(),
      BucketLabel::Versioned => self.configuration.as_ref().map(|bs| bs.versioned.to_string()).unwrap_or_default(),
    }
  }

  fn target_label(&self) -> Option<BucketLabel> {
    Some(BucketLabel::Target)
  }
}

impl SubjectFormatter<BucketLabel> for Bucket {
  fn value(&self, column: &BucketLabel, target_id: &str) -> String {
    match column {
      BucketLabel::Encrypted => self.encrypted.to_string(),
      BucketLabel::Target => target_id.to_string(),
      BucketLabel::Versioned => self.versioned.to_string(),
      _ => "".to_string(),
    }
  }

  fn target_label(&self) -> Option<BucketLabel> {
    Some(BucketLabel::Target)
  }
}

pub static BUCKET_STATUS_LABELS: [BucketLabel; 6] =
  [BucketLabel::Target, BucketLabel::Provisioned, BucketLabel::DerivedFrom, BucketLabel::Notifications, BucketLabel::Encrypted, BucketLabel::Versioned];

pub static BUCKET_LABELS: [BucketLabel; 3] = [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned];
