#![allow(clippy::module_inception)]

use std::fmt::{Display, Formatter};

use async_trait::async_trait;

use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::ResourceType;

#[derive(Eq, Hash, PartialEq)]
pub struct ResourceIdentifier {
  pub resource_type: ResourceType,
  pub name: String,
}

impl Display for ResourceIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.name, &self.resource_type)
  }
}

#[async_trait]
pub trait Resource {
  fn descriptor(&self) -> &ResourceDescriptor;
  fn identifier(&self) -> &ResourceIdentifier;
  fn name(&self) -> &str;
  fn resource_type(&self) -> ResourceType;
  async fn status(&self) -> Result<ResourceStatus, String>;
}

#[derive(Debug)]
pub struct ResourceStatus {
  pub up: bool,
}

impl Display for ResourceStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.up {
      write!(f, "up")
    } else {
      write!(f, "down")
    }
  }
}
