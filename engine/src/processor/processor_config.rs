use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::engine_target::{template_resolver, validate_template, TemplateMapping};
use crate::placeholder::PlaceHolder;
use crate::processor::dshapp::dshapp_config::DshAppSpecificConfig;
use crate::processor::dshservice::dshservice_config::DshServiceSpecificConfig;
use crate::processor::processor_descriptor::{DeploymentParameterDescriptor, JunctionDescriptor, ProcessorDescriptor, ProfileDescriptor};
use crate::processor::{JunctionDirection, JunctionId, JunctionTechnology, ParameterId, ProcessorId, ProcessorRealizationId, ProcessorTechnology};
use crate::read_config;
use crate::version::Version;

#[derive(Clone, Debug, Deserialize)]
pub struct ProcessorConfig {
  pub processor: ProcessorGlobalConfig,
  #[serde(rename = "inbound-junctions")]
  pub inbound_junctions: Option<HashMap<JunctionId, JunctionConfig>>,
  #[serde(rename = "outbound-junctions")]
  pub outbound_junctions: Option<HashMap<JunctionId, JunctionConfig>>,
  pub deploy: Option<DeployConfig>,
  #[serde(rename = "health-check")]
  pub health_check: Option<HealthCheck>,
  #[serde(rename = "dshapp")]
  pub dshapp_specific_config: Option<DshAppSpecificConfig>,
  #[serde(rename = "dshservice")]
  pub dshservice_specific_config: Option<DshServiceSpecificConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProcessorGlobalConfig {
  #[serde(rename = "processor-technology")]
  pub processor_technology: ProcessorTechnology,
  #[serde(rename = "processor-realization-id")]
  pub processor_realization_id: ProcessorRealizationId,
  pub label: String,
  pub description: String,
  pub version: Version,
  pub icon: Option<String>, // TODO Is String the proper type?
  pub tags: Option<Vec<String>>,
  #[serde(rename = "more-info-url")]
  pub more_info_url: Option<String>,
  #[serde(rename = "metrics-url")]
  pub metrics_url: Option<String>,
  #[serde(rename = "viewer-url")]
  pub viewer_url: Option<String>,
  pub metadata: Option<Vec<(String, String)>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JunctionConfig {
  #[serde(rename = "junction-technology")]
  pub junction_technology: JunctionTechnology,
  pub label: String,
  pub description: String,
  #[serde(rename = "minimum-number-of-connections")]
  pub minimum_number_of_connections: Option<u32>,
  #[serde(rename = "maximum-number-of-connections")]
  pub maximum_number_of_connections: Option<u32>,
  #[serde(rename = "multiple-connections-separator")]
  pub multiple_connections_separator: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeployConfig {
  pub parameters: Option<Vec<DeploymentParameterConfig>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HealthCheck {
  pub ping: Option<HealthCheckPing>,
  #[serde(rename = "http-response-status")]
  pub http_response_status: Option<HealthCheckHttpResponseStatus>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HealthCheckPing {
  pub host: VariableConfig,
  #[serde(rename = "timeout-millis")]
  pub timeout_millis: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HealthCheckHttpResponseStatus {
  pub endpoint: VariableConfig,
  #[serde(rename = "accepted-response-status")]
  pub accepted_response_status: Option<Vec<u32>>,
  #[serde(rename = "timeout-millis")]
  pub timeout_millis: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum VariableType {
  #[serde(rename = "deployment-parameter")]
  DeploymentParameter,
  #[serde(rename = "inbound-junction")]
  InboundJunction,
  #[serde(rename = "outbound-junction")]
  OutboundJunction,
  #[serde(rename = "template")]
  Template,
  #[serde(rename = "value")]
  Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VariableConfig {
  #[serde(rename = "type")]
  pub typ: VariableType,
  pub id: Option<String>,
  pub value: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeploymentParameterConfig {
  #[serde(rename = "type")]
  pub typ: DeploymentParameterType,
  pub id: String,
  pub label: String,
  pub description: String,
  #[serde(rename = "initial-value")]
  pub initial_value: Option<String>,
  pub options: Option<Vec<DeploymentParameterConfigOption>>,
  pub optional: Option<bool>,
  pub default: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DeploymentParameterType {
  #[serde(rename = "boolean")]
  Boolean,
  #[serde(rename = "free-text")]
  FreeText,
  #[serde(rename = "selection")]
  Selection,
  // TODO Json, Multiline, Number, RegularExpression, SelectionOrFreeText Sql, Toml, Yaml
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum DeploymentParameterConfigOption {
  Label(DeploymentParameterConfigOptionLabel),
  Id(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DeploymentParameterConfigOptionLabel {
  pub id: String,
  pub label: String,
  pub description: Option<String>,
}

impl ProcessorConfig {
  fn validate(&self, processor_technology: ProcessorTechnology) -> Result<(), String> {
    if self.processor.processor_technology != processor_technology {
      return Err(format!(
        "processor type '{}' doesn't match expected type '{}'",
        self.processor.processor_technology, processor_technology
      ));
    }
    if !ProcessorRealizationId::is_valid(&self.processor.processor_realization_id) {
      return Err(format!(
        "illegal {} name (must be between 1 and 20 characters long and may contain only lowercase alphabetical characters and digits)",
        processor_technology
      ));
    }
    if self.processor.description.is_empty() {
      return Err(format!("{} description cannot be empty", processor_technology));
    }
    if let Some(ref url) = self.processor.more_info_url {
      validate_config_template(url, "more-info-url template")?
    }
    if let Some(ref url) = self.processor.metrics_url {
      validate_config_template(url, "metrics-url template")?
    }
    if let Some(ref url) = self.processor.viewer_url {
      validate_config_template(url, "viewer-url template")?
    }
    if let (Some(inbound), Some(outbound)) = (&self.inbound_junctions, &self.outbound_junctions) {
      if let Some(ambiguous_id) = inbound.keys().find(|id| outbound.contains_key(*id)) {
        return Err(format!("'{}' used as inbound as well as outbound id", ambiguous_id));
      }
    }
    if let Some(inbound_junctions) = &self.inbound_junctions {
      for (id, inbound_junction) in inbound_junctions {
        inbound_junction.validate(id)?
      }
    }
    if let Some(outbound_junctions) = &self.outbound_junctions {
      for (id, outbound_junction) in outbound_junctions {
        outbound_junction.validate(id)?
      }
    }
    if let Some(deploy_config) = &self.deploy {
      if let Some(ref parameter_configs) = deploy_config.parameters {
        for deploy_parameter_config in parameter_configs {
          deploy_parameter_config.validate()?
        }
      }
    }
    Ok(())
  }

  pub(crate) fn convert_to_descriptor(&self, profiles: Vec<ProfileDescriptor>, mapping: &TemplateMapping) -> ProcessorDescriptor {
    ProcessorDescriptor {
      technology: ProcessorTechnology::DshService,
      id: ProcessorId(self.processor.processor_realization_id.clone().parse().unwrap()), // TODO
      label: self.processor.label.clone(),
      description: self.processor.description.clone(),
      version: Some(self.processor.version.clone()),
      icon: self.processor.icon.clone(),
      tags: self.processor.tags.clone().unwrap_or_default(),
      inbound_junctions: match &self.inbound_junctions {
        Some(inbound_junctions) => inbound_junctions
          .iter()
          .map(|(id, junction_config)| junction_config.convert_to_descriptor(id, JunctionDirection::Inbound))
          .collect::<Vec<JunctionDescriptor>>(),
        None => vec![],
      },
      outbound_junctions: match &self.outbound_junctions {
        Some(outbound_junctions) => outbound_junctions
          .iter()
          .map(|(id, junction_config)| junction_config.convert_to_descriptor(id, JunctionDirection::Outbound))
          .collect::<Vec<JunctionDescriptor>>(),
        None => vec![],
      },
      deployment_parameters: match &self.deploy {
        Some(deploy_config) => match &deploy_config.parameters {
          Some(parameters) => parameters
            .iter()
            .map(|h| (h.id.clone(), h))
            .map(DeploymentParameterDescriptor::from)
            .collect::<Vec<DeploymentParameterDescriptor>>(),
          None => vec![],
        },
        None => vec![],
      },
      profiles,
      metadata: self.processor.metadata.clone().unwrap_or_default(),
      more_info_url: self.processor.more_info_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
      metrics_url: self.processor.metrics_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
      viewer_url: self.processor.viewer_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
    }
  }
}

impl JunctionConfig {
  pub fn validate(&self, id: &JunctionId) -> Result<(), String> {
    if self.label.is_empty() {
      return Err(format!("junction '{}' has empty label", id));
    }
    if self.description.is_empty() {
      return Err(format!("junction '{}' has empty description", id));
    }
    match (self.minimum_number_of_connections, self.maximum_number_of_connections) {
      (None, Some(max)) if max < 1 => return Err(format!("junction '{}' maximum number of resources must be 1 or greater", id)),
      (Some(min), Some(max)) if min > max => {
        return Err(format!(
          "junction '{}' maximum number of resources must be greater or equal to the minimum number of resources ",
          id
        ))
      }
      _ => (),
    }
    Ok(())
  }

  fn convert_to_descriptor(&self, id: &JunctionId, direction: JunctionDirection) -> JunctionDescriptor {
    let (min, max) = match (self.minimum_number_of_connections, self.maximum_number_of_connections) {
      (None, None) => (1, 1),
      (None, Some(max)) => (1, max),
      (Some(min), None) => (min, u32::MAX),
      (Some(min), Some(max)) => (min, max),
    };
    JunctionDescriptor {
      id: JunctionId(id.0.to_owned()), // TODO
      direction,
      junction_technology: self.junction_technology.clone(),
      label: self.label.clone(),
      description: self.description.clone(),
      minimum_number_of_connections: min,
      maximum_number_of_connections: max,
    }
  }

  pub(crate) fn number_of_resources_range(&self) -> (u32, u32) {
    match (self.minimum_number_of_connections, self.maximum_number_of_connections) {
      (None, None) => (1, 1),
      (None, Some(max)) => (0, max),
      (Some(min), None) => (min, u32::MAX),
      (Some(min), Some(max)) => (min, max),
    }
  }
}

impl DeployConfig {
  pub fn validate(&self, _attribute: &str) -> Result<(), String> {
    Ok(())
  }
}

impl VariableConfig {
  pub fn validate(&self, attribute: &str) -> Result<(), String> {
    match &self.typ {
      VariableType::InboundJunction => match &self.id {
        Some(id) => {
          if id.is_empty() {
            Err(format!("variable '{}' referencing inbound junction requires a non-empty 'id' attribute", attribute))
          } else {
            Ok(())
          }
        }
        None => Err(format!("variable '{}' referencing inbound junction requires a 'id' attribute", attribute)),
      },
      VariableType::OutboundJunction => match &self.id {
        Some(id) => {
          if id.is_empty() {
            Err(format!(
              "variable '{}' referencing outbound junction requires a non-empty 'id' attribute",
              attribute
            ))
          } else {
            Ok(())
          }
        }
        None => Err(format!("variable '{}' referencing outbound junction requires a 'id' attribute", attribute)),
      },
      VariableType::DeploymentParameter => match &self.id {
        Some(id) => {
          if id.is_empty() {
            Err(format!(
              "variable '{}' referencing deployment parameter requires a non-empty 'id' attribute",
              attribute
            ))
          } else {
            Ok(())
          }
        }
        None => Err(format!("variable '{}' referencing deployment parameter requires a 'id' attribute", attribute)),
      },
      VariableType::Template | VariableType::Value => match &self.value {
        Some(_) => Ok(()),
        None => Err(format!("variable '{}' requires a 'value' attribute", attribute)),
      },
    }
  }
}

impl DeploymentParameterConfig {
  // TODO More validation?
  pub fn validate(&self) -> Result<(), String> {
    if !ParameterId::is_valid(&self.id) {
      return Err(format!("illegal parameter identifier '{}'", self.id));
    }
    if self.label.is_empty() {
      return Err(format!("empty label for parameter '{}'", self.id));
    }
    match self.typ {
      DeploymentParameterType::Selection => match &self.options {
        Some(opts) => {
          if opts.is_empty() {
            Err(format!("empty options list for parameter '{}'", self.id))
          } else {
            for opt in opts {
              match opt {
                DeploymentParameterConfigOption::Label(ref label) => {
                  if label.id.is_empty() {
                    return Err(format!("empty id for parameter '{}'", self.id));
                  }
                  if label.label.is_empty() {
                    return Err(format!("empty label for parameter '{}.{}'", self.id, label.id));
                  }
                  if label.description.clone().is_some_and(|description| description.is_empty()) {
                    return Err(format!("empty description for parameter '{}.{}'", self.id, label.id));
                  }
                }
                DeploymentParameterConfigOption::Id(id) => {
                  if id.is_empty() {
                    return Err(format!("empty id for parameter '{}'", self.id));
                  }
                }
              }
            }
            Ok(())
          }
        }
        None => Err(format!("missing options attribute for parameter '{}'", self.id)),
      },
      _ => Ok(()),
    }
  }
}

impl DeploymentParameterConfigOptionLabel {
  pub fn validate(&self, _attribute: &str) -> Result<(), String> {
    Ok(())
  }
}

impl Display for ProcessorConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.processor.fmt(f)?;
    if let Some(ref inbound_junctions) = self.inbound_junctions {
      writeln!(f, "  inbound junctions")?;
      for (junction_id, junction_config) in inbound_junctions.iter() {
        writeln!(f, "    {} -> {}", junction_id, junction_config)?;
      }
    }
    if let Some(ref outbound_junctions) = self.outbound_junctions {
      writeln!(f, "  outbound junctions")?;
      for (junction_id, junction_config) in outbound_junctions.iter() {
        writeln!(f, "    {} -> {}", junction_id, junction_config)?;
      }
    }
    Ok(())
  }
}

impl Display for ProcessorGlobalConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}:{}:{} ({})",
      self.processor_realization_id, self.version, self.processor_technology, self.description
    )
  }
}

impl Display for JunctionConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} [{},{}] ({}, {})",
      self.junction_technology,
      self.minimum_number_of_connections.unwrap_or_default(),
      self.maximum_number_of_connections.unwrap_or_default(),
      self.label,
      self.description
    )
  }
}

impl Display for DeploymentParameterType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      DeploymentParameterType::Boolean => write!(f, "boolean"),
      DeploymentParameterType::FreeText => write!(f, "free-text"),
      DeploymentParameterType::Selection => write!(f, "selection"),
    }
  }
}

