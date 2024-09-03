use std::sync::Arc;

use crate::engine_target::EngineTarget;
use crate::processor::dshapp::dshapp_registry::DshAppRealizationRegistry;
use crate::processor::dshservice::dshservice_registry::DshServiceRealizationRegistry;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};
use crate::resource::resource_registry::ResourceRegistry;

pub struct ProcessorRegistry {
  dshapp_realization_registry: DshAppRealizationRegistry,
  dshservice_realization_registry: DshServiceRealizationRegistry,
}

impl ProcessorRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create(engine_target: Arc<EngineTarget>, resource_registry: Arc<ResourceRegistry>) -> Result<ProcessorRegistry, String> {
    Ok(ProcessorRegistry {
      dshapp_realization_registry: DshAppRealizationRegistry::create(engine_target.clone(), resource_registry.clone())?,
      dshservice_realization_registry: DshServiceRealizationRegistry::create(engine_target, resource_registry)?,
    })
  }

  pub fn processor_types(&self) -> Vec<ProcessorTypeDescriptor> {
    vec![ProcessorTypeDescriptor::from(&ProcessorTechnology::DshApp), ProcessorTypeDescriptor::from(&ProcessorTechnology::DshService)]
  }

  pub fn processor_realization<'a>(&'a self, processor_technology: ProcessorTechnology, processor_id: &ProcessorRealizationId) -> Option<&(dyn ProcessorRealization + 'a)> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.dshapp_realization_by_id(processor_id),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.dshservice_realization_by_id(processor_id),
    }
  }

  pub fn processor_realization_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn ProcessorRealization)> {
    match processor_identifier.processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.dshapp_realization_by_id(&processor_identifier.id),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.dshservice_realization_by_id(&processor_identifier.id),
    }
  }

  pub fn processor_descriptor(&self, processor_technology: ProcessorTechnology, processor_id: &ProcessorRealizationId) -> Option<ProcessorDescriptor> {
    match processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(processor_id)
        .map(|realization| realization.descriptor()),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(processor_id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(&processor_identifier.id)
        .map(|realization| realization.descriptor()),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(&processor_identifier.id)
        .map(|realization| realization.descriptor()),
    }
  }

  pub fn processor_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dshservice_realization_registry.dshservice_descriptors()
  }

  pub fn processor_descriptors_by_type(&self, processor_technology: ProcessorTechnology) -> Vec<ProcessorDescriptor> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.dshapp_descriptors(),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.dshservice_descriptors(),
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    [self.dshapp_realization_registry.dshapp_identifiers(), self.dshservice_realization_registry.dshservice_identifiers()].concat()
  }

  pub fn processor_identifiers_by_type(&self, processor_technology: ProcessorTechnology) -> Vec<&ProcessorIdentifier> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.dshapp_identifiers(),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.dshservice_identifiers(),
    }
  }
}

impl Default for ProcessorRegistry {
  fn default() -> Self {
    let engine_target = Arc::new(EngineTarget::default());
    match ResourceRegistry::create(engine_target.clone()) {
      Ok(resource_registry) => match Self::create(engine_target, Arc::new(resource_registry)) {
        Ok(registry) => registry,
        Err(error) => panic!("unable to create processor registry ({})", error),
      },
      Err(error) => panic!("unable to create resource registry ({})", error),
    }
  }
}
