#![allow(clippy::module_inception)]

use async_trait::async_trait;
use dsh_rest_api_client::Error::UnexpectedResponse;
use log::error;

use crate::processor::application::application_config::{ApplicationConfig, ProfileConfig};
use crate::processor::application::converters::{into_api_application, TemplateMapping};
use crate::processor::application::TargetClientFactory;
use crate::processor::processor::{Processor, ProcessorDeployParameters, ProcessorIdentifier, ProcessorStatus};
use crate::processor::processor_config::PlaceHolder;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::ProcessorType;

pub struct ApplicationImpl<'a> {
  processor_identifier: ProcessorIdentifier,
  processor_descriptor: ProcessorDescriptor,
  config: ApplicationConfig,
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> ApplicationImpl<'a> {
  pub fn create(application_config: ApplicationConfig, client_factory: &'a TargetClientFactory) -> Result<Self, String> {
    let template_mapping = TemplateMapping::from(client_factory);
    Ok(ApplicationImpl {
      processor_identifier: ProcessorIdentifier { processor_type: ProcessorType::Application, id: application_config.application_id.clone() },
      processor_descriptor: ProcessorDescriptor::from((&application_config, &template_mapping)),
      config: application_config,
      target_client_factory: client_factory,
    })
  }
}

#[async_trait]
impl Processor for ApplicationImpl<'_> {
  async fn deploy(&self, service_id: &str, parameters: &ProcessorDeployParameters) -> Result<(), String> {
    let profile: ProfileConfig = match parameters.profile_id {
      Some(pn) => match self.config.application.profiles.iter().find(|p| p.id == pn) {
        Some(p) => p.clone(),
        None => return Err(format!("profile {} is not defined", pn)),
      },
      None => {
        if self.config.application.profiles.is_empty() {
          return Err("no default profile defined".to_string());
        } else if self.config.application.profiles.len() == 1 {
          self.config.application.profiles.get(1).cloned().unwrap()
        } else {
          return Err("unable to select profile".to_string());
        }
      }
    };
    let target_client = self.target_client_factory.get().await?;
    let mut template_mapping: TemplateMapping = TemplateMapping::from(self.target_client_factory);
    template_mapping.insert(PlaceHolder::ServiceId, service_id.to_string());
    let api_application = into_api_application(&self.config, parameters, &profile, target_client.user.clone(), &template_mapping)?;
    match target_client
      .client
      .application_put_by_tenant_application_by_appid_configuration(target_client.tenant, service_id, &target_client.token, &api_application)
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

  fn id(&self) -> &str {
    &self.processor_identifier.id
  }

  fn label(&self) -> &str {
    &self.processor_descriptor.label
  }

  fn processor_type(&self) -> ProcessorType {
    ProcessorType::Application
  }

  async fn start(&self, _service_id: &str) -> Result<String, String> {
    Err("start method not yet implemented".to_string())
  }

  async fn status(&self, service_id: &str) -> Result<ProcessorStatus, String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_get_by_tenant_application_by_appid_status(target_client.tenant, service_id, &target_client.token)
      .await
    {
      Ok(r) => {
        if r.status() == 200 {
          Ok(ProcessorStatus { up: true })
        } else {
          Ok(ProcessorStatus { up: false })
        }
      }
      Err(UnexpectedResponse(e)) => {
        if e.status() == 404 {
          Ok(ProcessorStatus { up: false })
        } else {
          error!("unexpected response on get status request: {:?}", e);
          Err("unexpected response on get status request".to_string())
        }
      }
      Err(e) => {
        error!("unexpected error on get status request: {:?}", e);
        Err("unexpected error on get status request".to_string())
      }
    }
  }

  async fn stop(&self, _service_id: &str) -> Result<String, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self, service_id: &str) -> Result<(), String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, service_id, &target_client.token)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => Err(e.to_string()),
    }
  }
}
