use std::collections::HashMap;
use std::fs;

use trifonius_dsh_api::DshApiClientFactory;

use crate::processor::dshapp::dshapp_config::read_dshapp_config;
use crate::processor::dshapp::dshapp_realization::DshAppRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorIdentifier, ProcessorRealizationId, ProcessorType};
use crate::resource::resource_registry::ResourceRegistry;

pub(crate) struct DshAppRealizationRegistry<'a> {
  dshapp_realizations: HashMap<ProcessorIdentifier, DshAppRealization<'a>>,
}

impl<'a> DshAppRealizationRegistry<'a> {
  pub(crate) fn create<'b: 'a>(client_factory: &'a DshApiClientFactory, resource_registry: &'a ResourceRegistry) -> Result<DshAppRealizationRegistry<'a>, String> {
    let mut dshapp_realizations: HashMap<ProcessorIdentifier, DshAppRealization<'a>> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dshapp", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_dshapp_config(&file_name)?;
      let id = ProcessorRealizationId::try_from(config.processor.id.as_str())?;
      let dshapp_realization = DshAppRealization::create(config, client_factory.tenant().clone(), resource_registry)?;
      dshapp_realizations.insert(ProcessorIdentifier { processor_type: ProcessorType::DshApp, id }, dshapp_realization);
    }
    Ok(Self { dshapp_realizations })
  }

  pub(crate) fn dshapp_realization_by_id(&self, id: &ProcessorRealizationId) -> Option<&dyn ProcessorRealization> {
    match self
      .dshapp_realizations
      .get(&ProcessorIdentifier { processor_type: ProcessorType::DshApp, id: id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dshapp_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dshapp_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dshapp_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dshapp_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
