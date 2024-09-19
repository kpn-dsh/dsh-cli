use trifonius_dsh_api::types::{Volume, VolumeStatus};

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum VolumeLabel {
  ActualSize,
  ConfigurationSize,
  Size,
  Target,
}

impl Label for VolumeLabel {
  fn label_list(&self) -> &str {
    match self {
      Self::ActualSize => "actual size",
      Self::ConfigurationSize => "conf size",
      Self::Size => "size",
      Self::Target => "id",
    }
  }

  fn label_show(&self) -> &str {
    match self {
      Self::ActualSize => "actual size",
      Self::ConfigurationSize => "configuration size",
      Self::Size => "volume size",
      Self::Target => "volume id",
    }
  }
}

impl SubjectFormatter<VolumeLabel> for Volume {
  fn value(&self, label: &VolumeLabel, target_id: &str) -> String {
    match label {
      VolumeLabel::Target => target_id.to_string(),
      VolumeLabel::Size => self.size_gi_b.to_string(),
      _ => "".to_string(),
    }
  }

  fn target_label(&self) -> Option<VolumeLabel> {
    Some(VolumeLabel::Target)
  }
}

impl SubjectFormatter<VolumeLabel> for VolumeStatus {
  fn value(&self, label: &VolumeLabel, target_id: &str) -> String {
    match label {
      VolumeLabel::ActualSize => self.actual.clone().map(|a| a.size_gi_b.to_string()).unwrap_or("NA".to_string()),
      VolumeLabel::ConfigurationSize => self.configuration.clone().map(|a| a.size_gi_b.to_string()).unwrap_or("NA".to_string()),
      VolumeLabel::Size => self.actual.clone().map(|a| a.size_gi_b.to_string()).unwrap_or("NA".to_string()),
      VolumeLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<VolumeLabel> {
    Some(VolumeLabel::Target)
  }
}

pub static VOLUME_LABELS: [VolumeLabel; 2] = [VolumeLabel::Target, VolumeLabel::Size];

pub static VOLUME_STATUS_LABELS: [VolumeLabel; 4] = [VolumeLabel::Target, VolumeLabel::Size, VolumeLabel::ConfigurationSize, VolumeLabel::ActualSize];
