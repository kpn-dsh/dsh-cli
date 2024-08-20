#![allow(clippy::module_inception)]

use std::collections::HashMap;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::{DshApiClientFactory, DshApiTenant};

use crate::engine_target::{from_tenant_to_template_mapping, TemplateMapping};
use crate::pipeline::PipelineName;
use crate::placeholder::PlaceHolder;
use crate::processor::dsh_service::dsh_service_api::into_api_application;
use crate::processor::dsh_service::dsh_service_config::ProfileConfig;
use crate::processor::dsh_service::dsh_service_instance::DshServiceInstance;
use crate::processor::dsh_service::DshServiceName;
use crate::processor::processor_config::{JunctionConfig, ProcessorConfig};
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProfileDescriptor};
use crate::processor::processor_instance::ProcessorInstance;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{JunctionId, ParameterId, ProcessorId, ProcessorIdentifier, ProcessorName, ProcessorType, ProfileId};
use crate::resource::resource_descriptor::ResourceDirection;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::{ResourceIdentifier, ResourceType};

pub struct DshServiceRealization<'a> {
  processor_identifier: ProcessorIdentifier,
  pub(crate) processor_config: ProcessorConfig,
  dsh_api_tenant: DshApiTenant,
  resource_registry: &'a ResourceRegistry<'a>,
}

impl<'a> DshServiceRealization<'a> {
  pub fn create(processor_config: ProcessorConfig, dsh_api_tenant: DshApiTenant, resource_registry: &'a ResourceRegistry) -> Result<Self, String> {
    Ok(DshServiceRealization {
      processor_identifier: ProcessorIdentifier { processor_type: ProcessorType::DshService, id: ProcessorId::try_from(processor_config.processor.id.as_str())? },
      processor_config,
      dsh_api_tenant,
      resource_registry,
    })
  }
}

impl<'a> ProcessorRealization<'a> for DshServiceRealization<'a> {
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
      .convert_to_descriptor(profiles, &from_tenant_to_template_mapping(&self.dsh_api_tenant))
  }

  fn id(&self) -> &ProcessorId {
    &self.processor_identifier.id
  }

  fn identifier(&self) -> &ProcessorIdentifier {
    &self.processor_identifier
  }

  fn label(&self) -> &str {
    &self.processor_config.processor.label
  }

  fn processor_instance(
    &'a self,
    pipeline_name: Option<&PipelineName>,
    processor_name: &ProcessorName,
    client_factory: &'a DshApiClientFactory,
  ) -> Result<Box<dyn ProcessorInstance + 'a>, String> {
    match DshServiceInstance::create(pipeline_name, processor_name, self, client_factory, self.resource_registry) {
      Ok(processor) => Ok(Box::new(processor)),
      Err(error) => Err(error),
    }
  }

  fn processor_type(&self) -> ProcessorType {
    ProcessorType::DshService
  }
}

impl DshServiceRealization<'_> {
  pub(crate) fn dsh_deployment_config(
    &self,
    pipeline_name: Option<&PipelineName>,
    processor_name: &ProcessorName,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
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
          dsh_service_specific_config.profiles.first().cloned().unwrap()
        } else {
          return Err("unable to select default profile".to_string());
        }
      }
    };
    let mut template_mapping: TemplateMapping = from_tenant_to_template_mapping(&self.dsh_api_tenant);
    template_mapping.insert(PlaceHolder::ProcessorId, self.processor_identifier.id.0.clone());
    if let Some(pipeline_name) = pipeline_name {
      template_mapping.insert(PlaceHolder::PipelineName, pipeline_name.to_string());
    }
    template_mapping.insert(PlaceHolder::ProcessorName, processor_name.to_string());
    let dsh_service_name = DshServiceName::try_from((pipeline_name, processor_name))?;
    template_mapping.insert(PlaceHolder::ServiceName, dsh_service_name.to_string());
    template_mapping.insert(PlaceHolder::DshServiceName, dsh_service_name.to_string());
    let api_application = into_api_application(
      pipeline_name,
      processor_name,
      &dsh_service_name,
      dsh_service_specific_config,
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
