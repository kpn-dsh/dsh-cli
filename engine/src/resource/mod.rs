use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub mod dsh_topic;
pub mod resource;
pub mod resource_descriptor;
pub mod resource_registry;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ResourceType {
  #[serde(rename = "dsh-topic")]
  DshTopic,
  // DshGateway,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceType,
  pub id: ResourceId,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ResourceId(pub String);

impl Display for ResourceType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ResourceType::DshTopic => write!(f, "dsh-topic"),
    }
  }
}

impl ResourceType {
  fn description(&self) -> &str {
    match self {
      ResourceType::DshTopic => "Kafka topic managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      ResourceType::DshTopic => "Dsh Topic",
    }
  }
}

impl Display for ResourceIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.id, &self.resource_type)
  }
}

impl ResourceId {
  pub fn new(id: &str) -> Self {
    if Self::is_valid(id) {
      ResourceId(id.to_string())
    } else {
      panic!("invalid resource id '{}' found", id)
    }
  }

  pub fn is_valid(id: &str) -> bool {
    lazy_static! {
      static ref RESOURCE_ID_REGEX: Regex = Regex::new("^[a-z][a-z0-9_-]{1,50}$").unwrap();
    }
    RESOURCE_ID_REGEX.is_match(id)
  }
}

impl TryFrom<&str> for ResourceId {
  type Error = String;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if Self::is_valid(id) {
      Ok(ResourceId(id.to_string()))
    } else {
      Err(format!("invalid resource id '{}'", id))
    }
  }
}

impl TryFrom<String> for ResourceId {
  type Error = String;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if Self::is_valid(id.as_str()) {
      Ok(ResourceId(id))
    } else {
      Err(format!("invalid resource id '{}'", id))
    }
  }
}

impl Display for ResourceId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
