use trifonius_dsh_api::types::{Topic, TopicStatus};

use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::notifications_to_string;

#[derive(Eq, Hash, PartialEq)]
pub enum TopicLabel {
  CleanupPolicy,
  DerivedFrom,
  MaxMessageBytes,
  Notifications,
  Partitions,
  Provisioned,
  ReplicationFactor,
  Target,
  TimestampType,
}

impl Label for TopicLabel {
  fn label_list(&self) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup",
      Self::DerivedFrom => "derived",
      Self::MaxMessageBytes => "max bytes",
      Self::Notifications => "not",
      Self::Partitions => "part",
      Self::Provisioned => "prov",
      Self::ReplicationFactor => "repl",
      Self::Target => "id",
      Self::TimestampType => "ts",
    }
  }

  fn label_show(&self) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup policy",
      Self::DerivedFrom => "derived from",
      Self::MaxMessageBytes => "max message bytes",
      Self::Notifications => "notifications",
      Self::Partitions => "number of partitions",
      Self::Provisioned => "provisioned",
      Self::ReplicationFactor => "replication factor",
      Self::Target => "topic id",
      Self::TimestampType => "timestamp type",
    }
  }
}

impl SubjectFormatter<TopicLabel> for Topic {
  fn value(&self, label: &TopicLabel, target_id: &str) -> String {
    match label {
      TopicLabel::CleanupPolicy => self.kafka_properties.get("cleanup.policy").cloned().unwrap_or_default(),
      TopicLabel::DerivedFrom => "".to_string(),
      TopicLabel::MaxMessageBytes => self.kafka_properties.get("max.message.bytes").cloned().unwrap_or_default(),
      TopicLabel::Notifications => "".to_string(),
      TopicLabel::Partitions => self.partitions.to_string(),
      TopicLabel::Provisioned => "".to_string(),
      TopicLabel::ReplicationFactor => self.replication_factor.to_string(),
      TopicLabel::Target => target_id.to_string(),
      TopicLabel::TimestampType => self.kafka_properties.get("message.timestamp.type").cloned().unwrap_or_default(),
    }
  }

  fn target_label(&self) -> Option<TopicLabel> {
    Some(TopicLabel::Target)
  }
}

impl SubjectFormatter<TopicLabel> for TopicStatus {
  fn value(&self, label: &TopicLabel, target_id: &str) -> String {
    match label {
      TopicLabel::CleanupPolicy => self
        .actual
        .as_ref()
        .and_then(|a| a.kafka_properties.get("cleanup.policy"))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::DerivedFrom => self.status.derived_from.clone().unwrap_or_default(),
      TopicLabel::MaxMessageBytes => self
        .actual
        .as_ref()
        .and_then(|a| a.kafka_properties.get("max.message.bytes"))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::Notifications => notifications_to_string(&self.status.notifications),
      TopicLabel::Partitions => self.actual.as_ref().map(|a| a.partitions.to_string()).unwrap_or_default(),
      TopicLabel::Provisioned => self.status.provisioned.to_string(),
      TopicLabel::ReplicationFactor => self.actual.as_ref().map(|a| a.replication_factor.to_string()).unwrap_or_default(),
      TopicLabel::Target => target_id.to_string(),
      TopicLabel::TimestampType => self
        .actual
        .as_ref()
        .and_then(|a| a.kafka_properties.get("message.timestamp.type"))
        .cloned()
        .unwrap_or_default(),
    }
  }

  fn target_label(&self) -> Option<TopicLabel> {
    Some(TopicLabel::Target)
  }
}

pub static TOPIC_STATUS_LABELS: [TopicLabel; 9] = [
  TopicLabel::Target,
  TopicLabel::Partitions,
  TopicLabel::ReplicationFactor,
  TopicLabel::CleanupPolicy,
  TopicLabel::TimestampType,
  TopicLabel::MaxMessageBytes,
  TopicLabel::DerivedFrom,
  TopicLabel::Notifications,
  TopicLabel::Provisioned,
];

pub static TOPIC_LABELS: [TopicLabel; 9] = [
  TopicLabel::Target,
  TopicLabel::Partitions,
  TopicLabel::ReplicationFactor,
  TopicLabel::CleanupPolicy,
  TopicLabel::TimestampType,
  TopicLabel::MaxMessageBytes,
  TopicLabel::DerivedFrom,
  TopicLabel::Notifications,
  TopicLabel::Provisioned,
];
