use lazy_static::lazy_static;

use crate::engine_target::{EngineTarget, DEFAULT_ENGINE_TARGET};
use crate::pipeline::PipelineName;
use crate::processor::dsh_app::dsh_app_registry::DshAppRealizationRegistry;
use crate::processor::dsh_service::dsh_service_registry::DshServiceRealizationRegistry;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::processor_instance::ProcessorInstance;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorId, ProcessorIdentifier, ProcessorName, ProcessorType};
use crate::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};

lazy_static! {
  pub static ref DEFAULT_PROCESSOR_REGISTRY: ProcessorRegistry<'static> = ProcessorRegistry::default();
}

pub struct ProcessorRegistry<'a> {
  dsh_app_realization_registry: DshAppRealizationRegistry<'a>,
  dsh_service_realization_registry: DshServiceRealizationRegistry<'a>,
  resource_registry: &'a ResourceRegistry<'a>,
  engine_target: &'a EngineTarget<'a>,
}

impl<'a> ProcessorRegistry<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: &'a EngineTarget, resource_registry: &'a ResourceRegistry) -> Result<ProcessorRegistry<'a>, String> {
    Ok(ProcessorRegistry {
      dsh_app_realization_registry: DshAppRealizationRegistry::create(engine_target.dsh_api_client_factory, resource_registry)?,
      dsh_service_realization_registry: DshServiceRealizationRegistry::create(engine_target.dsh_api_client_factory, resource_registry)?,
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

  pub fn processor_realization(&self, processor_type: ProcessorType, processor_id: &ProcessorId) -> Option<&(dyn ProcessorRealization)> {
    match processor_type {
      ProcessorType::DshApp => self.dsh_app_realization_registry.dsh_app_realization_by_id(processor_id),
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_realization_by_id(processor_id),
    }
  }

  pub fn processor_realization_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn ProcessorRealization)> {
    match processor_identifier.processor_type {
      ProcessorType::DshApp => self.dsh_app_realization_registry.dsh_app_realization_by_id(&processor_identifier.id),
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_realization_by_id(&processor_identifier.id),
    }
  }

  pub fn processor_instance(
    &'a self,
    processor_type: ProcessorType,
    processor_id: &ProcessorId,
    pipeline_name: Option<&'a PipelineName>,
    processor_name: &'a ProcessorName,
  ) -> Option<Result<Box<dyn ProcessorInstance + 'a>, String>> {
    self
      .processor_realization(processor_type, processor_id)
      .map(|realization| realization.processor_instance(pipeline_name, processor_name, self.engine_target))
  }

  pub fn processor_instance_by_identifier(
    &'a self,
    processor_identifier: &ProcessorIdentifier,
    pipeline_name: Option<&'a PipelineName>,
    processor_name: &'a ProcessorName,
  ) -> Option<Result<Box<dyn ProcessorInstance + 'a>, String>> {
    self.processor_instance(processor_identifier.processor_type.clone(), &processor_identifier.id, pipeline_name, processor_name)
  }

  pub fn processor_descriptor(&self, processor_type: ProcessorType, processor_id: &ProcessorId) -> Option<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshApp => self
        .dsh_app_realization_registry
        .dsh_app_realization_by_id(processor_id)
        .map(|realization| realization.descriptor()),
      ProcessorType::DshService => self
        .dsh_service_realization_registry
        .dsh_service_realization_by_id(processor_id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_type {
      ProcessorType::DshApp => self
        .dsh_app_realization_registry
        .dsh_app_realization_by_id(&processor_identifier.id)
        .map(|realization| realization.descriptor()),
      ProcessorType::DshService => self
        .dsh_service_realization_registry
        .dsh_service_realization_by_id(&processor_identifier.id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn processor_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dsh_service_realization_registry.dsh_service_descriptors()
  }

  pub fn processor_descriptors_by_type(&self, processor_type: ProcessorType) -> Vec<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshApp => self.dsh_app_realization_registry.dsh_app_descriptors(),
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_descriptors(),
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    [self.dsh_app_realization_registry.dsh_app_identifiers(), self.dsh_service_realization_registry.dsh_service_identifiers()].concat()
  }

  pub fn processor_identifiers_by_type(&self, processor_type: ProcessorType) -> Vec<&ProcessorIdentifier> {
    match processor_type {
      ProcessorType::DshApp => self.dsh_app_realization_registry.dsh_app_identifiers(),
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_identifiers(),
    }
  }
}

impl Default for ProcessorRegistry<'_> {
  fn default() -> Self {
    Self::create(&DEFAULT_ENGINE_TARGET, &DEFAULT_RESOURCE_REGISTRY).expect("unable to create default processor registry")
  }
}
