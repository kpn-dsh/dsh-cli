use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{config_dir_name, identifier};

pub mod dsh_service;
pub mod processor;
pub mod processor_config;
pub mod processor_descriptor;
pub mod processor_registry;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ProcessorType {
  #[serde(rename = "dsh-service")]
  DshService,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProcessorIdentifier {
  pub processor_type: ProcessorType,
  pub id: ProcessorId,
}

identifier!(JunctionId, "junction identifier", "^[a-z][a-z0-9_-]{1,50}$");
identifier!(ParameterId, "parameter identifier", "^[a-z][a-z0-9_-]{1,30}$");
identifier!(ProcessorId, "processor identifier", "^[a-z][a-z0-9]{0,19}$");
identifier!(ProfileId, "profile identifier", "^[a-z0-9]{1,20}$");
identifier!(ServiceId, "service identifier", "^[a-z][a-z0-9]{0,19}$");
identifier!(ServiceName, "service name", "^[a-z][a-z0-9]{0,19}-[a-z][a-z0-9]{0,19}$");
identifier!(TaskId, "task identifier", "^.*$");

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

impl Display for ProcessorIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.id, &self.processor_type)
  }
}

pub(crate) fn processor_config_dir_name() -> String {
  format!("{}/processors", config_dir_name())
}
