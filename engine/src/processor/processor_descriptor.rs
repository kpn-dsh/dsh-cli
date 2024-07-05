use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::processor::application::application_descriptor::ApplicationDescriptor;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ProcessorType {
  #[serde(rename = "application")]
  Application,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessorDescriptor {
  #[serde(rename = "type")]
  pub processor_type: ProcessorType,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub application: Option<ApplicationDescriptor>,
}

impl Display for ProcessorDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.processor_type {
      ProcessorType::Application => self.clone().application.unwrap_or_else(|| unreachable!()).fmt(f),
    }
  }
}

impl From<ApplicationDescriptor> for ProcessorDescriptor {
  fn from(value: ApplicationDescriptor) -> Self {
    ProcessorDescriptor { processor_type: ProcessorType::Application, application: Some(value) }
  }
}
