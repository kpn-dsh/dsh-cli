use lazy_static::lazy_static;

use trifonius_dsh_api::types::{Topic, TopicStatus};

use crate::formatters::notifications_to_string;

enum TopicLabel {
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

lazy_static! {
  static ref DEFAULT_TOPIC_STATUS_TABLE_COLUMNS: Vec<TopicLabel> = vec![
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
  static ref DEFAULT_TOPIC_TABLE_COLUMNS: Vec<TopicLabel> = vec![
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
  static ref DEFAULT_TOPIC_CONFIGURATION_TABLE_COLUMNS: Vec<TopicLabel> =
    vec![TopicLabel::Target, TopicLabel::Partitions, TopicLabel::ReplicationFactor, TopicLabel::CleanupPolicy, TopicLabel::TimestampType, TopicLabel::MaxMessageBytes,];
  static ref DEFAULT_TOPIC_STATUS_TABLE_ROWS: Vec<TopicLabel> = vec![
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
}

pub(crate) fn _topic_status_to_table(target: &str, target_id: &str, topic: &TopicStatus) -> Vec<Vec<String>> {
  DEFAULT_TOPIC_STATUS_TABLE_ROWS
    .iter()
    .map(|row| vec![row.row_label(target).to_string(), topic_status_value(target_id, topic, row)])
    .collect()
}

pub(crate) fn _topic_status_table_row_labels(target: &str) -> Vec<String> {
  DEFAULT_TOPIC_STATUS_TABLE_ROWS.iter().map(|column| column.row_label(target).to_string()).collect()
}

pub(crate) fn _topic_to_table_row(target_id: &str, topic: &Topic) -> Vec<String> {
  DEFAULT_TOPIC_TABLE_COLUMNS.iter().map(|column| topic_value(target_id, topic, column)).collect()
}

pub(crate) fn topic_configuration_to_table_row(target_id: &str, topic: &Topic) -> Vec<String> {
  DEFAULT_TOPIC_CONFIGURATION_TABLE_COLUMNS
    .iter()
    .map(|column| topic_value(target_id, topic, column))
    .collect()
}

pub(crate) fn topic_status_to_table_row(target_id: &str, topic_status: &TopicStatus) -> Vec<String> {
  DEFAULT_TOPIC_STATUS_TABLE_COLUMNS
    .iter()
    .map(|column| topic_status_value(target_id, topic_status, column))
    .collect()
}

pub(crate) fn topic_status_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_TOPIC_STATUS_TABLE_COLUMNS
    .iter()
    .map(|column| column.column_label(target).to_string())
    .collect()
}

pub(crate) fn _topic_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_TOPIC_TABLE_COLUMNS.iter().map(|column| column.column_label(target).to_string()).collect()
}

pub(crate) fn topic_configuration_table_column_labels(target: &str) -> Vec<String> {
  DEFAULT_TOPIC_CONFIGURATION_TABLE_COLUMNS
    .iter()
    .map(|column| column.column_label(target).to_string())
    .collect()
}

fn topic_status_value(target: &str, topic: &TopicStatus, label: &TopicLabel) -> String {
  match label {
    TopicLabel::CleanupPolicy => topic
      .actual
      .as_ref()
      .and_then(|a| a.kafka_properties.get("cleanup.policy"))
      .cloned()
      .unwrap_or_default(),
    TopicLabel::DerivedFrom => topic.status.derived_from.clone().unwrap_or_default(),
    TopicLabel::MaxMessageBytes => topic
      .actual
      .as_ref()
      .and_then(|a| a.kafka_properties.get("max.message.bytes"))
      .cloned()
      .unwrap_or_default(),
    TopicLabel::Notifications => notifications_to_string(&topic.status.notifications),
    TopicLabel::Partitions => topic.actual.as_ref().map(|a| a.partitions.to_string()).unwrap_or_default(),
    TopicLabel::Provisioned => topic.status.provisioned.to_string(),
    TopicLabel::ReplicationFactor => topic.actual.as_ref().map(|a| a.replication_factor.to_string()).unwrap_or_default(),
    TopicLabel::Target => target.to_string(),
    TopicLabel::TimestampType => topic
      .actual
      .as_ref()
      .and_then(|a| a.kafka_properties.get("message.timestamp.type"))
      .cloned()
      .unwrap_or_default(),
  }
}

fn topic_value(target: &str, topic: &Topic, label: &TopicLabel) -> String {
  match label {
    TopicLabel::CleanupPolicy => topic.kafka_properties.get("cleanup.policy").cloned().unwrap_or_default(),
    TopicLabel::DerivedFrom => "".to_string(),
    TopicLabel::MaxMessageBytes => topic.kafka_properties.get("max.message.bytes").cloned().unwrap_or_default(),
    TopicLabel::Notifications => "".to_string(),
    TopicLabel::Partitions => topic.partitions.to_string(),
    TopicLabel::Provisioned => "".to_string(),
    TopicLabel::ReplicationFactor => topic.replication_factor.to_string(),
    TopicLabel::Target => target.to_string(),
    TopicLabel::TimestampType => topic.kafka_properties.get("message.timestamp.type").cloned().unwrap_or_default(),
  }
}

impl TopicLabel {
  fn row_label<'a>(&'a self, target: &'a str) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup policy",
      Self::DerivedFrom => "derived from",
      Self::MaxMessageBytes => "max message bytes",
      Self::Notifications => "notifications",
      Self::Partitions => "number of partitions",
      Self::Provisioned => "provisioned",
      Self::ReplicationFactor => "replication factor",
      Self::Target => target,
      Self::TimestampType => "timestamp type",
    }
  }

  fn column_label<'a>(&'a self, target: &'a str) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup",
      Self::DerivedFrom => "derived",
      Self::MaxMessageBytes => "max bytes",
      Self::Notifications => "not",
      Self::Partitions => "part",
      Self::Provisioned => "prov",
      Self::ReplicationFactor => "repl",
      Self::Target => target,
      Self::TimestampType => "ts",
    }
  }
}
