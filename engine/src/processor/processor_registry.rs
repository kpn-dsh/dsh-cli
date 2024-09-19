use crate::engine_target::EngineTarget;
use crate::processor::dshapp::dshapp_registry::DshAppRealizationRegistry;
use crate::processor::dshservice::dshservice_registry::DshServiceRealizationRegistry;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};

pub struct ProcessorRegistry {
  dshapp_realization_registry: DshAppRealizationRegistry,
  dshservice_realization_registry: DshServiceRealizationRegistry,
}

impl ProcessorRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create() -> Result<ProcessorRegistry, String> {
    Ok(ProcessorRegistry { dshapp_realization_registry: DshAppRealizationRegistry::create()?, dshservice_realization_registry: DshServiceRealizationRegistry::create()? })
  }

  pub fn processor_types(&self) -> Vec<ProcessorTypeDescriptor> {
    vec![ProcessorTypeDescriptor::from(&ProcessorTechnology::DshApp), ProcessorTypeDescriptor::from(&ProcessorTechnology::DshService)]
  }

  pub fn processor_realization<'a>(
    &'a self,
    processor_technology: ProcessorTechnology,
    processor_realization_id: &ProcessorRealizationId,
  ) -> Option<&(dyn ProcessorRealization + 'a)> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.dshapp_realization_by_id(processor_realization_id),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.dshservice_realization_by_id(processor_realization_id),
    }
  }

  pub fn processor_realization_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn ProcessorRealization)> {
    match processor_identifier.processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(&processor_identifier.processor_realization_id),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(&processor_identifier.processor_realization_id),
    }
  }

  pub fn processor_descriptor(
    &self,
    processor_technology: ProcessorTechnology,
    processor_realization_id: &ProcessorRealizationId,
    engine_target: &EngineTarget,
  ) -> Option<ProcessorDescriptor> {
    match processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(processor_realization_id)
        .map(|realization| realization.descriptor(engine_target)),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(processor_realization_id)
        .map(|realization| realization.descriptor(engine_target)),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier, engine_target: &EngineTarget) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .dshapp_realization_by_id(&processor_identifier.processor_realization_id)
        .map(|realization| realization.descriptor(engine_target)),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .dshservice_realization_by_id(&processor_identifier.processor_realization_id)
        .map(|realization| realization.descriptor(engine_target)),
    }
  }

  pub fn processor_descriptors(&self, engine_target: &EngineTarget) -> Vec<ProcessorDescriptor> {
    self.dshservice_realization_registry.dshservice_descriptors(engine_target)
  }

  pub fn processor_descriptors_by_type(&self, processor_technology: ProcessorTechnology, engine_target: &EngineTarget) -> Vec<ProcessorDescriptor> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.dshapp_descriptors(engine_target),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.dshservice_descriptors(engine_target),
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
    match Self::create() {
      Ok(registry) => registry,
      Err(error) => panic!("unable to create processor registry ({})", error),
    }
  }
}
