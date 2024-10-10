#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::types::Application;

use crate::engine_target::{from_tenant_to_template_mapping, TemplateMapping};
use crate::pipeline::PipelineId;
use crate::placeholder::PlaceHolder;
use crate::processor::dshapp::dshapp_config::{read_dshapp_config, ProfileConfig};
use crate::processor::dshapp::dshapp_instance::DshAppInstance;
use crate::processor::dshapp::DshAppName;
use crate::processor::processor_config::{JunctionConfig, ProcessorConfig};
use crate::processor::processor_context::ProcessorContext;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProfileDescriptor};
use crate::processor::processor_instance::ProcessorInstance;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{JunctionDirection, JunctionId, JunctionIdentifier, ParameterId, ProcessorId, ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};
use crate::resource::ResourceTechnology;
use crate::ProfileId;

// TODO Voeg environment variabelen toe die de processor beschrijven en ook in welke pipeline hij zit

#[derive(Debug)]
pub struct DshAppRealization {
  processor_identifier: ProcessorIdentifier,
  pub(crate) processor_config: ProcessorConfig,
}

impl DshAppRealization {
  pub fn create(config_file_name: &str) -> Result<Self, String> {
    let processor_config = read_dshapp_config(config_file_name)?;
    Ok(DshAppRealization {
      processor_identifier: ProcessorIdentifier {
        processor_technology: ProcessorTechnology::DshApp,
        processor_realization_id: ProcessorRealizationId::try_from(processor_config.processor.processor_realization_id.to_string())?,
      },
      processor_config,
    })
  }
}

impl ProcessorRealization for DshAppRealization {
  fn descriptor(&self, dsh_api_tenant: &DshApiTenant) -> ProcessorDescriptor {
    let profiles = self
      .processor_config
      .dshapp_specific_config
      .as_ref()
      .unwrap()
      .profiles
      .iter()
      .map(|p| p.convert_to_descriptor())
      .collect::<Vec<ProfileDescriptor>>();
    self
      .processor_config
      .convert_to_descriptor(profiles, &from_tenant_to_template_mapping(dsh_api_tenant))
  }

  fn processor_realization_id(&self) -> &ProcessorRealizationId {
    &self.processor_identifier.processor_realization_id
  }

  fn identifier(&self) -> &ProcessorIdentifier {
    &self.processor_identifier
  }

  fn label(&self) -> &str {
    &self.processor_config.processor.label
  }

  fn processor_instance<'a>(
    &'a self,
    pipeline_id: Option<PipelineId>,
    processor_id: ProcessorId,
    processor_context: Arc<ProcessorContext>,
  ) -> Result<Box<dyn ProcessorInstance + 'a>, String> {
    match DshAppInstance::create(pipeline_id, processor_id, self, processor_context) {
      Ok(processor) => Ok(Box::new(processor)),
      Err(error) => Err(error),
    }
  }

  fn processor_technology(&self) -> ProcessorTechnology {
    ProcessorTechnology::DshApp
  }
}

