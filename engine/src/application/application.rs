#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;

use crate::application::application_config::{ApplicationConfig, PlaceHolder, Profile};
use crate::application::application_descriptor::ApplicationDescriptor;
use crate::application::converters::api_application;
use crate::TargetClientFactory;

#[async_trait]
pub trait Application {
  fn descriptor(&self) -> ApplicationDescriptor;
  async fn deploy(&self, application_name: &str, config: &HashMap<String, String>, profile_name: Option<&str>) -> Result<(), String>;

  fn name(&self) -> String;

  async fn status(&self, application_name: &str) -> Result<String, String>;

  async fn undeploy(&self, application_name: &str) -> Result<(), String>;
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

#[async_trait]
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

  async fn deploy(&self, application_name: &str, parameters: &HashMap<String, String>, profile_name: Option<&str>) -> Result<(), String> {
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
    let target_client = self.client_factory.get().await?;
    let template_mapping = HashMap::from([(PlaceHolder::TENANT, target_client.tenant), (PlaceHolder::USER, target_client.user)]);
    let api_application = api_application(&self.config, parameters, &profile, target_client.user.clone(), &template_mapping)?;

    match target_client
      .client
      .application_put_by_tenant_application_by_appid_configuration(target_client.tenant, application_name, &target_client.token, &api_application)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }

  fn name(&self) -> String {
    self.config.name.clone()
  }

  async fn status(&self, application_name: &str) -> Result<String, String> {
    let target_client = self.client_factory.get().await?;
    match target_client
      .client
      .application_get_by_tenant_application_by_appid_status(target_client.tenant, application_name, &target_client.token)
      .await
    {
      Ok(r) => Ok(r.status().to_string()), // TODO Status struct
      Err(e) => Err(e.to_string()),
    }
  }

  async fn undeploy(&self, application_name: &str) -> Result<(), String> {
    let target_client = self.client_factory.get().await?;
    match target_client
      .client
      .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, application_name, &target_client.token)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }
}
