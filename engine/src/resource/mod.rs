use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::identifier;

pub mod dshtopic;
pub mod resource_descriptor;
pub mod resource_instance;
pub mod resource_realization;
pub mod resource_registry;

identifier!(
  "trifonius_engine::resource",
  ResourceRealizationId,
  "resource realization id",
  "^[a-z][a-z0-9_-]{1,49}$",
  "valid_resource_realization_id",
  "invalid.resource.realization.id"
);
identifier!(
  "trifonius_engine::resource",
  ResourceId,
  "resource id",
  "^[a-z][a-z0-9]{0,17}$",
  "validid",
  "invalid-id"
);

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ResourceType {
  // #[serde(rename = "dshgateway")]
  // DshGateway,
  #[serde(rename = "dshtopic")]
  DshTopic,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceType,
  pub id: ResourceRealizationId,
}

impl Display for ResourceType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      // ResourceType::DshGateway => write!(f, "dshgateway"),
      ResourceType::DshTopic => write!(f, "dshtopic"),
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
