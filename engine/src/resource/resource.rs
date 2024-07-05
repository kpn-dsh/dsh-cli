#![allow(clippy::module_inception)]

use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceType};

#[derive(Eq, Hash, PartialEq)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceType,
  pub name: String,
}

pub trait Resource {
  fn descriptor(&self) -> &ResourceDescriptor;

  fn identifier(&self) -> &ResourceIdentifier;

  fn resource_type(&self) -> ResourceType;

  fn status(&self, resource_name: &str) -> Result<String, String>;
}
