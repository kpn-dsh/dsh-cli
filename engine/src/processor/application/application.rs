#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;

use crate::processor::application::application_config::{ApplicationConfig, ProfileConfig};
use crate::processor::application::converters::into_api_application;
use crate::processor::processor::{Processor, ProcessorDeployParameters, ProcessorIdentifier, ProcessorStatus};
use crate::processor::processor_config::PlaceHolder;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;
use crate::TargetClientFactory;

pub struct ApplicationImpl<'a> {
  processor_identifier: ProcessorIdentifier,
  processor_descriptor: ProcessorDescriptor,
  config: ApplicationConfig,
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> ApplicationImpl<'a> {
  pub fn create(application_config: ApplicationConfig, client_factory: &'a TargetClientFactory) -> Result<Self, String> {
    let template_mapping: HashMap<PlaceHolder, &str> = HashMap::from([(PlaceHolder::TENANT, client_factory.tenant.as_str()), (PlaceHolder::USER, client_factory.user.as_str())]);
    Ok(ApplicationImpl {
      processor_identifier: ProcessorIdentifier { processor_type: ProcessorType::Application, name: application_config.application_name.clone() },
      processor_descriptor: ProcessorDescriptor::from((&application_config, &template_mapping)),
      config: application_config,
      target_client_factory: client_factory,
    })
  }
}

#[async_trait]
impl Processor for ApplicationImpl<'_> {
  async fn deploy(&self, processor_instance_name: &str, parameters: &ProcessorDeployParameters) -> Result<(), String> {
    let profile: ProfileConfig = match parameters.profile_name {
      Some(pn) => match self.config.application.profiles.get(pn) {
        Some(p) => p.clone(),
        None => return Err(format!("profile {} is not defined", pn)),
      },
      None => {
        if self.config.application.profiles.is_empty() {
          return Err("no default profile defined".to_string());
        } else if self.config.application.profiles.len() == 1 {
          self.config.application.profiles.iter().nth(1).map(|p| p.1.clone()).unwrap()
        } else {
          return Err("unable to select profile".to_string());
        }
      }
    };
    let target_client = self.target_client_factory.get().await?;
    let template_mapping: HashMap<PlaceHolder, &str> =
      HashMap::from([(PlaceHolder::INSTANCE, processor_instance_name), (PlaceHolder::TENANT, target_client.tenant.as_str()), (PlaceHolder::USER, target_client.user)]);
    let api_application = into_api_application(&self.config, parameters, &profile, target_client.user.clone(), &template_mapping)?;
    match target_client
      .client
      .application_put_by_tenant_application_by_appid_configuration(target_client.tenant, processor_instance_name, &target_client.token, &api_application)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }

  fn descriptor(&self) -> &ProcessorDescriptor {
    &self.processor_descriptor
  }

  fn identifier(&self) -> &ProcessorIdentifier {
    &self.processor_identifier
  }

  fn name(&self) -> &str {
    &self.processor_identifier.name
  }

  fn processor_type(&self) -> ProcessorType {
    ProcessorType::Application
  }

  async fn start(&self, _processor_name: &str) -> Result<String, String> {
    todo!()
  }

  async fn status(&self, instance_name: &str) -> Result<ProcessorStatus, String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_get_by_tenant_application_by_appid_status(target_client.tenant, instance_name, &target_client.token)
      .await
    {
      Ok(r) => {
        if r.status() == 200 {
          Ok(ProcessorStatus { up: true })
        } else {
          Ok(ProcessorStatus { up: false })
        }
      }
      Err(e) => Err(e.to_string()),
    }
  }

  async fn stop(&self, _processor_name: &str) -> Result<String, String> {
    todo!()
  }

  async fn undeploy(&self, instance_name: &str) -> Result<(), String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, instance_name, &target_client.token)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }
}
