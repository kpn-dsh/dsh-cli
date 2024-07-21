use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

pub mod dsh_service;
pub mod platform;
pub mod processor;
pub mod processor_config;
pub mod processor_descriptor;
pub mod processor_registry;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ProcessorType {
  #[serde(rename = "dsh-service")]
  DshService,
}

impl Display for ProcessorType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ProcessorType::DshService => write!(f, "dsh-service"),
    }
  }
}

impl ProcessorType {
  fn description(&self) -> &str {
    match self {
      ProcessorType::DshService => "DSH service managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      ProcessorType::DshService => "DSH Service",
    }
  }
}
