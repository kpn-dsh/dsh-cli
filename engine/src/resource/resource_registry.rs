use crate::processor::application::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTOR};
use crate::resource::dsh_topic::topic_registry::TopicRegistry;
use crate::resource::resource::{Resource, ResourceIdentifier, ResourceStatus};
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::ResourceType;

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

  pub fn resource(&self, resource_type: ResourceType, resource_id: &str) -> Option<&(dyn Resource)> {
    match resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_by_id(resource_id),
    }
  }

  pub fn resource_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&(dyn Resource)> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_by_id(resource_identifier.id.as_str()),
    }
  }

  pub fn resource_descriptor(&self, resource_type: ResourceType, resource_id: &str) -> Option<&ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_by_id(resource_id).map(|r| r.descriptor()),
    }
  }

  pub fn resource_descriptor_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&ResourceDescriptor> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_by_id(resource_identifier.id.as_str()).map(|r| r.descriptor()),
    }
  }

  pub fn resource_descriptors(&self) -> Vec<&ResourceDescriptor> {
    self.topic_registry.resource_descriptors()
  }

  pub fn resource_descriptors_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_descriptors(),
    }
  }

  pub fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.topic_registry.resource_identifiers()
  }

  pub fn resource_identifiers_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceIdentifier> {
    match resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_identifiers(),
    }
  }

  pub async fn resource_descriptors_with_status(&self) -> Result<Vec<(&ResourceDescriptor, ResourceStatus)>, String> {
    self.topic_registry.resource_descriptors_with_status().await
  }

  pub async fn resource_descriptors_by_type_with_status(&self, resource_type: ResourceType) -> Result<Vec<(&ResourceDescriptor, ResourceStatus)>, String> {
    match resource_type {
      ResourceType::DshTopic => self.topic_registry.resource_descriptors_with_status().await,
    }
  }
}