impl Display for &DeploymentParameterConfigOption {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DeploymentParameterConfigOption::Label(label) => match label.description {
        Some(ref description) => write!(f, "{}, {} ({})", label.id, label.label, description),
        None => write!(f, "{}, {}", label.id, label.label),
      },
      DeploymentParameterConfigOption::Id(id) => write!(f, "{}", id),
    }
  }
}

impl Display for DeploymentParameterConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}, {}", &self.label, &self.typ)?;
    match &self.typ {
      DeploymentParameterType::Boolean => {}
      DeploymentParameterType::FreeText => {}
      DeploymentParameterType::Selection => {
        let s = match self.options {
          Some(ref opts) => opts.iter().map(|o| o.to_string()).collect::<Vec<String>>().join(", "),
          None => unreachable!(),
        };
        write!(f, ", [{}]", s)?
      }
    }
    if self.optional.is_some_and(|o| o) {
      match &self.default {
        Some(dflt) => write!(f, ", optional, default is {}", dflt)?,
        None => write!(f, ", optional, no default")?,
      };
    }
    Ok(())
  }
}

pub fn read_processor_config(config_file_name: &str, processor_technology: ProcessorTechnology) -> Result<ProcessorConfig, String> {
  debug!("read {} config file: {}", processor_technology, config_file_name);
  let processor_config = read_config::<ProcessorConfig>(config_file_name, processor_technology.to_string().as_str())?;
  debug!("successfully read and parsed {} config file\n{:#?}", processor_technology, processor_config);
  processor_config.validate(processor_technology)?;
  debug!("successfully validated config");
  Ok(processor_config)
}

fn validate_config_template(template: &str, template_id: &str) -> Result<(), String> {
  static VALID_PLACEHOLDERS: [PlaceHolder; 10] = [
    PlaceHolder::AppDomain,
    PlaceHolder::ConsoleUrl,
    PlaceHolder::MonitoringUrl,
    PlaceHolder::Platform,
    PlaceHolder::Realm,
    PlaceHolder::RestAccessTokenUrl,
    PlaceHolder::RestApiUrl,
    PlaceHolder::Tenant,
    PlaceHolder::User,
    PlaceHolder::PublicVhostsDomain,
  ];
  if template.is_empty() {
    return Err(format!("{} cannot be empty", template_id));
  }
  validate_template(template, &VALID_PLACEHOLDERS).map_err(|message| format!("{} has {}", template_id, message))
}
