use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;

use dsh_api::dsh_api_tenant::DshApiTenant;

use crate::processor::dshapp::dshapp_registry::DshAppRealizationRegistry;
use crate::processor::dshservice::dshservice_registry::DshServiceRealizationRegistry;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorTypeDescriptor};
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};

#[derive(Debug)]
pub struct ProcessorRegistry {
  dshapp_realization_registry: DshAppRealizationRegistry,
  dshservice_realization_registry: DshServiceRealizationRegistry,
}

lazy_static! {
  pub static ref DEFAULT_PROCESSOR_REGISTRY: ProcessorRegistry = ProcessorRegistry::default();
}

impl ProcessorRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn create() -> Result<ProcessorRegistry, String> {
    let dshapp_realization_registry = DshAppRealizationRegistry::create()?;
    let dshservice_realization_registry = DshServiceRealizationRegistry::create()?;
    for dshapp_realization_id in dshapp_realization_registry.processor_realization_ids() {
      if dshservice_realization_registry.processor_realization_by_id(dshapp_realization_id).is_some() {
        return Err(format!("processor realization id '{}' is defined more than once", dshapp_realization_id));
      }
    }
    Ok(ProcessorRegistry { dshapp_realization_registry, dshservice_realization_registry })
  }

  pub fn processor_types(&self) -> Vec<ProcessorTypeDescriptor> {
    vec![ProcessorTypeDescriptor::from(&ProcessorTechnology::DshApp), ProcessorTypeDescriptor::from(&ProcessorTechnology::DshService)]
  }

  pub(crate) fn _processor_realization_ids(&self) -> Vec<&ProcessorRealizationId> {
    let mut ids = [self.dshapp_realization_registry.processor_realization_ids(), self.dshservice_realization_registry.processor_realization_ids()].concat();
    ids.sort();
    ids
  }

  pub fn processor_realization<'a>(&'a self, processor_realization_id: &ProcessorRealizationId) -> Option<&(dyn ProcessorRealization + 'a)> {
    match self.dshapp_realization_registry.processor_realization_by_id(processor_realization_id) {
      Some(realization) => Some(realization),
      None => self.dshservice_realization_registry.processor_realization_by_id(processor_realization_id),
    }
  }

  pub fn processor_realization_by_identifier(&self, processor_identifier: &ProcessorIdentifier) -> Option<&(dyn ProcessorRealization)> {
    match processor_identifier.processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .processor_realization_by_id(&processor_identifier.processor_realization_id),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .processor_realization_by_id(&processor_identifier.processor_realization_id),
    }
  }

  pub fn processor_descriptor(
    &self,
    processor_technology: ProcessorTechnology,
    processor_realization_id: &ProcessorRealizationId,
    dsh_api_tenant: &DshApiTenant,
  ) -> Option<ProcessorDescriptor> {
    match processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .processor_realization_by_id(processor_realization_id)
        .map(|realization| realization.descriptor(dsh_api_tenant)),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .processor_realization_by_id(processor_realization_id)
        .map(|realization| realization.descriptor(dsh_api_tenant)),
    }
  }

  pub fn processor_descriptor_by_identifier(&self, processor_identifier: &ProcessorIdentifier, dsh_api_tenant: &DshApiTenant) -> Option<ProcessorDescriptor> {
    match processor_identifier.processor_technology {
      ProcessorTechnology::DshApp => self
        .dshapp_realization_registry
        .processor_realization_by_id(&processor_identifier.processor_realization_id)
        .map(|realization| realization.descriptor(dsh_api_tenant)),
      ProcessorTechnology::DshService => self
        .dshservice_realization_registry
        .processor_realization_by_id(&processor_identifier.processor_realization_id)
        .map(|realization| realization.descriptor(dsh_api_tenant)),
    }
  }

  pub fn processor_descriptors(&self, dsh_api_tenant: &DshApiTenant) -> Vec<ProcessorDescriptor> {
    self.dshservice_realization_registry.descriptors(dsh_api_tenant)
  }

  pub fn processor_descriptors_by_type(&self, processor_technology: ProcessorTechnology, dsh_api_tenant: &DshApiTenant) -> Vec<ProcessorDescriptor> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.descriptors(dsh_api_tenant),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.descriptors(dsh_api_tenant),
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    [self.dshapp_realization_registry.processor_identifiers(), self.dshservice_realization_registry.processor_identifiers()].concat()
  }

  pub fn processor_identifiers_by_type(&self, processor_technology: ProcessorTechnology) -> Vec<&ProcessorIdentifier> {
    match processor_technology {
      ProcessorTechnology::DshApp => self.dshapp_realization_registry.processor_identifiers(),
      ProcessorTechnology::DshService => self.dshservice_realization_registry.processor_identifiers(),
    }
  }
}

impl Display for ProcessorRegistry {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.dshservice_realization_registry.fmt(f)
  }
}

impl Default for ProcessorRegistry {
  fn default() -> Self {
    match Self::create() {
      Ok(registry) => registry,
      Err(error) => panic!("unable to create default processor registry ({})", error),
    }
  }
}
