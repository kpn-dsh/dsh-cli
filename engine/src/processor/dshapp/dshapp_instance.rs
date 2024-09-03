#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;
use log::{debug, error};

use crate::pipeline::PipelineId;
use crate::processor::dshapp::dshapp_realization::DshAppRealization;
use crate::processor::dshapp::DshAppName;
use crate::processor::processor_instance::{ProcessorInstance, ProcessorStatus};
use crate::processor::{JunctionId, ParameterId, ProcessorId};
use crate::resource::resource_descriptor::ResourceDirection;
use crate::resource::{ResourceIdentifier, ResourceRealizationId, ResourceType};
use crate::ProfileId;

// TODO Voeg environment variabelen toe die de processor beschrijven en ook in welke pipeline hij zit

pub struct DshAppInstance<'a> {
  pipeline_id: Option<PipelineId>,
  processor_id: ProcessorId,
  dshapp_name: DshAppName,
  processor_realization: &'a DshAppRealization,
}

impl<'a> DshAppInstance<'a> {
  pub fn create(pipeline_id: Option<PipelineId>, processor_id: ProcessorId, processor_realization: &'a DshAppRealization) -> Result<Self, String> {
    Ok(Self {
      pipeline_id: pipeline_id.clone(),
      processor_id: processor_id.clone(),
      dshapp_name: DshAppName::try_from((pipeline_id.as_ref(), &processor_id))?,
      processor_realization,
    })
  }
}

#[async_trait]
impl ProcessorInstance for DshAppInstance<'_> {
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
        for resource_descriptor in self.processor_realization.resource_registry.resource_descriptors_by_type(allowed_resource_type) {
          match direction {
            ResourceDirection::Inbound => {
              if resource_descriptor.readable {
                compatible_resources.push(ResourceIdentifier { resource_type: ResourceType::DshTopic, id: ResourceRealizationId::try_from(resource_descriptor.id.as_str())? })
              }
            }
            ResourceDirection::Outbound => {
              if resource_descriptor.writable {
                compatible_resources.push(ResourceIdentifier { resource_type: ResourceType::DshTopic, id: ResourceRealizationId::try_from(resource_descriptor.id.as_str())? })
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
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<(), String> {
    let dsh_deployment_config = self.processor_realization.dsh_deployment_config(
      self.pipeline_id.as_ref(),
      &self.processor_id,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      self.processor_realization.engine_target.tenant().user().to_string(),
    )?;
    debug!("dsh configuration file\n{:#?}", &dsh_deployment_config);
    // match &self.api_client.de target_client
    //   .client()
    //   .application_put_by_tenant_application_by_appid_configuration(target_client.tenant(), &self.dshapp_name, target_client.token(), &dsh_deployment_config)
    //   .await
    // {
    //   Ok(response) => {
    //     response.status();
    //     match response.status() {
    //       StatusCode::ACCEPTED => Ok(()),
    //       unexpected => {
    //         error!("unexpected response code {}: {:?}", unexpected, response);
    //         Ok(())
    //       }
    //     }
    //   }
    //   Err(UnexpectedResponse(response)) => {
    //     error!("unexpected response on get status request: {:?}", response);
    //     Err("unexpected response on status request".to_string())
    //   }
    //   Err(error) => {
    //     error!("unexpected error on get status request: {:?}", error);
    //     Err("unexpected error on get status request".to_string())
    //   }
    // }
    Ok(())
  }

  async fn deploy_dry_run(
    &self,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<String, String> {
    let dsh_config = self.processor_realization.dsh_deployment_config(
      self.pipeline_id.as_ref(),
      &self.processor_id,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      self.processor_realization.engine_target.tenant().user().to_string(),
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

  fn pipeline_id(&self) -> Option<&PipelineId> {
    self.pipeline_id.as_ref()
  }

  fn processor_id(&self) -> &ProcessorId {
    &self.processor_id
  }

  async fn start(&self) -> Result<bool, String> {
    Err("start method not yet implemented".to_string())
  }

  async fn status(&self) -> Result<ProcessorStatus, String> {
    // match target_client
    //   .client()
    //   .application_get_by_tenant_application_by_appid_status(target_client.tenant(), &self.dshapp_name, target_client.token())
    //   .await
    // {
    //   Ok(response) => match response.status() {
    //     StatusCode::OK => Ok(ProcessorStatus { up: true }),
    //     _ => Ok(ProcessorStatus { up: false }),
    //   },
    //   Err(UnexpectedResponse(response)) => match response.status() {
    //     StatusCode::NOT_FOUND => Ok(ProcessorStatus { up: false }),
    //     _ => {
    //       error!("unexpected response on get status request: {:?}", response);
    //       Err("unexpected response on status request".to_string())
    //     }
    //   },
    //   Err(error) => {
    //     error!("unexpected error on get status request: {:?}", error);
    //     Err("unexpected error on get status request".to_string())
    //   }
    // }
    todo!()
  }

  async fn stop(&self) -> Result<bool, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self) -> Result<bool, String> {
    // match target_client
    //   .client()
    //   .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant(), &self.dshapp_name, target_client.token())
    //   .await
    // {
    //   Ok(response) => match response.status() {
    //     StatusCode::ACCEPTED => Ok(true),
    //     StatusCode::NO_CONTENT => Ok(true),
    //     StatusCode::OK => Ok(true),
    //     _ => Ok(false),
    //   },
    //   Err(UnexpectedResponse(response)) => match response.status() {
    //     StatusCode::NOT_FOUND => Ok(false),
    //     _ => {
    //       error!("unexpected response on undeploy request: {:?}", response);
    //       Err("unexpected response on undeploy request".to_string())
    //     }
    //   },
    //   Err(error) => {
    //     error!("unexpected error on undeploy request: {:?}", error);
    //     Err("unexpected error on undeploy request".to_string())
    //   }
    // }
    todo!()
  }
}
