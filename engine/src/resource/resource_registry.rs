use lazy_static::lazy_static;

use crate::processor::application::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTORY};
use crate::resource::dsh_topic::topic_registry::TopicRegistry;
use crate::resource::resource::{Resource, ResourceIdentifier, ResourceStatus};
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::ResourceType;

lazy_static! {
  pub static ref DEFAULT_RESOURCE_REGISTRY: ResourceRegistry<'static> = ResourceRegistry::default();
}

pub struct ResourceRegistry<'a> {
  dsh_topic_registry: TopicRegistry<'a>,
}

impl Default for ResourceRegistry<'_> {
  fn default() -> Self {
    ResourceRegistry::create(&DEFAULT_TARGET_CLIENT_FACTORY).expect("unable to create default resource registry")
  }
}

impl<'a> ResourceRegistry<'a> {
  pub fn new() -> Self {
    ResourceRegistry::default()
  }

  pub fn create(target_client_factory: &'a TargetClientFactory) -> Result<ResourceRegistry<'a>, String> {
    Ok(ResourceRegistry { dsh_topic_registry: TopicRegistry::create(target_client_factory)? })
  }

  pub fn resource(&self, resource_type: ResourceType, resource_id: &str) -> Option<&(dyn Resource + Sync)> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_by_id(resource_id),
    }
  }

  pub fn resource_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&(dyn Resource + Sync)> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_by_id(resource_identifier.id.as_str()),
    }
  }

  pub fn resource_descriptor(&self, resource_type: ResourceType, resource_id: &str) -> Option<&ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_by_id(resource_id).map(|r| r.descriptor()),
    }
  }

  pub fn resource_descriptor_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&ResourceDescriptor> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_by_id(resource_identifier.id.as_str()).map(|r| r.descriptor()),
    }
  }

  pub fn resource_descriptors(&self) -> Vec<&ResourceDescriptor> {
    self.dsh_topic_registry.resource_descriptors()
  }

  pub fn resource_descriptors_by_type(&self, resource_type: &ResourceType) -> Vec<&ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_descriptors(),
    }
  }

  pub fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.dsh_topic_registry.resource_identifiers()
  }

  pub fn resource_identifiers_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceIdentifier> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_identifiers(),
    }
  }

  pub async fn resource_descriptors_with_status(&self) -> Result<Vec<(&ResourceDescriptor, ResourceStatus)>, String> {
    self.dsh_topic_registry.resource_descriptors_with_status().await
  }

  pub async fn resource_descriptors_by_type_with_status(&self, resource_type: ResourceType) -> Result<Vec<(&ResourceDescriptor, ResourceStatus)>, String> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_registry.resource_descriptors_with_status().await,
    }
  }
}
