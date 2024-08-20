use std::collections::HashMap;
use std::fs;
use trifonius_dsh_api::DshApiClientFactory;

use crate::processor::dsh_app::dsh_app_config::read_dsh_app_config;
use crate::processor::dsh_app::dsh_app_realization::DshAppRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorId, ProcessorIdentifier, ProcessorType};
use crate::resource::resource_registry::ResourceRegistry;

pub(crate) struct DshAppRealizationRegistry<'a> {
  dsh_app_realizations: HashMap<ProcessorIdentifier, DshAppRealization<'a>>,
}

impl<'a> DshAppRealizationRegistry<'a> {
  pub(crate) fn create<'b: 'a>(client_factory: &'a DshApiClientFactory, resource_registry: &'a ResourceRegistry) -> Result<DshAppRealizationRegistry<'a>, String> {
    let mut dsh_app_realizations: HashMap<ProcessorIdentifier, DshAppRealization<'a>> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dsh-apps", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_dsh_app_config(&file_name)?;
      let id = ProcessorId::try_from(config.processor.id.as_str())?;
      let dsh_app_realization = DshAppRealization::create(config, client_factory.tenant().clone(), resource_registry)?;
      dsh_app_realizations.insert(ProcessorIdentifier { processor_type: ProcessorType::DshApp, id }, dsh_app_realization);
    }
    Ok(Self { dsh_app_realizations })
  }

  pub(crate) fn dsh_app_realization_by_id(&self, id: &ProcessorId) -> Option<&dyn ProcessorRealization> {
    match self
      .dsh_app_realizations
      .get(&ProcessorIdentifier { processor_type: ProcessorType::DshApp, id: id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dsh_app_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dsh_app_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dsh_app_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dsh_app_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
