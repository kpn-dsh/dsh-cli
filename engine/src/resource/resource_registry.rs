use crate::resource::resource::{Resource, ResourceIdentifier};
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceType};
use crate::resource::topic::topic_registry::TopicRegistry;
use crate::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTOR};

pub struct ResourceRegistry<'a> {
  topic_registry: TopicRegistry<'a>,
}

impl Default for ResourceRegistry<'_> {
  fn default() -> Self {
    let target_client_factory = &DEFAULT_TARGET_CLIENT_FACTOR;
    let topic_registry = TopicRegistry::create(target_client_factory).expect("");
    ResourceRegistry { topic_registry }
  }
}

impl<'a> ResourceRegistry<'a> {
  pub fn create(target_client_factory: &'a TargetClientFactory) -> Result<ResourceRegistry<'a>, String> {
    Ok(ResourceRegistry { topic_registry: TopicRegistry::create(target_client_factory)? })
  }

  pub fn resource(&self, resource_type: ResourceType, resource_name: &str) -> Result<Box<dyn Resource + 'a>, String> {
    match resource_type {
      ResourceType::Topic => self.topic_registry.resource_by_name(resource_name),
    }
  }

  pub fn resource_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Result<Box<dyn Resource + 'a>, String> {
    match resource_identifier.resource_type {
      ResourceType::Topic => self.topic_registry.resource_by_name(resource_identifier.name.as_str()),
    }
  }

  pub fn resource_identifiers(&self) -> Result<Vec<ResourceIdentifier>, String> {
    self.topic_registry.resource_identifiers()
  }

  pub fn resource_identifiers_by_type(&self, resource_type: ResourceType) -> Result<Vec<ResourceIdentifier>, String> {
    match resource_type {
      ResourceType::Topic => self.topic_registry.resource_identifiers(),
    }
  }

  pub fn resource_descriptor(&self, resource_type: ResourceType, resource_name: &str) -> Result<Option<ResourceDescriptor>, String> {
    match resource_type {
      ResourceType::Topic => self.topic_registry.resource_descriptor_by_name(resource_name),
    }
  }

  pub fn resource_descriptor_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Result<Option<ResourceDescriptor>, String> {
    match resource_identifier.resource_type {
      ResourceType::Topic => self.topic_registry.resource_descriptor_by_name(resource_identifier.name.as_str()),
    }
  }

  pub fn resource_descriptors(&self) -> Result<Vec<ResourceDescriptor>, String> {
    self.topic_registry.resource_descriptors()
  }

  pub fn resource_descriptors_by_type(&self, resource_type: ResourceType) -> Result<Vec<ResourceDescriptor>, String> {
    match resource_type {
      ResourceType::Topic => self.topic_registry.resource_descriptors(),
    }
  }

  pub fn resource_descriptors_with_status(&self) -> Result<Vec<(ResourceDescriptor, bool)>, String> {
    self.topic_registry.resource_descriptors_with_status()
  }

  pub fn resource_descriptors_by_type_with_status(&self, resource_type: ResourceType) -> Result<Vec<(ResourceDescriptor, bool)>, String> {
    match resource_type {
      ResourceType::Topic => self.topic_registry.resource_descriptors_with_status(),
    }
  }
}
