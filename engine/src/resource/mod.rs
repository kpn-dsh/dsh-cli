use std::fmt::{Display, Formatter};

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
  "invalid.resource.realization.id",
  /// Uniquely identifies a specific realization of a resource,
  /// based on a specific resource technology,
  /// usually with a configuration file.
  /// `ResourceRealization`s define the resources that are available to Trifonius designers.
);
identifier!(
  "trifonius_engine::resource",
  ResourceId,
  "resource id",
  "^[a-z][a-z0-9]{0,17}$",
  "validid",
  "invalid-id",
  /// Identifies an instance of a resource realization in the scope of a pipeline.
);

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ResourceTechnology {
  #[serde(rename = "dshtopic")]
  DshTopic,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceTechnology,
  pub id: ResourceRealizationId,
}

impl Display for ResourceTechnology {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ResourceTechnology::DshTopic => write!(f, "dshtopic"),
    }
  }
}

impl ResourceTechnology {
  fn description(&self) -> &str {
    match self {
      ResourceTechnology::DshTopic => "Kafka topic managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      ResourceTechnology::DshTopic => "Dsh Topic",
    }
  }
}

impl Display for ResourceIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.id, &self.resource_type)
  }
}
