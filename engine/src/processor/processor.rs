#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use async_trait::async_trait;

use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;

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
  async fn deploy(&self, instance_id: &str, parameters: &ProcessorDeployParameters) -> Result<(), String>;
  fn descriptor(&self) -> &ProcessorDescriptor;
  fn id(&self) -> &str;
  fn identifier(&self) -> &ProcessorIdentifier;
  fn label(&self) -> &str;
  fn processor_type(&self) -> ProcessorType;
  async fn start(&self, instance_id: &str) -> Result<String, String>;
  async fn status(&self, instance_id: &str) -> Result<ProcessorStatus, String>;
  async fn stop(&self, instance_id: &str) -> Result<String, String>;
  async fn undeploy(&self, instance_id: &str) -> Result<(), String>;
}

pub struct ProcessorDeployParameters<'a> {
  pub inbound_junctions: &'a HashMap<String, String>,
  pub outbound_junctions: &'a HashMap<String, String>,
  pub parameters: &'a HashMap<String, String>,
  pub profile_id: Option<&'a str>,
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
