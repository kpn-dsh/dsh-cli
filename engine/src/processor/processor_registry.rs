use lazy_static::lazy_static;

use crate::processor::dsh_service::dsh_service_registry::DshServiceRegistry;
use crate::processor::dsh_service::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTORY};
use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::ProcessorType;
use crate::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};

lazy_static! {
  pub static ref DEFAULT_PROCESSOR_REGISTRY: ProcessorRegistry<'static> = ProcessorRegistry::default();
}

pub struct ProcessorRegistry<'a> {
  dsh_service_registry: DshServiceRegistry<'a>,
  resource_registry: &'a ResourceRegistry<'a>,
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
    Ok(ProcessorRegistry { dsh_service_registry: DshServiceRegistry::create(target_client_factory, resource_registry)?, resource_registry })
  }

  pub fn resource_registry(&self) -> &ResourceRegistry {
    self.resource_registry
  }

  pub fn processor_types(&self) -> Vec<ProcessorTypeDescriptor> {
    vec![ProcessorTypeDescriptor::from(&ProcessorType::DshService)]
  }

  pub fn processor(&self, processor_type: ProcessorType, processor_id: &str) -> Option<&(dyn Processor)> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_registry.dsh_service_by_id(processor_id),
    }
  }

  pub fn processor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn Processor)> {
    match processor_identifier.processor_type {
      ProcessorType::DshService => self.dsh_service_registry.dsh_service_by_id(processor_identifier.id.as_str()),
    }
  }

  pub fn processor_descriptor(&self, processor_type: ProcessorType, processor_id: &str) -> Option<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_registry.dsh_service_by_id(processor_id).map(|a| a.descriptor()),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_type {
      ProcessorType::DshService => self
        .dsh_service_registry
        .dsh_service_by_id(processor_identifier.id.as_str())
        .map(|a| a.descriptor()),
    }
  }

  pub fn processor_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dsh_service_registry.dsh_service_descriptors()
  }

  pub fn processor_descriptors_by_type(&self, processor_type: ProcessorType) -> Vec<ProcessorDescriptor> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_registry.dsh_service_descriptors(),
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dsh_service_registry.processor_identifiers()
  }

  pub fn processor_identifiers_by_type(&self, processor_type: ProcessorType) -> Vec<&ProcessorIdentifier> {
    match processor_type {
      ProcessorType::DshService => self.dsh_service_registry.processor_identifiers(),
    }
  }
}
