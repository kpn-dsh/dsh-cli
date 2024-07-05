#![allow(clippy::module_inception)]

use std::collections::HashMap;

use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorType};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ProcessorIdentifier {
  pub processor_type: ProcessorType,
  pub name: String,
}

pub trait Processor {
  fn deploy(&self, processor_name: &str, config: &HashMap<String, String>, profile_name: Option<&str>) -> Result<(), String>;

  fn descriptor(&self) -> &ProcessorDescriptor;

  fn identifier(&self) -> &ProcessorIdentifier;

  fn name(&self) -> &str;
  fn processor_type(&self) -> ProcessorType;

  fn status(&self, processor_name: &str) -> Result<String, String>;

  fn undeploy(&self, processor_name: &str) -> Result<(), String>;
}
