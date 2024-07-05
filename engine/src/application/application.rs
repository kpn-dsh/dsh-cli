#![allow(clippy::module_inception)]

use std::collections::HashMap;

use futures::executor;

use crate::application::application_config::{ApplicationConfig, PlaceHolder, Profile};
use crate::application::application_descriptor::ApplicationDescriptor;
use crate::application::converters::api_application;
use crate::TargetClientFactory;

pub trait Application {
  fn descriptor(&self) -> ApplicationDescriptor;
  fn deploy(&self, application_name: &str, config: &HashMap<String, String>, profile_name: Option<&str>) -> Result<(), String>;

  fn name(&self) -> String;

  fn status(&self, application_name: &str) -> Result<String, String>;

  fn undeploy(&self, application_name: &str) -> Result<(), String>;
}

pub struct ApplicationImpl<'a> {
  pub config: ApplicationConfig,
  client_factory: &'a TargetClientFactory,
}

impl<'a> ApplicationImpl<'a> {
  pub fn create(application_config: ApplicationConfig, client_factory: &'a TargetClientFactory) -> Result<Self, String> {
    Ok(ApplicationImpl { config: application_config, client_factory })
  }
}

impl Application for ApplicationImpl<'_> {
  fn descriptor(&self) -> ApplicationDescriptor {
    ApplicationDescriptor {
      application_name: self.config.name.clone(),
      application_description: self.config.description.clone(),
      application_version: self.config.version.clone(),
      grafana_url: self.config.grafana_url.clone(),
      deployment_parameters: self.config.deployment_parameters.clone(),
      deployment_profiles: self
        .config
        .profiles
        .iter()
        .map(|ps| (ps.0.to_string(), ps.1.to_string()))
        .collect::<Vec<(String, String)>>(),
    }
  }

  fn deploy(&self, application_name: &str, parameters: &HashMap<String, String>, profile_name: Option<&str>) -> Result<(), String> {
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
    let target_client = executor::block_on(self.client_factory.get())?;
    let template_mapping = HashMap::from([(PlaceHolder::TENANT, target_client.tenant), (PlaceHolder::USER, target_client.user)]);
    let api_application = api_application(&self.config, parameters, &profile, target_client.user.clone(), &template_mapping)?;

    match executor::block_on(target_client.client.application_put_by_tenant_application_by_appid_configuration(
      target_client.tenant,
      application_name,
      &target_client.token,
      &api_application,
    )) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }

  fn name(&self) -> String {
    self.config.name.clone()
  }

  fn status(&self, application_name: &str) -> Result<String, String> {
    let target_client = executor::block_on(self.client_factory.get())?;
    match executor::block_on(
      target_client
        .client
        .application_get_by_tenant_application_by_appid_status(target_client.tenant, application_name, &target_client.token),
    ) {
      Ok(r) => Ok(r.status().to_string()), // TODO Status struct
      Err(e) => Err(e.to_string()),
    }
  }

  fn undeploy(&self, application_name: &str) -> Result<(), String> {
    let target_client = executor::block_on(self.client_factory.get())?;
    match executor::block_on(
      target_client
        .client
        .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, application_name, &target_client.token),
    ) {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }
}
