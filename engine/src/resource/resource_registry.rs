use lazy_static::lazy_static;

use crate::engine_target::{EngineTarget, DEFAULT_ENGINE_TARGET};
use crate::pipeline::PipelineName;
use crate::resource::dsh_topic::dsh_topic_registry::DshTopicRealizationRegistry;
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceTypeDescriptor};
use crate::resource::resource_instance::ResourceInstance;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceName, ResourceType};

lazy_static! {
  pub static ref DEFAULT_RESOURCE_REGISTRY: ResourceRegistry<'static> = ResourceRegistry::default();
}

pub struct ResourceRegistry<'a> {
  dsh_topic_realization_registry: DshTopicRealizationRegistry,
  engine_target: &'a EngineTarget<'a>,
}

impl<'a> ResourceRegistry<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: &'a EngineTarget) -> Result<ResourceRegistry<'a>, String> {
    Ok(Self { dsh_topic_realization_registry: DshTopicRealizationRegistry::create(engine_target)?, engine_target })
  }

  pub fn resource_types(&self) -> Vec<ResourceTypeDescriptor> {
    vec![ResourceTypeDescriptor::from(&ResourceType::DshTopic)]
  }

  pub fn resource_realization(&self, resource_type: ResourceType, resource_id: &ResourceId) -> Option<&(dyn ResourceRealization)> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_realization_registry.dsh_topic_realization_by_id(resource_id),
    }
  }

  pub fn resource_realization_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<&(dyn ResourceRealization)> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self.dsh_topic_realization_registry.dsh_topic_realization_by_id(&resource_identifier.id),
    }
  }

  pub fn resource_instance(
    &'a self,
    resource_type: ResourceType,
    resource_id: &ResourceId,
    pipeline_name: Option<&'a PipelineName>,
    resource_name: &'a ResourceName,
  ) -> Option<Result<Box<dyn ResourceInstance + 'a>, String>> {
    self
      .resource_realization(resource_type, resource_id)
      .map(|realization| realization.resource_instance(pipeline_name, resource_name, self.engine_target))
  }

  pub fn resource_instance_by_identifier(
    &'a self,
    resource_identifier: &ResourceIdentifier,
    pipeline_name: Option<&'a PipelineName>,
    resource_name: &'a ResourceName,
  ) -> Option<Result<Box<dyn ResourceInstance + 'a>, String>> {
    self.resource_instance(resource_identifier.resource_type.clone(), &resource_identifier.id, pipeline_name, resource_name)
  }

  pub fn resource_descriptor(&self, resource_type: ResourceType, resource_id: &ResourceId) -> Option<ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self
        .dsh_topic_realization_registry
        .dsh_topic_realization_by_id(resource_id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn resource_descriptor_by_identifier(&self, resource_identifier: &ResourceIdentifier) -> Option<ResourceDescriptor> {
    match resource_identifier.resource_type {
      ResourceType::DshTopic => self
        .dsh_topic_realization_registry
        .dsh_topic_realization_by_id(&resource_identifier.id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn resource_descriptors(&self) -> Vec<ResourceDescriptor> {
    self.dsh_topic_realization_registry.dsh_topic_descriptors()
  }

  pub fn resource_descriptors_by_type(&self, resource_type: &ResourceType) -> Vec<ResourceDescriptor> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_realization_registry.dsh_topic_descriptors(),
    }
  }

  pub fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.dsh_topic_realization_registry.dsh_topic_identifiers()
  }

  pub fn resource_identifiers_by_type(&self, resource_type: ResourceType) -> Vec<&ResourceIdentifier> {
    match resource_type {
      ResourceType::DshTopic => self.dsh_topic_realization_registry.dsh_topic_identifiers(),
    }
  }
}

impl Default for ResourceRegistry<'_> {
  fn default() -> Self {
    Self::create(&DEFAULT_ENGINE_TARGET).expect("unable to create default resource registry")
  }
}
