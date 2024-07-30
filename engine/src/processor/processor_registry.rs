use lazy_static::lazy_static;

use crate::processor::dsh_service::dsh_service_registry::DshServiceRealizationRegistry;
use crate::processor::processor::Processor;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorId, ProcessorIdentifier, ProcessorType};
use crate::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};
use crate::target_client::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTORY};

lazy_static! {
  pub static ref DEFAULT_PROCESSOR_REGISTRY: ProcessorRegistry<'static> = ProcessorRegistry::default();
}

pub struct ProcessorRegistry<'a> {
  dsh_service_realization_registry: DshServiceRealizationRegistry<'a>,
  resource_registry: &'a ResourceRegistry<'a>,
  target_client_factory: &'a TargetClientFactory,
}

impl Default for ProcessorRegistry<'_> {
  fn default() -> Self {
    ProcessorRegistry::create(&DEFAULT_TARGET_CLIENT_FACTORY, &DEFAULT_RESOURCE_REGISTRY).expect("unable to create default processor registry")
  }
}

impl<'a> ProcessorRegistry<'a> {
  pub fn new() -> Self {
    ProcessorRegistry::default()
  }

  pub fn create(target_client_factory: &'a TargetClientFactory, resource_registry: &'a ResourceRegistry) -> Result<ProcessorRegistry<'a>, String> {
    Ok(ProcessorRegistry {
      dsh_service_realization_registry: DshServiceRealizationRegistry::create(target_client_factory, resource_registry)?,
      resource_registry,
      target_client_factory,
    })
  }

  pub fn resource_registry(&self) -> &ResourceRegistry {
    self.resource_registry
  }

  pub fn processor_types(&self) -> Vec<ProcessorTypeDescriptor> {
    vec![ProcessorTypeDescriptor::from(&ProcessorType::DshService)]
  }

  pub fn processor_realization(&self, processor_type: ProcessorType, processor_id: &ProcessorId) -> Option<&(dyn ProcessorRealization)> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_realization_by_id(processor_id),
    }
  }

  pub fn processor_realization_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn ProcessorRealization)> {
    match processor_identifier.processor_type {
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_realization_by_id(&processor_identifier.id),
    }
  }

  pub fn processor(&'a self, processor_type: ProcessorType, processor_id: &ProcessorId) -> Option<Result<Box<dyn Processor + 'a>, String>> {
    self
      .processor_realization(processor_type, processor_id)
      .map(|realization| realization.processor(Some(self.target_client_factory)))
  }

  pub fn processor_by_identifier(&'a self, processor_identifier: &ProcessorIdentifier) -> Option<Result<Box<dyn Processor + 'a>, String>> {
    self.processor(processor_identifier.processor_type.clone(), &processor_identifier.id)
  }

  pub fn processor_descriptor(&self, processor_type: ProcessorType, processor_id: &ProcessorId) -> Option<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshService => self
        .dsh_service_realization_registry
        .dsh_service_realization_by_id(processor_id)
        .map(|a| a.descriptor()),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_type {
      ProcessorType::DshService => self
        .dsh_service_realization_registry
        .dsh_service_realization_by_id(&processor_identifier.id)
        .map(|a| a.descriptor()),
    }
  }

  pub fn processor_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dsh_service_realization_registry.dsh_service_descriptors()
  }

  pub fn processor_descriptors_by_type(&self, processor_type: ProcessorType) -> Vec<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_realization_registry.dsh_service_descriptors(),
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dsh_service_realization_registry.processor_identifiers()
  }

  pub fn processor_identifiers_by_type(&self, processor_type: ProcessorType) -> Vec<&ProcessorIdentifier> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_realization_registry.processor_identifiers(),
    }
  }
}
