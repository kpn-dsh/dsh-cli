#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use trifonius_dsh_api::dsh_api_client::DshApiClient;
use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiError;

use crate::engine_target::{from_tenant_to_template_mapping, EngineTarget, TemplateMapping};
use crate::pipeline::PipelineId;
use crate::placeholder::PlaceHolder;
use crate::processor::dshservice::dshservice_api::into_api_application;
use crate::processor::dshservice::dshservice_config::ProfileConfig;
use crate::processor::dshservice::DshServiceName;
use crate::processor::processor_config::{JunctionConfig, ProcessorConfig};
use crate::processor::processor_instance::{ProcessorInstance, ProcessorStatus};
use crate::processor::{JunctionId, ParameterId, ProcessorId, ProcessorProfileId};
use crate::resource::resource_descriptor::ResourceDirection;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::{ResourceIdentifier, ResourceRealizationId, ResourceType};

pub struct DshServiceInstance<'a> {
  pipeline_id: Option<PipelineId>,
  processor_id: ProcessorId,
  processor_config: &'a ProcessorConfig,
  dshservice_name: DshServiceName,
  engine_target: Arc<EngineTarget>,
  resource_registry: Arc<ResourceRegistry>,
}

impl<'a> DshServiceInstance<'a> {
  pub fn create(
    pipeline_id: Option<PipelineId>,
    processor_id: ProcessorId,
    processor_config: &'a ProcessorConfig,
    engine_target: Arc<EngineTarget>,
    resource_registry: Arc<ResourceRegistry>,
  ) -> Result<Self, String> {
    let dshservice_name = DshServiceName::try_from((pipeline_id.as_ref(), &processor_id))?;
    Ok(Self { pipeline_id, processor_id, processor_config, dshservice_name, engine_target, resource_registry })
  }
}

#[async_trait]
impl ProcessorInstance for DshServiceInstance<'_> {
  async fn compatible_resources(&self, junction_id: &JunctionId) -> Result<Vec<ResourceIdentifier>, String> {
    if let Some((direction, junction_config)) = self
      .processor_config
      .inbound_junctions
      .as_ref()
      .and_then(|m| m.get(junction_id).map(|config| (ResourceDirection::Inbound, config)))
      .or_else(|| {
        self
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
    profile_id: Option<&ProcessorProfileId>,
  ) -> Result<(), String> {
    let dsh_application_config = self.dsh_deployment_config(
      self.pipeline_id.as_ref(),
      &self.processor_id,
      inbound_junctions,
      outbound_junctions,
      deploy_parameters,
      profile_id,
      self.engine_target.tenant().user().to_string(),
    )?;
    let client: DshApiClient = self.engine_target.dsh_api_client().await?;
    match client.create_application(&self.dshservice_name, dsh_application_config).await {
      Ok(()) => Ok(()),
      Err(DshApiError::NotFound) => Err(format!("unexpected NotFound response when deploying service {}", &self.dshservice_name)),
      Err(DshApiError::NotAuthorized) => Err(format!("authorization failure when deploying service {}", &self.dshservice_name)),
      Err(DshApiError::Unexpected(error)) => Err(format!("unexpected error when deploying service {} ({})", &self.dshservice_name, error)),
    }
  }

  async fn deploy_dry_run(
    &self,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProcessorProfileId>,
  ) -> Result<String, String> {
    let dsh_application_config = self.dsh_deployment_config(
      self.pipeline_id.as_ref(),
      &self.processor_id,
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
    match self
      .engine_target
      .dsh_api_client()
      .await?
      .get_application_allocation_status(&self.dshservice_name)
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
      Err(DshApiError::NotAuthorized) => Err(format!("authorization failure when requesting status for {} service", &self.dshservice_name)),
      Err(DshApiError::Unexpected(error)) => Err(format!("unexpected error when requesting status for {} service ({})", &self.dshservice_name, error)),
    }
  }

  async fn stop(&self) -> Result<bool, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self) -> Result<bool, String> {
    match self.engine_target.dsh_api_client().await?.delete_application(&self.dshservice_name).await {
      Ok(()) => Ok(true),
      Err(DshApiError::NotFound) => Ok(false),
      Err(DshApiError::NotAuthorized) => Ok(false),
      Err(DshApiError::Unexpected(message)) => Err(format!("unexpected error when undeploying {} service ({})", &self.dshservice_name, message)),
    }
  }
}

