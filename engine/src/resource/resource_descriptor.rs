use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resource::ResourceType;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceDescriptor {
  #[serde(rename = "type")]
  pub resource_type: ResourceType,
  pub id: String,
  pub label: String,
  pub description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  pub writable: bool,
  pub readable: bool,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub metadata: Vec<(String, String)>,
  #[serde(rename = "more-info-url", skip_serializing_if = "Option::is_none")]
  pub more_info_url: Option<String>,
  #[serde(rename = "metrics-url", skip_serializing_if = "Option::is_none")]
  pub metrics_url: Option<String>,
  #[serde(rename = "viewer-url", skip_serializing_if = "Option::is_none")]
  pub viewer_url: Option<String>,
  #[serde(rename = "dsh-topic-descriptor", skip_serializing_if = "Option::is_none")]
  pub dsh_topic_descriptor: Option<DshTopicDescriptor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DshTopicDescriptor {
  pub id: String,
  pub topic: String,
  pub partitions: u32,
  pub replication: u32,
  pub dsh_envelope: bool,
  pub read: String,
  pub write: String,
  pub read_pattern: Option<String>,
  pub write_pattern: Option<String>,
  pub partitioner: String,
  #[serde(rename = "partitioning-depth")]
  pub partitioning_depth: u32,
  #[serde(rename = "can-retain")]
  pub can_retain: bool,
  pub cluster: String,
}

impl Display for ResourceDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(ref version) = self.version {
      write!(f, "{}:{}", self.id, version)?;
    } else {
      write!(f, "{}", self.id)?;
    }
    write!(f, "\n  {}", self.resource_type)?;
    write!(f, "\n  {}", self.description)?;
    if self.writable {
      write!(f, "\n  writable resource")?;
    }
    if self.readable {
      write!(f, "\n  readable resource")?;
    }
    if !&self.metadata.is_empty() {
      write!(f, "\n  metadata")?;
      for (key, value) in &self.metadata {
        write!(f, "\n    {}: {}", key, value)?
      }
    }
    if let Some(ref url) = self.more_info_url {
      write!(f, "\n  more info url: {}", url)?
    }
    if let Some(ref url) = self.metrics_url {
      write!(f, "\n  metrics url: {}", url)?
    }
    if let Some(ref url) = self.viewer_url {
      write!(f, "\n  viewer url: {}", url)?
    }
    Ok(())
  }
}
