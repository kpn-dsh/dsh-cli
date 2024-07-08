#![allow(clippy::module_inception)]

use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceType};
use async_trait::async_trait;

#[derive(Eq, Hash, PartialEq)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceType,
  pub name: String,
}

#[async_trait]
pub trait Resource {
  fn descriptor(&self) -> &ResourceDescriptor;

  fn identifier(&self) -> &ResourceIdentifier;

  fn resource_type(&self) -> ResourceType;

  async fn status(&self, resource_name: &str) -> Result<String, String>;
}
