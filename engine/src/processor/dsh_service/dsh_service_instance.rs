#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;

use trifonius_dsh_api::DshApiClient;
use trifonius_dsh_api::DshApiError;

use crate::engine_target::EngineTarget;
use crate::pipeline::PipelineName;
use crate::processor::dsh_service::dsh_service_realization::DshServiceRealization;
use crate::processor::dsh_service::DshServiceName;
use crate::processor::processor_instance::{ProcessorInstance, ProcessorStatus};
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{JunctionId, ParameterId, ProcessorName, ProfileId};
use crate::resource::resource_descriptor::ResourceDirection;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceType};

pub struct DshServiceInstance<'a> {
  pipeline_name: Option<PipelineName>,
  processor_name: ProcessorName,
  dsh_service_name: DshServiceName,
  processor_realization: &'a DshServiceRealization<'a>,
  engine_target: &'a EngineTarget<'a>,
  resource_registry: &'a ResourceRegistry<'a>,
}

impl<'a> DshServiceInstance<'a> {
  pub fn create(
    pipeline_name: Option<&PipelineName>,
    processor_name: &ProcessorName,
    processor_realization: &'a DshServiceRealization,
    engine_target: &'a EngineTarget<'a>,
    resource_registry: &'a ResourceRegistry,
  ) -> Result<Self, String> {
    Ok(Self {
      pipeline_name: pipeline_name.cloned(),
      processor_name: processor_name.clone(),
      dsh_service_name: DshServiceName::try_from((pipeline_name, processor_name))?,
      processor_realization,
      engine_target,
      resource_registry,
    })
  }
}

#[async_trait]
impl ProcessorInstance for DshServiceInstance<'_> {
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

  fn processor_realization(&self) -> &dyn ProcessorRealization {
    self.processor_realization
  }

  async fn deploy(
    &self,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<(), String> {
    let dsh_application_config = self.processor_realization.dsh_deployment_config(
      self.pipeline_name.as_ref(),
      &self.processor_name,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      self.engine_target.tenant.user().to_string(),
    )?;
    let client: DshApiClient = self.engine_target.dsh_api_client().await?;
    match client.create_application(&self.dsh_service_name, dsh_application_config).await {
      Ok(()) => Ok(()),
      Err(DshApiError::NotFound) => Err(format!("unexpected NotFound response when deploying service {}", &self.dsh_service_name)),
      Err(DshApiError::NotAuthorized) => Err(format!("authorization failure when deploying service {}", &self.dsh_service_name)),
      Err(DshApiError::Unexpected(error)) => Err(format!("unexpected error when deploying service {} ({})", &self.dsh_service_name, error)),
    }
  }

  async fn deploy_dry_run(
    &self,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<String, String> {
    let dsh_application_config = self.processor_realization.dsh_deployment_config(
      self.pipeline_name.as_ref(),
      &self.processor_name,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      self.engine_target.tenant().user().to_string(),
    )?;
    match serde_json::to_string_pretty(&dsh_application_config) {
      Ok(config) => Ok(config),
      Err(_) => Err("unable to serialize configuration".to_string()),
    }
  }

  fn pipeline_name(&self) -> Option<&PipelineName> {
    self.pipeline_name.as_ref()
  }

  fn processor_name(&self) -> &ProcessorName {
    &self.processor_name
  }

  async fn start(&self) -> Result<bool, String> {
    Err("start method not yet implemented".to_string())
  }

  async fn status(&self) -> Result<ProcessorStatus, String> {
    match self
      .engine_target
      .dsh_api_client()
      .await?
      .get_application_allocation_status(&self.dsh_service_name)
      .await
    {
      Ok(status) => {
        if status.provisioned {
          Ok(ProcessorStatus { deployed: true, up: Some(true) })
        } else {
          Ok(ProcessorStatus { deployed: true, up: Some(false) })
        }
      }
      Err(DshApiError::NotFound) => Ok(ProcessorStatus { deployed: false, up: None }),
      Err(DshApiError::NotAuthorized) => Err(format!("authorization failure when requesting status for {} service", &self.dsh_service_name)),
      Err(DshApiError::Unexpected(error)) => Err(format!(
        "unexpected error when requesting status for {} service ({})",
        &self.dsh_service_name, error
      )),
    }
  }

  async fn stop(&self) -> Result<bool, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self) -> Result<bool, String> {
    match self.engine_target.dsh_api_client().await?.delete_application(&self.dsh_service_name).await {
      Ok(()) => Ok(true),
      Err(DshApiError::NotFound) => Ok(false),
      Err(DshApiError::NotAuthorized) => Ok(false),
      Err(DshApiError::Unexpected(message)) => Err(format!("unexpected error when undeploying {} service ({})", &self.dsh_service_name, message)),
    }
  }
}
