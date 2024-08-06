use std::collections::HashMap;
use std::fs;

use trifonius_dsh_api::DshApiClientFactory;

use crate::processor::dsh_service::dsh_service_config::read_dsh_service_config;
use crate::processor::dsh_service::dsh_service_realization::DshServiceRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorId, ProcessorIdentifier, ProcessorType};
use crate::resource::resource_registry::ResourceRegistry;

pub(crate) struct DshServiceRealizationRegistry<'a> {
  dsh_service_realizations: HashMap<ProcessorIdentifier, DshServiceRealization<'a>>,
}

impl<'a> DshServiceRealizationRegistry<'a> {
  pub(crate) fn create<'b: 'a>(client_factory: &'a DshApiClientFactory, resource_registry: &'a ResourceRegistry) -> Result<DshServiceRealizationRegistry<'a>, String> {
    let mut dsh_service_realizations: HashMap<ProcessorIdentifier, DshServiceRealization<'a>> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dsh-services", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_dsh_service_config(&file_name)?;
      let id = ProcessorId::try_from(config.processor.id.as_str())?;
      let dsh_service_realization = DshServiceRealization::create(config, client_factory.target_tenant().clone(), resource_registry)?;
      dsh_service_realizations.insert(ProcessorIdentifier { processor_type: ProcessorType::DshService, id }, dsh_service_realization);
    }
    Ok(Self { dsh_service_realizations })
  }

  pub(crate) fn dsh_service_realization_by_id(&self, id: &ProcessorId) -> Option<&dyn ProcessorRealization> {
    match self
      .dsh_service_realizations
      .get(&ProcessorIdentifier { processor_type: ProcessorType::DshService, id: id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dsh_service_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dsh_service_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dsh_service_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dsh_service_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
