use lazy_static::lazy_static;

use crate::engine_target::{EngineTarget, DEFAULT_ENGINE_TARGET};
use crate::pipeline::PipelineId;
use crate::processor::dshapp::dshapp_registry::DshAppRealizationRegistry;
use crate::processor::dshservice::dshservice_registry::DshServiceRealizationRegistry;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::processor_instance::ProcessorInstance;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorId, ProcessorIdentifier, ProcessorRealizationId, ProcessorType};
use crate::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};

lazy_static! {
  pub static ref DEFAULT_PROCESSOR_REGISTRY: ProcessorRegistry<'static> = ProcessorRegistry::default();
}

pub struct ProcessorRegistry<'a> {
  dshapp_realization_registry: DshAppRealizationRegistry<'a>,
  dshservice_realization_registry: DshServiceRealizationRegistry<'a>,
  resource_registry: &'a ResourceRegistry<'a>,
  engine_target: &'a EngineTarget<'a>,
}

impl<'a> ProcessorRegistry<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: &'a EngineTarget, resource_registry: &'a ResourceRegistry) -> Result<ProcessorRegistry<'a>, String> {
    Ok(ProcessorRegistry {
      dshapp_realization_registry: DshAppRealizationRegistry::create(engine_target.dsh_api_client_factory, resource_registry)?,
      dshservice_realization_registry: DshServiceRealizationRegistry::create(engine_target.dsh_api_client_factory, resource_registry)?,
      resource_registry,
      engine_target,
    })
  }

  pub fn resource_registry(&self) -> &ResourceRegistry {
    self.resource_registry
  }

  pub fn processor_types(&self) -> Vec<ProcessorTypeDescriptor> {
    vec![ProcessorTypeDescriptor::from(&ProcessorType::DshApp), ProcessorTypeDescriptor::from(&ProcessorType::DshService)]
  }

  pub fn processor_realization(&self, processor_type: ProcessorType, processor_id: &ProcessorRealizationId) -> Option<&(dyn ProcessorRealization)> {
    match processor_type {
      ProcessorType::DshApp => self.dshapp_realization_registry.dshapp_realization_by_id(processor_id),
      ProcessorType::DshService => self.dshservice_realization_registry.dshservice_realization_by_id(processor_id),
    }
  }

  pub fn processor_realization_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn ProcessorRealization)> {
    match processor_identifier.processor_type {
      ProcessorType::DshApp => self.dshapp_realization_registry.dshapp_realization_by_id(&processor_identifier.id),
      ProcessorType::DshService => self.dshservice_realization_registry.dshservice_realization_by_id(&processor_identifier.id),
    }
  }

  pub fn processor_instance(
    &'a self,
    processor_type: ProcessorType,
    processor_realization_id: &ProcessorRealizationId,
    pipeline_id: Option<&'a PipelineId>,
    processor_id: &'a ProcessorId,
  ) -> Option<Result<Box<dyn ProcessorInstance + 'a>, String>> {
    self
      .processor_realization(processor_type, processor_realization_id)
      .map(|realization| realization.processor_instance(pipeline_id, processor_id, self.engine_target))
  }

  pub fn processor_instance_by_identifier(
    &'a self,
    processor_identifier: &ProcessorIdentifier,
    pipeline_id: Option<&'a PipelineId>,
    processor_id: &'a ProcessorId,
  ) -> Option<Result<Box<dyn ProcessorInstance + 'a>, String>> {
    self.processor_instance(processor_identifier.processor_type.clone(), &processor_identifier.id, pipeline_id, processor_id)
  }

  pub fn processor_descriptor(&self, processor_type: ProcessorType, processor_id: &ProcessorRealizationId) -> Option<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(processor_id)
        .map(|realization| realization.descriptor()),
      ProcessorType::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(processor_id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_type {
      ProcessorType::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(&processor_identifier.id)
        .map(|realization| realization.descriptor()),
      ProcessorType::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(&processor_identifier.id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn processor_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dshservice_realization_registry.dshservice_descriptors()
  }

  pub fn processor_descriptors_by_type(&self, processor_type: ProcessorType) -> Vec<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshApp => self.dshapp_realization_registry.dshapp_descriptors(),
      ProcessorType::DshService => self.dshservice_realization_registry.dshservice_descriptors(),
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    [self.dshapp_realization_registry.dshapp_identifiers(), self.dshservice_realization_registry.dshservice_identifiers()].concat()
  }

  pub fn processor_identifiers_by_type(&self, processor_type: ProcessorType) -> Vec<&ProcessorIdentifier> {
    match processor_type {
      ProcessorType::DshApp => self.dshapp_realization_registry.dshapp_identifiers(),
      ProcessorType::DshService => self.dshservice_realization_registry.dshservice_identifiers(),
    }
  }
}

impl Default for ProcessorRegistry<'_> {
  fn default() -> Self {
    Self::create(&DEFAULT_ENGINE_TARGET, &DEFAULT_RESOURCE_REGISTRY).expect("unable to create default processor registry")
  }
}
