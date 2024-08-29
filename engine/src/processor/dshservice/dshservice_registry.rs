use std::collections::HashMap;
use std::fs;

use trifonius_dsh_api::DshApiClientFactory;

use crate::processor::dshservice::dshservice_config::read_dshservice_config;
use crate::processor::dshservice::dshservice_realization::DshServiceRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorIdentifier, ProcessorRealizationId, ProcessorType};
use crate::resource::resource_registry::ResourceRegistry;

pub(crate) struct DshServiceRealizationRegistry<'a> {
  dshservice_realizations: HashMap<ProcessorIdentifier, DshServiceRealization<'a>>,
}

impl<'a> DshServiceRealizationRegistry<'a> {
  pub(crate) fn create<'b: 'a>(client_factory: &'a DshApiClientFactory, resource_registry: &'a ResourceRegistry) -> Result<DshServiceRealizationRegistry<'a>, String> {
    let mut dshservice_realizations: HashMap<ProcessorIdentifier, DshServiceRealization<'a>> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dshservice", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_dshservice_config(&file_name)?;
      let id = ProcessorRealizationId::try_from(config.processor.id.as_str())?;
      let dshservice_realization = DshServiceRealization::create(config, client_factory.tenant().clone(), resource_registry)?;
      dshservice_realizations.insert(ProcessorIdentifier { processor_type: ProcessorType::DshService, id }, dshservice_realization);
    }
    Ok(Self { dshservice_realizations })
  }

  pub(crate) fn dshservice_realization_by_id(&self, id: &ProcessorRealizationId) -> Option<&dyn ProcessorRealization> {
    match self
      .dshservice_realizations
      .get(&ProcessorIdentifier { processor_type: ProcessorType::DshService, id: id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dshservice_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dshservice_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dshservice_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dshservice_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
