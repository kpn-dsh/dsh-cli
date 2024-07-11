use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

pub mod application;
pub mod processor;
pub mod processor_config;
pub mod processor_descriptor;
pub mod processor_registry;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ProcessorType {
  #[serde(rename = "application")]
  Application,
}

impl Display for ProcessorType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ProcessorType::Application => write!(f, "application"),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum DeploymentParameterType {
  #[serde(rename = "boolean")]
  Boolean,
  #[serde(rename = "free-text")]
  FreeText,
  #[serde(rename = "selection")]
  Selection,
}

impl Display for DeploymentParameterType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      DeploymentParameterType::Boolean => write!(f, "boolean"),
      DeploymentParameterType::FreeText => write!(f, "free-text"),
      DeploymentParameterType::Selection => write!(f, "selection"),
    }
  }
}
