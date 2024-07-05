use std::collections::HashMap;
use std::fs;
use std::ops::Deref;

use crate::application::application::{Application, ApplicationImpl};
use crate::application::application_descriptor::ApplicationDescriptor;
use crate::TargetClientFactory;

pub struct ApplicationRegistry<'a> {
  applications: HashMap<String, Box<dyn Application + 'a>>,
}

static CONFIG_DIR: &str = "config";

impl<'a> ApplicationRegistry<'a> {
  pub fn create<'b: 'a>(client_factory: &'b TargetClientFactory) -> Result<ApplicationRegistry<'a>, String> {
    let mut applications: HashMap<String, Box<dyn Application>> = HashMap::new();
    let paths = fs::read_dir(format!("{}/applications", CONFIG_DIR)).unwrap();
    for path in paths {
      let file_name = path.unwrap().path().display().to_string();
      let config = crate::application::application_config::read_application_config(&file_name).unwrap();
      let application: Box<dyn Application> = Box::new(ApplicationImpl::create(config, client_factory).unwrap());
      applications.insert(application.name(), application);
    }
    Ok(ApplicationRegistry { applications })
  }

  pub fn application(&self, name: &str) -> Option<&dyn Application> {
    self.applications.get(name).map(|a| a.deref())
  }

  pub fn _application_names(&self) -> Vec<&str> {
    self.applications.iter().map(|a| a.0.as_str()).collect()
  }

  pub fn application_descriptors(&self) -> Vec<ApplicationDescriptor> {
    self.applications.iter().map(|a| a.1.descriptor().clone()).collect::<Vec<ApplicationDescriptor>>()
  }
}
