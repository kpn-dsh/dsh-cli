#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;
use dsh_rest_api_client::Error::UnexpectedResponse;
use log::error;
use reqwest::StatusCode;

use crate::placeholder::PlaceHolder;
use crate::processor::dsh_service::dsh_service_api::into_api_application;
use crate::processor::dsh_service::dsh_service_config::ProfileConfig;
use crate::processor::processor::{Processor, ProcessorStatus};
use crate::processor::processor_config::{JunctionConfig, ProcessorConfig};
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProfileDescriptor};
use crate::processor::{JunctionId, ParameterId, ProcessorId, ProcessorIdentifier, ProcessorType, ProfileId, ServiceId};
use crate::resource::resource_descriptor::ResourceDirection;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceType};
use crate::target_client::{TargetClientFactory, TemplateMapping};

pub struct DshService<'a> {
  processor_identifier: ProcessorIdentifier,
  processor_config: ProcessorConfig,
  target_client_factory: &'a TargetClientFactory,
  resource_registry: &'a ResourceRegistry<'a>,
}

impl<'a> DshService<'a> {
  pub fn create(processor_config: ProcessorConfig, target_client_factory: &'a TargetClientFactory, resource_registry: &'a ResourceRegistry) -> Result<Self, String> {
    Ok(DshService {
      processor_identifier: ProcessorIdentifier { processor_type: ProcessorType::DshService, id: ProcessorId::try_from(processor_config.id.as_str())? },
      processor_config,
      target_client_factory,
      resource_registry,
    })
  }
}

#[async_trait]
impl Processor for DshService<'_> {
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
    service_id: &ServiceId,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: &Option<ProfileId>,
  ) -> Result<(), String> {
    let inbound_junction_topics: HashMap<String, String> = match &self.processor_config.inbound_junctions {
      Some(inbound_junction_configs) => self.junctions(ResourceDirection::Inbound, inbound_junctions, inbound_junction_configs)?,
      None => HashMap::new(),
    };
    let outbound_junction_topics: HashMap<String, String> = match &self.processor_config.outbound_junctions {
      Some(outbound_junction_configs) => self.junctions(ResourceDirection::Outbound, outbound_junctions, outbound_junction_configs)?,
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

    let dsh_service_specific_config = self.processor_config.dsh_service_specific_config.as_ref().unwrap();
    let profile: ProfileConfig = match profile_id {
      Some(pn) => match dsh_service_specific_config.profiles.iter().find(|p| p.id == pn.0) {
        Some(p) => p.clone(),
        None => return Err(format!("profile '{}' is not defined", pn)),
      },
      None => {
        if dsh_service_specific_config.profiles.is_empty() {
          return Err("no default profile defined".to_string());
        } else if dsh_service_specific_config.profiles.len() == 1 {
          dsh_service_specific_config.profiles.get(1).cloned().unwrap()
        } else {
          return Err("unable to select profile".to_string());
        }
      }
    };
    let target_client = self.target_client_factory.get().await?;
    let mut template_mapping: TemplateMapping = TemplateMapping::from(self.target_client_factory);
    template_mapping.insert(PlaceHolder::ServiceId, service_id.to_string());
    let api_application = into_api_application(
      dsh_service_specific_config,
      &inbound_junction_topics,
      &outbound_junction_topics,
      &validated_parameters,
      &profile,
      target_client.user.clone(),
      &template_mapping,
    )?;
    match target_client
      .client
      .application_put_by_tenant_application_by_appid_configuration(target_client.tenant, service_id.0.as_str(), &target_client.token, &api_application)
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

  fn descriptor(&self) -> ProcessorDescriptor {
    let profiles = self
      .processor_config
      .dsh_service_specific_config
      .as_ref()
      .unwrap()
      .profiles
      .iter()
      .map(|p| p.convert_to_descriptor())
      .collect::<Vec<ProfileDescriptor>>();
    self
      .processor_config
      .convert_to_descriptor(profiles, &TemplateMapping::from(self.target_client_factory))
  }

  fn identifier(&self) -> &ProcessorIdentifier {
    &self.processor_identifier
  }

  fn id(&self) -> &ProcessorId {
    &self.processor_identifier.id
  }

  fn label(&self) -> &str {
    &self.processor_config.label
  }

  fn processor_type(&self) -> ProcessorType {
    ProcessorType::DshService
  }

  async fn start(&self, _service_id: &ServiceId) -> Result<bool, String> {
    Err("start method not yet implemented".to_string())
  }

  async fn status(&self, service_id: &ServiceId) -> Result<ProcessorStatus, String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_get_by_tenant_application_by_appid_status(target_client.tenant, service_id.0.as_str(), &target_client.token)
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

  async fn stop(&self, _service_id: &ServiceId) -> Result<bool, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self, service_id: &ServiceId) -> Result<bool, String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, service_id.0.as_str(), &target_client.token)
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

impl DshService<'_> {
  fn junctions(
    &self,
    in_out: ResourceDirection,
    junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    junction_configs: &HashMap<JunctionId, JunctionConfig>,
  ) -> Result<HashMap<String, String>, String> {
    let mut junction_topics = HashMap::<String, String>::new();
    for (junction_id, junction_config) in junction_configs {
      match junctions.get(junction_id) {
        Some(resource_ids) => {
          if let Some(illegal_resource) = resource_ids.iter().find(|ri| ri.resource_type != ResourceType::DshTopic) {
            return Err(format!(
              "resource '{}' connected to {} junction '{}' has wrong type, '{}' expected",
              illegal_resource,
              in_out,
              junction_id,
              ResourceType::DshTopic
            ));
          }
          let (min, max) = junction_config.number_of_resources_range();
          if resource_ids.len() < min as usize {
            return Err(format!(
              "there should be at least {} resource instance(s) connected to {} junction '{}'",
              min, in_out, junction_id
            ));
          }
          if resource_ids.len() > max as usize {
            return Err(format!(
              "there can be at most {} resource instance(s) connected to {} junction '{}'",
              min, in_out, junction_id
            ));
          }
          let mut topics = Vec::<String>::new();
          for resource_id in resource_ids {
            match self.resource_registry.resource_by_identifier(resource_id) {
              Some(resource) => match &resource.descriptor().dsh_topic_descriptor {
                Some(dsh_topic_descriptor) => topics.push(dsh_topic_descriptor.topic.to_string()),
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
          junction_topics.insert(junction_id.to_string(), topics.join(","));
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
