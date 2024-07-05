#![allow(clippy::module_inception)]

use std::collections::HashMap;

use futures::executor;

use crate::processor::application::application_config::{ApplicationConfig, PlaceHolder, Profile};
use crate::processor::application::application_descriptor::ApplicationDescriptor;
use crate::processor::application::converters::api_application;
use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProcessorType};
use crate::TargetClientFactory;

pub struct ApplicationImpl<'a> {
  application_identifier: ProcessorIdentifier,
  application_descriptor: ProcessorDescriptor,
  config: ApplicationConfig,
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> ApplicationImpl<'a> {
  pub fn create(application_config: ApplicationConfig, client_factory: &'a TargetClientFactory) -> Result<Self, String> {
    let application_descriptor = ApplicationDescriptor {
      application_name: application_config.name.clone(),
      application_description: application_config.description.clone(),
      application_version: application_config.version.clone(),
      grafana_url: application_config.grafana_url.clone(),
      deployment_parameters: application_config.deployment_parameters.clone(),
      deployment_profiles:
      application_config
          .profiles
          .iter()
          .map(|ps| (ps.0.to_string(), ps.1.to_string()))
          .collect::<Vec<(String, String)>>(),
    };
    Ok(ApplicationImpl {
      application_identifier: ProcessorIdentifier { processor_type: ProcessorType::Application, name: application_config.name.clone()},
      application_descriptor: ProcessorDescriptor::from(application_descriptor),
      config: application_config,
      target_client_factory: client_factory
    })
  }
}

impl Processor for ApplicationImpl<'_> {
  fn descriptor(&self) -> &ProcessorDescriptor {
    &self.application_descriptor
  }

  fn deploy(&self, instance_name: &str, parameters: &HashMap<String, String>, profile_name: Option<&str>) -> Result<(), String> {
    let profile: Profile = match profile_name {
      Some(pn) => match self.config.profiles.get(pn) {
        Some(p) => p.clone(),
        None => return Err(format!("profile {} is not defined", pn)),
      },
      None => {
        if self.config.profiles.is_empty() {
          return Err("no default profile defined".to_string());
        } else if self.config.profiles.len() == 1 {
          self.config.profiles.iter().nth(1).map(|p| p.1.clone()).unwrap()
        } else {
          return Err("unable to select profile".to_string());
        }
      }
    };
    let target_client = executor::block_on(self.target_client_factory.get())?;
    let template_mapping = HashMap::from([(PlaceHolder::TENANT, target_client.tenant), (PlaceHolder::USER, target_client.user)]);
    let api_application = api_application(&self.config, parameters, &profile, target_client.user.clone(), &template_mapping)?;

    match executor::block_on(target_client.client.application_put_by_tenant_application_by_appid_configuration(
      target_client.tenant,
      instance_name,
      &target_client.token,
      &api_application,
    )) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }

  fn status(&self, instance_name: &str) -> Result<String, String> {
    let target_client = executor::block_on(self.target_client_factory.get())?;
    match executor::block_on(
      target_client
        .client
        .application_get_by_tenant_application_by_appid_status(target_client.tenant, instance_name, &target_client.token),
    ) {
      Ok(r) => Ok(r.status().to_string()), // TODO Status struct
      Err(e) => Err(e.to_string()),
    }
  }

  fn undeploy(&self, instance_name: &str) -> Result<(), String> {
    let target_client = executor::block_on(self.target_client_factory.get())?;
    match executor::block_on(
      target_client
        .client
        .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, instance_name, &target_client.token),
    ) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }

  fn identifier(&self) -> &ProcessorIdentifier {
    &self.application_identifier
  }

  fn processor_type(&self) -> ProcessorType {
    ProcessorType::Application
  }

  fn name(&self) -> &str {
    &self.application_identifier.name
  }
}
