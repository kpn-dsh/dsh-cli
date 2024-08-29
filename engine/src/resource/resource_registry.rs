use lazy_static::lazy_static;

use crate::engine_target::{EngineTarget, DEFAULT_ENGINE_TARGET};
use crate::pipeline::PipelineId;
use crate::resource::dshtopic::dshtopic_registry::DshTopicRealizationRegistry;
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceTypeDescriptor};
use crate::resource::resource_instance::ResourceInstance;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceRealizationId, ResourceType};

lazy_static! {
  pub static ref DEFAULT_RESOURCE_REGISTRY: ResourceRegistry<'static> = ResourceRegistry::default();
}

pub struct ResourceRegistry<'a> {
  dshtopic_realization_registry: DshTopicRealizationRegistry,
  engine_target: &'a EngineTarget<'a>,
}

impl<'a> ResourceRegistry<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: &'a EngineTarget) -> Result<ResourceRegistry<'a>, String> {
    Ok(Self { dshtopic_realization_registry: DshTopicRealizationRegistry::create(engine_target)?, engine_target })
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

  pub fn resource_instance(
    &'a self,
    resource_type: ResourceType,
    resource_realization_id: &ResourceRealizationId,
    pipeline_id: Option<&'a PipelineId>,
    resource_id: &'a ResourceId,
  ) -> Option<Result<Box<dyn ResourceInstance + 'a>, String>> {
    self
      .resource_realization(resource_type, resource_realization_id)
      .map(|realization| realization.resource_instance(pipeline_id, resource_id, self.engine_target))
  }

  pub fn resource_instance_by_identifier(
    &'a self,
    resource_identifier: &ResourceIdentifier,
    pipeline_id: Option<&'a PipelineId>,
    resource_id: &'a ResourceId,
  ) -> Option<Result<Box<dyn ResourceInstance + 'a>, String>> {
    self.resource_instance(resource_identifier.resource_type.clone(), &resource_identifier.id, pipeline_id, resource_id)
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

impl Default for ResourceRegistry<'_> {
  fn default() -> Self {
    Self::create(&DEFAULT_ENGINE_TARGET).expect("unable to create default resource registry")
  }
}
