#![allow(clippy::module_inception)]

use std::collections::HashMap;

use async_trait::async_trait;
use dsh_rest_api_client::Error::UnexpectedResponse;
use log::error;
use reqwest::StatusCode;

use crate::processor::application::application_config::{ApplicationConfig, ProfileConfig};
use crate::processor::application::dsh_api::into_api_application;
use crate::processor::application::{TargetClientFactory, TemplateMapping};
use crate::processor::processor::{Processor, ProcessorIdentifier, ProcessorStatus};
use crate::processor::processor_config::PlaceHolder;
use crate::processor::processor_descriptor::{JunctionDescriptor, ProcessorDescriptor};
use crate::processor::ProcessorType;
use crate::resource::resource::ResourceIdentifier;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::ResourceType;

pub struct ApplicationImpl<'a> {
  processor_identifier: ProcessorIdentifier,
  processor_descriptor: ProcessorDescriptor,
  config: ApplicationConfig,
  target_client_factory: &'a TargetClientFactory,
  resource_registry: &'a ResourceRegistry<'a>,
}

impl<'a> ApplicationImpl<'a> {
  pub fn create(application_config: ApplicationConfig, client_factory: &'a TargetClientFactory, resource_registry: &'a ResourceRegistry) -> Result<Self, String> {
    let template_mapping = TemplateMapping::from(client_factory);
    Ok(ApplicationImpl {
      processor_identifier: ProcessorIdentifier { processor_type: ProcessorType::Application, id: application_config.application_id.clone() },
      processor_descriptor: ProcessorDescriptor::from((&application_config, &template_mapping)),
      config: application_config,
      target_client_factory: client_factory,
      resource_registry,
    })
  }
}

#[async_trait]
impl Processor for ApplicationImpl<'_> {
  async fn deploy(
    &self,
    service_id: &str,
    inbound_junctions: &HashMap<String, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<String, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<String, String>,
    profile_id: Option<&str>,
  ) -> Result<(), String> {
    let inbound_junction_topics = self.junctions("in", inbound_junctions, &self.processor_descriptor.inbound_junctions)?;
    let outbound_junction_topics = self.junctions("out", outbound_junctions, &self.processor_descriptor.outbound_junctions)?;
    let mut validated_parameters = HashMap::<String, String>::new();
    for parameter_descriptor in &self.processor_descriptor.deployment_parameters {
      match deploy_parameters.get(&parameter_descriptor.id) {
        Some(deploy_parameter) => _ = validated_parameters.insert(parameter_descriptor.id.to_string(), deploy_parameter.to_string()),
        None => match &parameter_descriptor.default {
          Some(default) => _ = validated_parameters.insert(parameter_descriptor.id.to_string(), default.clone()),
          None => {
            if !parameter_descriptor.optional {
              return Err(format!("required deployment parameter '{}' is not provided", parameter_descriptor.id));
            }
          }
        },
      }
    }
    let profile: ProfileConfig = match profile_id {
      Some(pn) => match self.config.application.profiles.iter().find(|p| p.id == pn) {
        Some(p) => p.clone(),
        None => return Err(format!("profile '{}' is not defined", pn)),
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
    let api_application = into_api_application(
      &self.config,
      &inbound_junction_topics,
      &outbound_junction_topics,
      &validated_parameters,
      &profile,
      target_client.user.clone(),
      &template_mapping,
    )?;
    match target_client
      .client
      .application_put_by_tenant_application_by_appid_configuration(target_client.tenant, service_id, &target_client.token, &api_application)
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

  async fn start(&self, _service_id: &str) -> Result<bool, String> {
    Err("start method not yet implemented".to_string())
  }

  async fn status(&self, service_id: &str) -> Result<ProcessorStatus, String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_get_by_tenant_application_by_appid_status(target_client.tenant, service_id, &target_client.token)
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

  async fn stop(&self, _service_id: &str) -> Result<bool, String> {
    Err("stop method not yet implemented".to_string())
  }

  async fn undeploy(&self, service_id: &str) -> Result<bool, String> {
    let target_client = self.target_client_factory.get().await?;
    match target_client
      .client
      .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant, service_id, &target_client.token)
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

impl ApplicationImpl<'_> {
  fn junctions(
    &self,
    in_out: &str,
    junctions: &HashMap<String, Vec<ResourceIdentifier>>,
    junction_descriptors: &Vec<JunctionDescriptor>,
  ) -> Result<HashMap<String, String>, String> {
    let mut junction_topics = HashMap::<String, String>::new();
    for junction_descriptor in junction_descriptors {
      match junctions.get(&junction_descriptor.id) {
        Some(resource_ids) => {
          if let Some(illegal_resource) = resource_ids.iter().find(|ri| ri.resource_type != ResourceType::DshTopic) {
            return Err(format!(
              "resource '{}' connected to {}bound junction '{}' has wrong type, '{}' expected",
              illegal_resource,
              in_out,
              junction_descriptor.id,
              ResourceType::DshTopic
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
                  "resource '{}' connected to {}bound junction '{}' does not exist",
                  resource_id, in_out, junction_descriptor.id
                ))
              }
            }
          }
          junction_topics.insert(junction_descriptor.id.to_string(), topics.join(","));
        }
        None => return Err(format!("required {}bound junction resource '{}' is not provided", in_out, junction_descriptor.id)),
      }
    }
    Ok(junction_topics)
  }
}
