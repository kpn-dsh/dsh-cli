use std::collections::HashMap;
use std::fs;

use crate::processor::application::application::ApplicationImpl;
use crate::processor::application::application_config::read_application_config;
use crate::processor::application::{TargetClientFactory, TRIFONIUS_CONFIG_DIR};
use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;

pub struct ApplicationRegistry<'a> {
  applications: HashMap<ProcessorIdentifier, ApplicationImpl<'a>>,
}

static DEFAULT_CONFIG_DIR: &str = "config";

impl<'a> ApplicationRegistry<'a> {
  pub fn create<'b: 'a>(client_factory: &'b TargetClientFactory) -> Result<ApplicationRegistry<'a>, String> {
    let mut applications: HashMap<ProcessorIdentifier, ApplicationImpl<'a>> = HashMap::new();
    let config_dir = std::env::var(TRIFONIUS_CONFIG_DIR).unwrap_or(DEFAULT_CONFIG_DIR.to_string());
    let paths = fs::read_dir(format!("{}/processors/applications", config_dir)).map_err(|e| e.to_string())?;
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_application_config(&file_name)?;
      let id = config.application_id.clone();
      let application = ApplicationImpl::create(config, client_factory)?;
      applications.insert(processor_identifier(id), application);
    }
    Ok(ApplicationRegistry { applications })
  }

  pub fn application_by_id(&self, id: &str) -> Option<&dyn Processor> {
    match self.applications.get(&processor_identifier(id.to_string())) {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.applications.values().map(|app| app.identifier()).collect()
  }

  pub fn application_descriptors(&self) -> Vec<&ProcessorDescriptor> {
    self.applications.values().map(|app| app.descriptor()).collect()
  }
}

fn processor_identifier(id: String) -> ProcessorIdentifier {
  ProcessorIdentifier { processor_type: ProcessorType::Application, id }
}
