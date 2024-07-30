#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;
use dsh_rest_api_client::Error::UnexpectedResponse;
use log::{debug, error};
use reqwest::StatusCode;

use crate::processor::dsh_service::dsh_service_realization::DshServiceRealization;
use crate::processor::processor::{Processor, ProcessorStatus};
use crate::processor::{JunctionId, ParameterId, ProfileId, ServiceName};
use crate::resource::resource_descriptor::ResourceDirection;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceType};
use crate::target_client::TargetClientFactory;

// TODO Voeg environment variabelen toe die de processor beschrijven en ook in welke pipeline hij zit

pub struct DshServiceImplementation<'a> {
  processor_realization: &'a DshServiceRealization<'a>,
  target_client_factory: &'a TargetClientFactory,
  resource_registry: &'a ResourceRegistry<'a>,
}

impl<'a> DshServiceImplementation<'a> {
  pub fn create(processor_realization: &'a DshServiceRealization, target_client_factory: &'a TargetClientFactory, resource_registry: &'a ResourceRegistry) -> Result<Self, String> {
    Ok(DshServiceImplementation { processor_realization, target_client_factory, resource_registry })
  }
}

#[async_trait]
impl Processor for DshServiceImplementation<'_> {
  async fn compatible_resources(&self, junction_id: &JunctionId) -> Result<Vec<ResourceIdentifier>, String> {
    if let Some((direction, junction_config)) = self
      .processor_realization
      .processor_config
      .inbound_junctions
      .as_ref()
      .and_then(|m| m.get(junction_id).map(|config| (ResourceDirection::Inbound, config)))
      .or_else(|| {
        self
          .processor_realization
          .processor_config
          .outbound_junctions
          .as_ref()
          .and_then(|m| m.get(junction_id).map(|config| (ResourceDirection::Outbound, config)))
      })
    {
      let mut compatible_resources = Vec::<ResourceIdentifier>::new();
      for allowed_resource_type in &junction_config.allowed_resource_types {
        for resource_descriptor in self.resource_registry.resource_descriptors_by_type(allowed_resource_type) {
          match direction {
            ResourceDirection::Inbound => {
              if resource_descriptor.readable {
                compatible_resources.push(ResourceIdentifier { resource_type: ResourceType::DshTopic, id: ResourceId::try_from(resource_descriptor.id.as_str())? })
              }
            }
            ResourceDirection::Outbound => {
              if resource_descriptor.writable {
                compatible_resources.push(ResourceIdentifier { resource_type: ResourceType::DshTopic, id: ResourceId::try_from(resource_descriptor.id.as_str())? })
              }
            }
          }
        }
      }
      Ok(compatible_resources)
    } else {
      Ok(vec![])
    }
  }

  async fn deploy(
    &self,
    service_name: &ServiceName,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<(), String> {
    let target_client = self.target_client_factory.client().await?;
    let dsh_config = self.processor_realization.dsh_config(
      service_name,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      target_client.user().to_string(),
    )?;
    debug!("dsh configuration file\n{:#?}", &dsh_config);
    match target_client
      .client()
      .application_put_by_tenant_application_by_appid_configuration(target_client.tenant(), service_name, target_client.token(), &dsh_config)
      .await
    {
      Ok(response) => {
        response.status();
        match response.status() {
          StatusCode::ACCEPTED => Ok(()),
          unexpected => {
            error!("unexpected response code {}: {:?}", unexpected, response);
            Ok(())
          }
        }
      }
      Err(UnexpectedResponse(response)) => {
        error!("unexpected response on get status request: {:?}", response);
        Err("unexpected response on status request".to_string())
      }
      Err(error) => {
        error!("unexpected error on get status request: {:?}", error);
        Err("unexpected error on get status request".to_string())
      }
    }
  }

  async fn deploy_dry_run(
    &self,
    service_name: &ServiceName,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<String, String> {
    let target_client = self.target_client_factory.client().await?;
    let dsh_config = self.processor_realization.dsh_config(
      service_name,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      target_client.user().to_string(),
    )?;
    debug!("dsh configuration file\n{:#?}", &dsh_config);
    match serde_json::to_string_pretty(&dsh_config) {
      Ok(config) => Ok(config),
      Err(error) => {
        error!("unable to serialize configuration\n{}\n{:#?}", error, dsh_config);
        Err("unable to serialize configuration".to_string())
      }
    }
  }

  async fn start(&self, _service_name: &ServiceName) -> Result<bool, String> {
    Err("start method not yet implemented".to_string())
  }

  async fn status(&self, service_name: &ServiceName) -> Result<ProcessorStatus, String> {
    let target_client = self.target_client_factory.client().await?;
    match target_client
      .client()
      .application_get_by_tenant_application_by_appid_status(target_client.tenant(), service_name, target_client.token())
      .await
    {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(ProcessorStatus { up: true }),
        _ => Ok(ProcessorStatus { up: false }),
      },
      Err(UnexpectedResponse(response)) => match response.status() {
        StatusCode::NOT_FOUND => Ok(ProcessorStatus { up: false }),
        _ => {
          error!("unexpected response on get status request: {:?}", response);
          Err("unexpected response on status request".to_string())
        }
      },
      Err(error) => {
        error!("unexpected error on get status request: {:?}", error);
        Err("unexpected error on get status request".to_string())
      }
    }
  }

  async fn stop(&self, _service_name: &ServiceName) -> Result<bool, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self, service_name: &ServiceName) -> Result<bool, String> {
    let target_client = self.target_client_factory.client().await?;
    match target_client
      .client()
      .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant(), service_name, target_client.token())
      .await
    {
      Ok(response) => match response.status() {
        StatusCode::ACCEPTED => Ok(true),
        StatusCode::NO_CONTENT => Ok(true),
        StatusCode::OK => Ok(true),
        _ => Ok(false),
      },
      Err(UnexpectedResponse(response)) => match response.status() {
        StatusCode::NOT_FOUND => Ok(false),
        _ => {
          error!("unexpected response on undeploy request: {:?}", response);
          Err("unexpected response on undeploy request".to_string())
        }
      },
      Err(error) => {
        error!("unexpected error on undeploy request: {:?}", error);
        Err("unexpected error on undeploy request".to_string())
      }
    }
  }
}
