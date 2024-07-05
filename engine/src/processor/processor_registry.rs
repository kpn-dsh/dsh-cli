use crate::{DEFAULT_TARGET_CLIENT_FACTOR, TargetClientFactory};
use crate::processor::application::application_registry::ApplicationRegistry;
use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorType};

pub struct ProcessorRegistry<'a> {
  application_registry: ApplicationRegistry<'a>,
}

impl Default for ProcessorRegistry<'_> {
  fn default() -> Self {
    let target_client_factory = &DEFAULT_TARGET_CLIENT_FACTOR;
    let application_registry = ApplicationRegistry::create(target_client_factory).expect("unable to create default application registry");
    ProcessorRegistry { application_registry }
  }
}

impl<'a> ProcessorRegistry<'a> {
  pub fn create(target_client_factory: &'a TargetClientFactory) -> Result<ProcessorRegistry<'a>, String> {
    Ok(ProcessorRegistry { application_registry: ApplicationRegistry::create(target_client_factory)? })
  }

  pub fn processor(&self, processor_type: ProcessorType, processor_name: &str) -> Option<&(dyn Processor)> {
    match processor_type {
      ProcessorType::Application => self.application_registry.application_by_name(processor_name),
    }
  }

  pub fn processor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn Processor)> {
    match processor_identifier.processor_type {
      ProcessorType::Application => self.application_registry.application_by_name(processor_identifier.name.as_str()),
    }
  }

  pub fn processor_descriptor(&self, processor_type: ProcessorType, processor_name: &str) -> Option<&ProcessorDescriptor> {
    match processor_type {
      ProcessorType::Application => self.application_registry.application_by_name(processor_name).map(|a| a.descriptor())
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&ProcessorDescriptor> {
    match processor_identifier.processor_type {
      ProcessorType::Application => self.application_registry.application_by_name(processor_identifier.name.as_str()).map(|a| a.descriptor()),
    }
  }

  pub fn processor_descriptors(&self) -> Vec<&ProcessorDescriptor> {
    self.application_registry.application_descriptors()
  }

  pub fn processor_descriptors_by_type(&self, processor_type: ProcessorType) -> Vec<&ProcessorDescriptor> {
    match processor_type {
      ProcessorType::Application => self.application_registry.application_descriptors()
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.application_registry.processor_identifiers()
  }

  pub fn processor_identifiers_by_type(&self, processor_type: ProcessorType) -> Vec<&ProcessorIdentifier> {
    match processor_type {
      ProcessorType::Application => self.application_registry.processor_identifiers(),
    }
  }
}
