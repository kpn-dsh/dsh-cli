use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::identifier;

pub mod dsh_topic;
pub mod resource_descriptor;
pub mod resource_instance;
pub mod resource_realization;
pub mod resource_registry;

identifier!(
  "resource",
  ResourceId,
  "resource id",
  "^[a-z][a-z0-9_-]{1,50}$",
  "valid_resource_id",
  "invalid.resource.id"
);
identifier!("resource", ResourceName, "resource name", "^[a-z][a-z0-9]{0,17}$", "validname", "invalid-name");

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ResourceType {
  // #[serde(rename = "dsh-gateway")]
  // DshGateway,
  #[serde(rename = "dsh-topic")]
  DshTopic,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceType,
  pub id: ResourceId,
}

impl Display for ResourceType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      // ResourceType::DshGateway => write!(f, "dsh-gateway"),
      ResourceType::DshTopic => write!(f, "dsh-topic"),
    }
  }
}

impl ResourceType {
  fn description(&self) -> &str {
    match self {
      // ResourceType::DshGateway => "Kafka streams topic connected to the DSH gateway",
      ResourceType::DshTopic => "Kafka topic managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      // ResourceType::DshGateway => "Dsh Gateway",
      ResourceType::DshTopic => "Dsh Topic",
    }
  }
}

impl Display for ResourceIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.id, &self.resource_type)
  }
}
