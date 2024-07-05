use std::collections::HashMap;
use std::fs;

use crate::processor::application::application::ApplicationImpl;
use crate::processor::application::application_config::read_application_config;
use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorType};
use crate::TargetClientFactory;

pub struct ApplicationRegistry<'a> {
  applications: HashMap<ProcessorIdentifier, ApplicationImpl<'a>>,
}

static CONFIG_DIR: &str = "config";

impl<'a> ApplicationRegistry<'a> {
  pub fn create<'b: 'a>(client_factory: &'b TargetClientFactory) -> Result<ApplicationRegistry<'a>, String> {
    let mut applications: HashMap<ProcessorIdentifier, ApplicationImpl<'a>> = HashMap::new();
    let paths = fs::read_dir(format!("{}/processors/applications", CONFIG_DIR)).unwrap();
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = read_application_config(&file_name).unwrap();
      let name = config.name.clone();
      let application = ApplicationImpl::create(config, client_factory).unwrap();
      applications.insert(processor_identifier(name), application);
    }
    Ok(ApplicationRegistry { applications })
  }

  pub fn application_by_name(&self, name: &str) -> Option<&dyn Processor> {
    match self.applications.get(&processor_identifier(name.to_string())) {
      Some(a) => Some(a),
      None => None
    }
  }

  pub fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.applications.iter().map(|(_, app)| app.identifier()).collect()
  }

  pub fn application_descriptors(&self) -> Vec<&ProcessorDescriptor> {
    self.applications.iter().map(|(_, app)| app.descriptor()).collect()
  }
}

fn processor_identifier(name: String) -> ProcessorIdentifier {ProcessorIdentifier { processor_type: ProcessorType::Application, name }}
