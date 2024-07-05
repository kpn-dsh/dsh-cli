use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use dsh_rest_api_client::types::{AllocationStatus as ApiAllocationStatus, Notification as ApiNotification};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CleanupPolicy {
  #[serde(rename = "delete")]
  Delete, // TODO What other values?
}

impl Display for CleanupPolicy {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CleanupPolicy::Delete => write!(f, "Delete"),
    }
  }
}

impl TryFrom<&str> for CleanupPolicy {
  type Error = String;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "delete" | "Delete" => Ok(CleanupPolicy::Delete),
      unrecognized => Err(format!("unrecognized cleanup policy '{}'", unrecognized)),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageTimestampType {
  #[serde(rename = "create-time")]
  CreateTime,
  #[serde(rename = "log-append-ime")]
  LogAppendTime,
}

impl Display for MessageTimestampType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MessageTimestampType::CreateTime => write!(f, "CreateTime"),
      MessageTimestampType::LogAppendTime => write!(f, "LogAppendTime"),
    }
  }
}

impl TryFrom<&str> for MessageTimestampType {
  type Error = String;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "CreateTime" => Ok(MessageTimestampType::CreateTime),
      "LogAppendTime" => Ok(MessageTimestampType::LogAppendTime),
      unrecognized => Err(format!("unrecognized message timestamp type '{}'", unrecognized)),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopicDescriptor {
  pub topic_name: String,
  #[serde(rename = "message-timestamp-type")]
  pub message_timestamp_type: Option<MessageTimestampType>,
  #[serde(rename = "cleanup-policy")]
  pub cleanup_policy: Option<CleanupPolicy>, // delete or
  #[serde(rename = "max-message-bytes")]
  pub max_message_bytes: Option<u64>,
  #[serde(rename = "segment-bytes")]
  pub segment_bytes: Option<u64>,
  #[serde(rename = "number-of-partitions")]
  pub number_of_partitions: u64,
  #[serde(rename = "replication-factor")]
  pub replication_factor: u64,
}

impl Display for TopicDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} (partitions: {}, replication factor: {}",
      self.topic_name, self.number_of_partitions, self.replication_factor
    )?;
    if let Some(ref cleanup_policy) = self.cleanup_policy {
      write!(f, ", cleanup policy: {}", cleanup_policy)?;
    }
    if let Some(ref max_message_bytes) = self.max_message_bytes {
      write!(f, ", max message bytes: {}", max_message_bytes)?;
    }
    if let Some(ref segment_bytes) = self.segment_bytes {
      write!(f, ", segment bytes: {}", segment_bytes)?;
    }
    if let Some(ref message_timestamp_type) = self.message_timestamp_type {
      write!(f, ", message timestamp type: {}", message_timestamp_type)?;
    }
    write!(f, ")")?;
    Ok(())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Notification {
  pub args: HashMap<String, String>,
  pub message: String,
  pub remove: bool,
}

impl From<ApiNotification> for Notification {
  fn from(value: ApiNotification) -> Self {
    Notification { args: value.args, message: value.message, remove: value.remove }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopicStatus {
  pub provisioned: bool,
  #[serde(rename = "derived-from")]
  pub derived_from: Option<String>,
  pub notifications: Option<Vec<Notification>>,
}

impl From<ApiAllocationStatus> for TopicStatus {
  fn from(value: ApiAllocationStatus) -> Self {
    TopicStatus {
      provisioned: value.provisioned,
      derived_from: value.derived_from,
      notifications: if value.notifications.is_empty() { None } else { Some(value.notifications.into_iter().map(Notification::from).collect()) },
    }
  }
}
