use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{config_dir_name, identifier};

pub mod dshapp;
pub mod dshservice;
pub mod processor_config;
pub mod processor_descriptor;
pub mod processor_instance;
pub mod processor_realization;
pub mod processor_registry;

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum ProcessorType {
  #[serde(rename = "dshapp")]
  DshApp,
  #[serde(rename = "dshservice")]
  DshService,
}

identifier!(
  "processor",
  JunctionId,
  "junction id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-junction-id",
  "invalid_junction_id"
);
identifier!(
  "processor",
  ParameterId,
  "parameter id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-parameter-id",
  "invalid_parameter_id"
);
identifier!(
  "processor",
  ProcessorRealizationId,
  "processor realization id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-processor-realization-id",
  "invalid_processor_realization_id"
);
identifier!("processor", ProcessorId, "processor id", "^[a-z][a-z0-9]{0,17}$", "validid", "invalid-id");
identifier!(
  "processor",
  ProcessorProfileId,
  "processor profile id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-processor-profile-id",
  "invalid_processor_profile_id"
);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProcessorIdentifier {
  pub processor_type: ProcessorType,
  pub id: ProcessorRealizationId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JunctionIdentifier {
  pub processor_identifier: ProcessorIdentifier,
  pub junction_id: JunctionId,
}

impl Display for ProcessorType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ProcessorType::DshApp => write!(f, "dshapp"),
      ProcessorType::DshService => write!(f, "dshservice"),
    }
  }
}

impl ProcessorType {
  fn description(&self) -> &str {
    match self {
      ProcessorType::DshApp => "DSH App Catalog application, managed by the DSH platform",
      ProcessorType::DshService => "DSH service managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      ProcessorType::DshApp => "DSH App",
      ProcessorType::DshService => "DSH Service",
    }
  }
}

impl ProcessorIdentifier {
  pub fn new(processor_type: ProcessorType, id: ProcessorRealizationId) -> Self {
    ProcessorIdentifier { processor_type, id }
  }
}

impl Display for ProcessorIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.processor_type, self.id)
  }
}

pub(crate) fn processor_config_dir_name() -> String {
  format!("{}/processors", config_dir_name())
}
