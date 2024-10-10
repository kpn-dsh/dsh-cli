use std::sync::Arc;

use lazy_static::lazy_static;

use crate::engine_target::EngineTarget;
use crate::resource::dshtopic::dshtopic_registry::DshTopicRealizationRegistry;
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceTypeDescriptor};
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceIdentifier, ResourceRealizationId, ResourceTechnology};

pub struct ResourceRegistry {
  dshtopic_realization_registry: DshTopicRealizationRegistry,
}

lazy_static! {
  pub static ref DEFAULT_RESOURCE_REGISTRY: ResourceRegistry = ResourceRegistry::default();
}

impl ResourceRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: Arc<EngineTarget>) -> Result<ResourceRegistry, String> {
    Ok(Self { dshtopic_realization_registry: DshTopicRealizationRegistry::create(engine_target)? })
  }

  pub fn resource_types(&self) -> Vec<ResourceTypeDescriptor> {
    vec![ResourceTypeDescriptor::from(&ResourceTechnology::DshTopic)]
  }

  pub fn resource_realization(&self, resource_realization_id: &ResourceRealizationId) -> Option<&(dyn ResourceRealization)> {
    self.dshtopic_realization_registry.dshtopic_realization_by_id(resource_realization_id)
  }

  pub fn resource_realization_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&(dyn ResourceRealization)> {
    match resource_identifier.resource_type {
      ResourceTechnology::DshTopic => self.dshtopic_realization_registry.dshtopic_realization_by_id(&resource_identifier.id),
    }
  }

  pub fn resource_descriptor(&self, resource_type: ResourceTechnology, resource_id: &ResourceRealizationId) -> Option<ResourceDescriptor> {
    match resource_type {
      ResourceTechnology::DshTopic => self
        .dshtopic_realization_registry
        .dshtopic_realization_by_id(resource_id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn resource_descriptor_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<ResourceDescriptor> {
    match resource_identifier.resource_type {
      ResourceTechnology::DshTopic => self
        .dshtopic_realization_registry
        .dshtopic_realization_by_id(&resource_identifier.id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn resource_descriptors(&self) -> Vec<ResourceDescriptor> {
    self.dshtopic_realization_registry.dshtopic_descriptors()
  }

  pub fn resource_descriptors_by_type(&self, resource_type: &ResourceTechnology) -> Vec<ResourceDescriptor> {
    match resource_type {
      ResourceTechnology::DshTopic => self.dshtopic_realization_registry.dshtopic_descriptors(),
    }
  }

  pub fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.dshtopic_realization_registry.dshtopic_identifiers()
  }

  pub fn resource_identifiers_by_type(&self, resource_type: ResourceTechnology) -> Vec<&ResourceIdentifier> {
    match resource_type {
      ResourceTechnology::DshTopic => self.dshtopic_realization_registry.dshtopic_identifiers(),
    }
  }
}

impl Default for ResourceRegistry {
  fn default() -> Self {
    Self::create(Arc::new(EngineTarget::default())).expect("unable to create default resource registry")
  }
}