impl DshAppRealization {
  pub(crate) fn dsh_deployment_config(
    &self,
    pipeline_id: Option<&PipelineId>,
    processor_id: &ProcessorId,
    inbound_junctions: &HashMap<JunctionId, Vec<JunctionIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<JunctionIdentifier>>,
    deploy_parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
    _user: String,
    processor_context: Arc<ProcessorContext>,
  ) -> Result<Application, String> {
    let _inbound_junction_topics: HashMap<JunctionId, String> = match &self.processor_config.inbound_junctions {
      Some(inbound_junction_configs) => self.junction_topics(JunctionDirection::Inbound, inbound_junctions, inbound_junction_configs, processor_context.clone())?,
      None => HashMap::new(),
    };
    let _outbound_junction_topics: HashMap<JunctionId, String> = match &self.processor_config.outbound_junctions {
      Some(outbound_junction_configs) => self.junction_topics(
        JunctionDirection::Outbound,
        outbound_junctions,
        outbound_junction_configs,
        processor_context.clone(),
      )?,
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

    let dshapp_specific_config = self.processor_config.dshapp_specific_config.as_ref().unwrap();
    let _profile: ProfileConfig = match profile_id {
      Some(pn) => match dshapp_specific_config.profiles.iter().find(|p| p.profile_id == pn.0) {
        Some(p) => p.clone(),
        None => return Err(format!("profile '{}' is not defined", pn)),
      },
      None => {
        if dshapp_specific_config.profiles.is_empty() {
          return Err("no default profile defined".to_string());
        } else if dshapp_specific_config.profiles.len() == 1 {
          dshapp_specific_config.profiles.first().cloned().unwrap()
        } else {
          return Err("unable to select default profile".to_string());
        }
      }
    };
    let mut template_mapping: TemplateMapping = from_tenant_to_template_mapping(processor_context.engine_target.tenant());
    template_mapping.insert(PlaceHolder::ProcessorRealizationId, self.processor_identifier.processor_realization_id.0.clone());
    if let Some(pipeline_id) = pipeline_id {
      template_mapping.insert(PlaceHolder::PipelineId, pipeline_id.to_string());
    }
    template_mapping.insert(PlaceHolder::ProcessorId, processor_id.to_string());
    let dshapp_name = DshAppName::try_from((pipeline_id, processor_id))?;
    template_mapping.insert(PlaceHolder::ServiceName, dshapp_name.to_string());
    template_mapping.insert(PlaceHolder::DshAppName, dshapp_name.to_string());
    // let api_application = into_api_application(
    //   todo!(),
    //   dshapp_specific_config,
    //   &inbound_junction_topics,
    //   &outbound_junction_topics,
    //   &validated_parameters,
    //   &profile,
    //   user,
    //   &template_mapping,
    // )?;
    // Ok(api_application)
    todo!()
  }

  fn junction_topics(
    &self,
    in_out: JunctionDirection,
    junctions: &HashMap<JunctionId, Vec<JunctionIdentifier>>,
    junctions_configs: &HashMap<JunctionId, JunctionConfig>,
    processor_context: Arc<ProcessorContext>,
  ) -> Result<HashMap<JunctionId, String>, String> {
    let mut junction_topics = HashMap::<JunctionId, String>::new();
    for (junction_id, junction_config) in junctions_configs {
      let multiple_resources_separator = junction_config.multiple_connections_separator.clone().unwrap_or(",".to_string());
      match junctions.get(junction_id) {
        Some(connected_junctions) => {
          if let Some(illegal_junction) = connected_junctions.iter().find(|connected_junction| match connected_junction {
            JunctionIdentifier::Processor(_, _, _) => true,
            JunctionIdentifier::Resource(resource_type, _) => *resource_type != ResourceTechnology::DshTopic,
          }) {
            return Err(format!(
              "resource junction '{}' connected to {} junction '{}' has wrong type, '{}' expected",
              illegal_junction,
              in_out,
              junction_id,
              ResourceTechnology::DshTopic
            ));
          }
          let (min, max) = junction_config.number_of_resources_range();
          if connected_junctions.len() < min as usize {
            return Err(format!(
              "there should be at least {} resource instance(s) connected to {} junction '{}'",
              min, in_out, junction_id
            ));
          }
          if connected_junctions.len() > max as usize {
            return Err(format!(
              "there can be at most {} resource instance(s) connected to {} junction '{}'",
              min, in_out, junction_id
            ));
          }
          let mut topics = Vec::<String>::new();
          for junction_id in connected_junctions {
            match junction_id {
              JunctionIdentifier::Processor(_, _, _) => unreachable!(),
              JunctionIdentifier::Resource(resource_type, resource_realization_id) => match processor_context.resource_registry.resource_realization(resource_realization_id) {
                Some(resource) => match &resource.descriptor().dshtopic_descriptor {
                  Some(dshtopic_descriptor) => topics.push(dshtopic_descriptor.topic.to_string()),
                  None => unreachable!(),
                },
                None => {
                  return Err(format!(
                    "resource junction '{}' connected to {} junction '{}' does not exist",
                    junction_id, in_out, junction_id
                  ))
                }
              },
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

impl Display for DshAppRealization {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "processor realization {}", self.processor_identifier)
  }
}
