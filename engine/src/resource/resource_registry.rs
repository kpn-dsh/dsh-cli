use std::sync::Arc;

use crate::engine_target::EngineTarget;
use crate::resource::dshtopic::dshtopic_registry::DshTopicRealizationRegistry;
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceTypeDescriptor};
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceIdentifier, ResourceRealizationId, ResourceType};

pub struct ResourceRegistry {
  dshtopic_realization_registry: DshTopicRealizationRegistry,
}

impl ResourceRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: Arc<EngineTarget>) -> Result<ResourceRegistry, String> {
    Ok(Self { dshtopic_realization_registry: DshTopicRealizationRegistry::create(engine_target)? })
  }

  pub fn resource_types(&self) -> Vec<ResourceTypeDescriptor> {
    vec![ResourceTypeDescriptor::from(&ResourceType::DshTopic)]
  }

  pub fn resource_realization(&self, resource_type: ResourceType, resource_id: &ResourceRealizationId) -> Option<&(dyn ResourceRealization)> {
    match resource_type {
      ResourceType::DshTopic => self.dshtopic_realization_registry.dshtopic_realization_by_id(resource_id),
    }
  }

  pub fn resource_realization_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&(dyn ResourceRealization)> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self.dshtopic_realization_registry.dshtopic_realization_by_id(&resource_identifier.id),
    }
  }

  pub fn resource_descriptor(&self, resource_type: ResourceType, resource_id: &ResourceRealizationId) -> Option<ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self
        .dshtopic_realization_registry
        .dshtopic_realization_by_id(resource_id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn resource_descriptor_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<ResourceDescriptor> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self
        .dshtopic_realization_registry
        .dshtopic_realization_by_id(&resource_identifier.id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn resource_descriptors(&self) -> Vec<ResourceDescriptor> {
    self.dshtopic_realization_registry.dshtopic_descriptors()
  }

  pub fn resource_descriptors_by_type(&self, resource_type: &ResourceType) -> Vec<ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self.dshtopic_realization_registry.dshtopic_descriptors(),
    }
  }

  pub fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.dshtopic_realization_registry.dshtopic_identifiers()
  }

  pub fn resource_identifiers_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceIdentifier> {
    match resource_type {
      ResourceType::DshTopic => self.dshtopic_realization_registry.dshtopic_identifiers(),
    }
  }
}

impl Default for ResourceRegistry {
  fn default() -> Self {
    Self::create(Arc::new(EngineTarget::default())).expect("unable to create default resource registry")
  }
}
