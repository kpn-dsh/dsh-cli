use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::resource::{ResourceRealizationId, ResourceType};
use crate::{config_dir_name, identifier};

pub mod dshapp;
pub mod dshservice;
pub mod processor_config;
pub mod processor_context;
pub mod processor_descriptor;
pub mod processor_instance;
pub mod processor_realization;
pub mod processor_registry;

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum ProcessorTechnology {
  #[serde(rename = "dshapp")]
  DshApp,
  #[serde(rename = "dshservice")]
  DshService,
}

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum JunctionTechnology {
  #[serde(rename = "grpc")]
  Grpc,
  #[serde(rename = "kafka")]
  Kafka,
}

identifier!(
  "trifonius_engine::processor",
  JunctionId,
  "junction id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-junction-id",
  "invalid_junction_id"
);
identifier!(
  "trifonius_engine::processor",
  ParameterId,
  "parameter id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-parameter-id",
  "invalid_parameter_id"
);
identifier!(
  "trifonius_engine::processor",
  ProcessorRealizationId,
  "processor realization id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-processor-realization-id",
  "invalid_processor_realization_id"
);
identifier!(
  "trifonius_engine::processor",
  ProcessorId,
  "processor id",
  "^[a-z][a-z0-9]{0,17}$",
  "validid",
  "invalid-id"
);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProcessorIdentifier {
  pub processor_technology: ProcessorTechnology,
  pub processor_realization_id: ProcessorRealizationId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum JunctionIdentifier {
  Processor(ProcessorTechnology, ProcessorRealizationId, JunctionId),
  Resource(ResourceType, ResourceRealizationId),
}

impl Display for JunctionIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      JunctionIdentifier::Processor(_, realization_id, junction_id) => write!(f, "{}.{}", realization_id, junction_id),
      JunctionIdentifier::Resource(resource_type, realization_id) => write!(f, "{}:{}", realization_id, resource_type),
    }
  }
}

#[derive(Deserialize, Serialize)]
pub enum JunctionDirection {
  #[serde(rename = "inbound")]
  Inbound,
  #[serde(rename = "outbound")]
  Outbound,
}

impl Display for JunctionDirection {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      JunctionDirection::Inbound => write!(f, "inbound"),
      JunctionDirection::Outbound => write!(f, "outbound"),
    }
  }
}

impl Display for JunctionTechnology {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      JunctionTechnology::Grpc => write!(f, "grpc"),
      JunctionTechnology::Kafka => write!(f, "kafka"),
    }
  }
}

impl Display for ProcessorTechnology {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ProcessorTechnology::DshApp => write!(f, "dshapp"),
      ProcessorTechnology::DshService => write!(f, "dshservice"),
    }
  }
}

impl ProcessorTechnology {
  fn description(&self) -> &str {
    match self {
      ProcessorTechnology::DshApp => "DSH App Catalog application, managed by the DSH platform",
      ProcessorTechnology::DshService => "DSH service managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      ProcessorTechnology::DshApp => "DSH App",
      ProcessorTechnology::DshService => "DSH Service",
    }
  }
}

impl ProcessorIdentifier {
  pub fn new(processor_technology: ProcessorTechnology, processor_realization_id: ProcessorRealizationId) -> Self {
    ProcessorIdentifier { processor_technology, processor_realization_id }
  }
}

impl Display for ProcessorIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.processor_technology, self.processor_realization_id)
  }
}

pub(crate) fn processor_config_dir_name() -> String {
  format!("{}/processors", config_dir_name())
}
