use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resource::dsh_topic::dsh_topic_descriptor::DshTopicDescriptor;
use crate::resource::ResourceType;

#[derive(Deserialize, Serialize)]
pub enum ResourceDirection {
  #[serde(rename = "inbound")]
  Inbound,
  #[serde(rename = "outbound")]
  Outbound,
}

impl Display for ResourceDirection {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ResourceDirection::Inbound => write!(f, "inbound"),
      ResourceDirection::Outbound => write!(f, "outbound"),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceTypeDescriptor {
  #[serde(rename = "type")]
  pub resource_type: ResourceType,
  pub label: String,
  pub description: String,
}

impl From<&ResourceType> for ResourceTypeDescriptor {
  fn from(value: &ResourceType) -> Self {
    ResourceTypeDescriptor { resource_type: value.clone(), label: value.label().to_owned(), description: value.description().to_owned() }
  }
}

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

impl Display for ResourceDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{} ({})", self.id, self.resource_type, self.label)?;
    if let Some(ref version) = self.version {
      write!(f, "\n  version: {}", version)?;
    }
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
        write!(f, "\n    {}: {}", key, value)?;
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
    if let Some(ref dsh_topic_descriptor) = self.dsh_topic_descriptor {
      std::fmt::Display::fmt(&dsh_topic_descriptor, f)?
    }
    Ok(())
  }
}
