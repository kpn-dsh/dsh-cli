#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use async_trait::async_trait;

use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProcessorIdentifier {
  pub processor_type: ProcessorType,
  pub name: String,
}

impl Display for ProcessorIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.name, &self.processor_type)
  }
}

#[async_trait]
pub trait Processor {
  async fn deploy(&self, processor_instance_name: &str, parameters: &ProcessorDeployParameters) -> Result<(), String>;
  fn descriptor(&self) -> &ProcessorDescriptor;
  fn identifier(&self) -> &ProcessorIdentifier;
  fn name(&self) -> &str;
  fn processor_type(&self) -> ProcessorType;
  async fn start(&self, processor_instance_name: &str) -> Result<String, String>;
  async fn status(&self, processor_instance_name: &str) -> Result<ProcessorStatus, String>;
  async fn stop(&self, processor_instance_name: &str) -> Result<String, String>;
  async fn undeploy(&self, processor_instance_name: &str) -> Result<(), String>;
}

pub struct ProcessorDeployParameters<'a> {
  pub inbound_junctions: &'a HashMap<String, String>,
  pub outbound_junctions: &'a HashMap<String, String>,
  pub parameters: &'a HashMap<String, String>,
  pub profile_name: Option<&'a str>,
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
