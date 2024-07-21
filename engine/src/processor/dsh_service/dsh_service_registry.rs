use crate::config_dir_name;
use std::collections::HashMap;
use std::fs;

use crate::processor::dsh_service::dsh_service::DshService;
use crate::processor::dsh_service::dsh_service_config::read_dsh_service_config;
use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;
use crate::resource::resource_registry::ResourceRegistry;
use crate::target_client::TargetClientFactory;

pub(crate) struct DshServiceRegistry<'a> {
  dsh_services: HashMap<ProcessorIdentifier, DshService<'a>>,
}

impl<'a> DshServiceRegistry<'a> {
  pub(crate) fn create<'b: 'a>(client_factory: &'b TargetClientFactory, resource_registry: &'a ResourceRegistry) -> Result<DshServiceRegistry<'a>, String> {
    let mut dsh_services: HashMap<ProcessorIdentifier, DshService<'a>> = HashMap::new();
    let config_dir = config_dir_name();
    let paths = fs::read_dir(format!("{}/processors/dsh-services", config_dir)).map_err(|e| e.to_string())?;
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_dsh_service_config(&file_name)?;
      let id = config.id.clone();
      let dsh_service = DshService::create(config, client_factory, resource_registry)?;
      dsh_services.insert(processor_identifier(id), dsh_service);
    }
    Ok(DshServiceRegistry { dsh_services })
  }

  pub(crate) fn dsh_service_by_id(&self, id: &str) -> Option<&dyn Processor> {
    match self.dsh_services.get(&processor_identifier(id.to_string())) {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dsh_services.values().map(|app| app.identifier()).collect()
  }

  pub(crate) fn dsh_service_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dsh_services.values().map(|app| app.descriptor()).collect()
  }
}

fn processor_identifier(id: String) -> ProcessorIdentifier {
  ProcessorIdentifier { processor_type: ProcessorType::DshService, id }
}
