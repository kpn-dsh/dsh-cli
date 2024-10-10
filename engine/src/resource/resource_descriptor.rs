use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resource::dshtopic::dshtopic_descriptor::DshTopicDescriptor;
use crate::resource::ResourceTechnology;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceTypeDescriptor {
  #[serde(rename = "type")]
  pub resource_type: ResourceTechnology,
  pub label: String,
  pub description: String,
}

impl From<&ResourceTechnology> for ResourceTypeDescriptor {
  fn from(value: &ResourceTechnology) -> Self {
    ResourceTypeDescriptor { resource_type: value.clone(), label: value.label().to_owned(), description: value.description().to_owned() }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceDescriptor {
  #[serde(rename = "technology")]
  pub technology: ResourceTechnology,
  pub id: String,
  pub label: String,
  pub description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub icon: Option<String>, // TODO Is String the proper type?
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub tags: Vec<String>,
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
  #[serde(rename = "data-catalog-url", skip_serializing_if = "Option::is_none")]
  pub data_catalog_url: Option<String>,
  #[serde(rename = "dshtopic-descriptor", skip_serializing_if = "Option::is_none")]
  pub dshtopic_descriptor: Option<DshTopicDescriptor>,
}

impl Display for ResourceDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{} ({})", self.id, self.technology, self.label)?;
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
    if let Some(ref dshtopic_descriptor) = self.dshtopic_descriptor {
      std::fmt::Display::fmt(&dshtopic_descriptor, f)?
    }
    Ok(())
  }
}
