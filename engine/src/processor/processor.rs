#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use async_trait::async_trait;

use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;
use crate::resource::resource::Resource;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProcessorIdentifier {
  pub processor_type: ProcessorType,
  pub id: String,
}

impl Display for ProcessorIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.id, &self.processor_type)
  }
}

#[async_trait]
pub trait Processor {
  async fn deploy(
    &self,
    service_id: &str,
    inbound_junctions: &HashMap<String, &(dyn Resource + Sync)>,
    outbound_junctions: &HashMap<String, &(dyn Resource + Sync)>,
    parameters: &HashMap<String, String>,
    profile_id: Option<&str>,
  ) -> Result<(), String>;
  fn descriptor(&self) -> &ProcessorDescriptor;
  fn id(&self) -> &str;
  fn identifier(&self) -> &ProcessorIdentifier;
  fn label(&self) -> &str;
  fn processor_type(&self) -> ProcessorType;
  async fn start(&self, service_id: &str) -> Result<String, String>;
  async fn status(&self, service_id: &str) -> Result<ProcessorStatus, String>;
  async fn stop(&self, service_id: &str) -> Result<String, String>;
  async fn undeploy(&self, service_id: &str) -> Result<(), String>;
}

#[derive(Debug)]
pub struct ProcessorStatus {
  pub up: bool,
}

impl Display for ProcessorStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.up {
      write!(f, "up")
    } else {
      write!(f, "down")
    }
  }
}