impl DshServiceInstance<'_> {
  fn dsh_deployment_config(
    &self,
    pipeline_id: Option<&PipelineId>,
    processor_id: &ProcessorId,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProcessorProfileId>,
    user: String,
  ) -> Result<Application, String> {
    let inbound_junction_topics: HashMap<JunctionId, String> = match &self.processor_config.inbound_junctions {
      Some(inbound_junction_configs) => self.junction_topics(ResourceDirection::Inbound, inbound_junctions, inbound_junction_configs)?,
      None => HashMap::new(),
    };
    let outbound_junction_topics: HashMap<JunctionId, String> = match &self.processor_config.outbound_junctions {
      Some(outbound_junction_configs) => self.junction_topics(ResourceDirection::Outbound, outbound_junctions, outbound_junction_configs)?,
      None => HashMap::new(),
    };

    let mut validated_parameters = HashMap::<String, String>::new();
    match &self.processor_config.deploy {
      Some(deploy) => match &deploy.parameters {
        Some(parameters) => {
          for parameter_config in parameters {
            match deploy_parameters.get(&ParameterId::new(parameter_config.id.as_str())) {
              Some(deploy_parameter) => _ = validated_parameters.insert(parameter_config.id.to_string(), deploy_parameter.to_string()),
              None => match &parameter_config.default {
                Some(default) => _ = validated_parameters.insert(parameter_config.id.to_string(), default.clone()),
                None => {
                  if !parameter_config.optional.is_some_and(|b| b) {
                    return Err(format!("mandatory deployment parameter '{}' is not provided", parameter_config.id));
                  }
                }
              },
            }
          }
        }
        None => {}
      },
      None => {}
    }

    let dshservice_specific_config = self.processor_config.dshservice_specific_config.as_ref().unwrap();
    let profile: ProfileConfig = match profile_id {
      Some(pn) => match dshservice_specific_config.profiles.iter().find(|p| p.id == pn.0) {
        Some(p) => p.clone(),
        None => return Err(format!("profile '{}' is not defined", pn)),
      },
      None => {
        if dshservice_specific_config.profiles.is_empty() {
          return Err("no default profile defined".to_string());
        } else if dshservice_specific_config.profiles.len() == 1 {
          dshservice_specific_config.profiles.first().cloned().unwrap()
        } else {
          return Err("unable to select default profile".to_string());
        }
      }
    };
    let mut template_mapping: TemplateMapping = from_tenant_to_template_mapping(self.engine_target.tenant());
    template_mapping.insert(
      PlaceHolder::ProcessorRealizationId,
      self.processor_config.processor.processor_realization_id.clone(),
    );
    if let Some(pipeline_name) = pipeline_id {
      template_mapping.insert(PlaceHolder::PipelineId, pipeline_name.to_string());
    }
    template_mapping.insert(PlaceHolder::ProcessorId, processor_id.to_string());
    let dsh_service_name = DshServiceName::try_from((pipeline_id, processor_id))?;
    template_mapping.insert(PlaceHolder::ServiceName, dsh_service_name.to_string());
    template_mapping.insert(PlaceHolder::DshServiceName, dsh_service_name.to_string());
    let api_application = into_api_application(
      pipeline_id,
      processor_id,
      &dsh_service_name,
      dshservice_specific_config,
      &inbound_junction_topics,
      &outbound_junction_topics,
      &validated_parameters,
      &profile,
      user,
      &template_mapping,
    )?;
    Ok(api_application)
  }

  fn junction_topics(
    &self,
    in_out: ResourceDirection,
    junctions_resources: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    junctions_configs: &HashMap<JunctionId, JunctionConfig>,
  ) -> Result<HashMap<JunctionId, String>, String> {
    let mut junction_topics = HashMap::<JunctionId, String>::new();
    for (junction_id, junction_config) in junctions_configs {
      let multiple_resources_separator = junction_config.multiple_resources_separator.clone().unwrap_or(",".to_string());
      match junctions_resources.get(junction_id) {
        Some(junction_resource_ids) => {
          if let Some(illegal_resource) = junction_resource_ids.iter().find(|ri| ri.resource_type != ResourceType::DshTopic) {
            return Err(format!(
              "resource '{}' connected to {} junction '{}' has wrong type, '{}' expected",
              illegal_resource,
              in_out,
              junction_id,
              ResourceType::DshTopic
            ));
          }
          let (min, max) = junction_config.number_of_resources_range();
          if junction_resource_ids.len() < min as usize {
            return Err(format!(
              "there should be at least {} resource instance(s) connected to {} junction '{}'",
              min, in_out, junction_id
            ));
          }
          if junction_resource_ids.len() > max as usize {
            return Err(format!(
              "there can be at most {} resource instance(s) connected to {} junction '{}'",
              min, in_out, junction_id
            ));
          }
          let mut topics = Vec::<String>::new();
          for resource_id in junction_resource_ids {
            match self.resource_registry.resource_realization_by_identifier(resource_id) {
              Some(resource) => match &resource.descriptor().dshtopic_descriptor {
                Some(dshtopic_descriptor) => topics.push(dshtopic_descriptor.topic.to_string()),
                None => unreachable!(),
              },
              None => {
                return Err(format!(
                  "resource '{}' connected to {} junction '{}' does not exist",
                  resource_id, in_out, junction_id
                ))
              }
            }
          }
          junction_topics.insert(junction_id.clone(), topics.join(multiple_resources_separator.as_str()));
        }
        None => {
          let (min, max) = junction_config.number_of_resources_range();
          if min != 0 || max != 0 {
            return Err(format!("required {} junction resources '{}' are not provided", in_out, junction_id));
          }
        }
      }
    }
    Ok(junction_topics)
  }
}
