use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::ErrorKind::NotFound;

use lazy_static::lazy_static;
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::is_valid_id;
use crate::processor::dsh_service::dsh_service_config::DshServiceSpecificConfig;
use crate::processor::dsh_service::{template_resolver, validate_template, TemplateMapping};
use crate::processor::processor_descriptor::{DeploymentParameterDescriptor, JunctionDescriptor, ProcessorDescriptor, ProfileDescriptor};
use crate::processor::ProcessorType;
use crate::resource::ResourceType;

#[derive(Clone, Debug, Deserialize)]
pub struct ProcessorConfig {
  #[serde(rename = "type")]
  pub processor_type: ProcessorType,
  pub id: String,
  pub label: String,
  pub description: String,
  pub version: Option<String>,
  pub metadata: Option<Vec<(String, String)>>,
  #[serde(rename = "more-info-url")]
  pub more_info_url: Option<String>,
  #[serde(rename = "metrics-url")]
  pub metrics_url: Option<String>,
  #[serde(rename = "viewer-url")]
  pub viewer_url: Option<String>,
  #[serde(rename = "inbound-junctions")]
  pub inbound_junctions: Option<HashMap<String, JunctionConfig>>,
  #[serde(rename = "outbound-junctions")]
  pub outbound_junctions: Option<HashMap<String, JunctionConfig>>,
  pub deploy: Option<DeployConfig>,
  #[serde(rename = "dsh-service")]
  pub dsh_service_specific_config: Option<DshServiceSpecificConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JunctionConfig {
  pub label: String,
  pub description: String,
  #[serde(rename = "minimum-number-of-resources")]
  pub minimum_number_of_resources: Option<u32>,
  #[serde(rename = "maximum-number-of-resources")]
  pub maximum_number_of_resources: Option<u32>,
  #[serde(rename = "allowed-resource-types")]
  pub allowed_resource_types: Vec<ResourceType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeployConfig {
  pub parameters: Option<Vec<DeploymentParameterConfig>>,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum DeploymentParameterType {
  #[serde(rename = "boolean")]
  Boolean,
  #[serde(rename = "free-text")]
  FreeText,
  #[serde(rename = "selection")]
  Selection,
  // TODO Json,
  // TODO Multiline,
  // TODO Number,
  // TODO RegularExpression,
  // TODO Sql,
  // TODO Toml,
  // TODO Yaml,
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

impl JunctionConfig {
  pub fn validate(&self, id: &str) -> Result<(), String> {
    if !is_valid_id(id) {
      return Err(format!("junction '{}' has invalid identifier", id));
    }
    if self.label.is_empty() {
      return Err(format!("junction '{}' has empty label", id));
    }
    if self.description.is_empty() {
      return Err(format!("junction '{}' has empty description", id));
    }
    match (self.minimum_number_of_resources, self.maximum_number_of_resources) {
      (None, Some(max)) if max < 1 => return Err(format!("junction '{}' maximum number of resources must be 1 or greater", id)),
      (Some(min), Some(max)) if min > max => {
        return Err(format!(
          "junction '{}' maximum number of resources must be greater or equal to the minimum number of resources ",
          id
        ))
      }
      _ => (),
    }
    if self.allowed_resource_types.is_empty() {
      return Err(format!("junction '{}' has no allowed resource types", id));
    }
    Ok(())
  }

  pub(crate) fn number_of_resources_range(&self) -> (u32, u32) {
    match (self.minimum_number_of_resources, self.maximum_number_of_resources) {
      (None, None) => (1, 1),
      (None, Some(max)) => (0, max),
      (Some(min), None) => (min, u32::MAX),
      (Some(min), Some(max)) => (min, max),
    }
  }
}

const APP_DOMAIN: &str = "APP_DOMAIN";
const CONSOLE_URL: &str = "CONSOLE_URL";
const DSH_INTERNAL_DOMAIN: &str = "DSH_INTERNAL_DOMAIN";
const MONITORING_URL: &str = "MONITORING_URL";
const PLATFORM: &str = "PLATFORM";
const PUBLIC_VHOSTS_DOMAIN: &str = "PUBLIC_VHOSTS_DOMAIN";
const RANDOM: &str = "RANDOM";
const RANDOM_UUID: &str = "RANDOM_UUID";
const REALM: &str = "REALM";
const _REGISTRY: &str = "REGISTRY"; // TODO
const REST_ACCESS_TOKEN_URL: &str = "REST_ACCESS_TOKEN_URL";
const REST_API_URL: &str = "REST_API_URL";
const SERVICE_ID: &str = "SERVICE_ID";
const TENANT: &str = "TENANT";
const USER: &str = "USER";

#[derive(Eq, Hash, PartialEq)]
pub enum PlaceHolder {
  AppDomain,
  ConsoleUrl,
  DshInternalDomain,
  MonitoringUrl,
  Platform,
  PublicVhostsDomain,
  Random,
  RandomUuid,
  Realm,
  RestAccessTokenUrl,
  RestApiUrl,
  ServiceId,
  Tenant,
  User,
}

impl Display for PlaceHolder {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      PlaceHolder::AppDomain => write!(f, "{}", APP_DOMAIN),
      PlaceHolder::ConsoleUrl => write!(f, "{}", CONSOLE_URL),
      PlaceHolder::DshInternalDomain => write!(f, "{}", DSH_INTERNAL_DOMAIN),
      PlaceHolder::MonitoringUrl => write!(f, "{}", MONITORING_URL),
      PlaceHolder::Platform => write!(f, "{}", PLATFORM),
      PlaceHolder::PublicVhostsDomain => write!(f, "{}", PUBLIC_VHOSTS_DOMAIN),
      PlaceHolder::Random => write!(f, "{}", RANDOM),
      PlaceHolder::RandomUuid => write!(f, "{}", RANDOM_UUID),
      PlaceHolder::Realm => write!(f, "{}", REALM),
      PlaceHolder::RestAccessTokenUrl => write!(f, "{}", REST_ACCESS_TOKEN_URL),
      PlaceHolder::RestApiUrl => write!(f, "{}", REST_API_URL),
      PlaceHolder::ServiceId => write!(f, "{}", SERVICE_ID),
      PlaceHolder::Tenant => write!(f, "{}", TENANT),
      PlaceHolder::User => write!(f, "{}", USER),
    }
  }
}

impl TryFrom<&str> for PlaceHolder {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      APP_DOMAIN => Ok(PlaceHolder::AppDomain),
      CONSOLE_URL => Ok(PlaceHolder::ConsoleUrl),
      DSH_INTERNAL_DOMAIN => Ok(PlaceHolder::DshInternalDomain),
      MONITORING_URL => Ok(PlaceHolder::MonitoringUrl),
      PLATFORM => Ok(PlaceHolder::Platform),
      PUBLIC_VHOSTS_DOMAIN => Ok(PlaceHolder::PublicVhostsDomain),
      RANDOM => Ok(PlaceHolder::Random),
      RANDOM_UUID => Ok(PlaceHolder::RandomUuid),
      REALM => Ok(PlaceHolder::Realm),
      REST_ACCESS_TOKEN_URL => Ok(PlaceHolder::RestAccessTokenUrl),
      REST_API_URL => Ok(PlaceHolder::RestApiUrl),
      SERVICE_ID => Ok(PlaceHolder::ServiceId),
      TENANT => Ok(PlaceHolder::Tenant),
      USER => Ok(PlaceHolder::User),
      unrecognized => Err(format!("unrecognized placeholder '{}'", unrecognized)),
    }
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

impl DeploymentParameterConfig {
  // TODO More validation?
  pub fn validate(&self, parameter: &str) -> Result<(), String> {
    if !is_valid_id(&self.id) {
      return Err(format!("illegal parameter identifier '{}'", parameter));
    }
    if self.label.is_empty() {
      return Err(format!("empty label for parameter '{}'", parameter));
    }
    match self.typ {
      DeploymentParameterType::Selection => match &self.options {
        Some(opts) => {
          if opts.is_empty() {
            Err(format!("empty options list for parameter '{}'", parameter))
          } else {
            for opt in opts {
              match opt {
                DeploymentParameterConfigOption::Label(ref label) => {
                  if label.id.is_empty() {
                    return Err(format!("empty id for parameter '{}'", parameter));
                  }
                  if label.label.is_empty() {
                    return Err(format!("empty label for parameter '{}.{}'", parameter, label.id));
                  }
                  if label.description.clone().is_some_and(|description| description.is_empty()) {
                    return Err(format!("empty description for parameter '{}.{}'", parameter, label.id));
                  }
                }
                DeploymentParameterConfigOption::Id(id) => {
                  if id.is_empty() {
                    return Err(format!("empty id for parameter '{}'", parameter));
                  }
                }
              }
            }
            Ok(())
          }
        }
        None => Err(format!("missing options attribute for parameter '{}'", parameter)),
      },
      _ => Ok(()),
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

lazy_static! {
  static ref PROCESSOR_ID_REGEX: Regex = Regex::new("^[a-z0-9]{1,20}$").unwrap();
}

pub fn read_processor_config(config_file_name: &str, processor_type: ProcessorType) -> Result<ProcessorConfig, String> {
  debug!("read {} config file: {}", processor_type, config_file_name);
  let processor_config = read_config::<ProcessorConfig>(config_file_name)?;
  debug!("successfully read and parsed {} config file\n{:#?}", processor_type, processor_config);
  if processor_config.processor_type != processor_type {
    return Err(format!(
      "processor type '{}' doesn't match expected type '{}'",
      processor_config.processor_type, processor_type
    ));
  }
  if !PROCESSOR_ID_REGEX.is_match(&processor_config.id) {
    return Err(format!(
      "illegal {} name (must be between 1 and 20 characters long and may contain only lowercase alphabetical characters and digits)",
      processor_type
    ));
  }
  if processor_config.description.is_empty() {
    return Err(format!("{} description cannot be empty", processor_type));
  }
  if processor_config.version.clone().is_some_and(|ref version| version.is_empty()) {
    return Err(format!("{} version cannot be empty", processor_type));
  }
  if let Some(ref url) = processor_config.more_info_url {
    validate_config_template(url, "more-info-url template")?
  }
  if let Some(ref url) = processor_config.metrics_url {
    validate_config_template(url, "metrics-url template")?
  }
  if let Some(ref url) = processor_config.viewer_url {
    validate_config_template(url, "viewer-url template")?
  }
  if let (Some(inbound), Some(outbound)) = (&processor_config.inbound_junctions, &processor_config.outbound_junctions) {
    if let Some(ambiguous_id) = inbound.keys().find(|id| outbound.contains_key(*id)) {
      return Err(format!("'{}' used as inbound as well as outbound id", ambiguous_id));
    }
  }
  if let Some(inbound_junctions) = &processor_config.inbound_junctions {
    for (id, inbound_junction) in inbound_junctions {
      inbound_junction.validate(id)?
    }
  }
  if let Some(outbound_junctions) = &processor_config.outbound_junctions {
    for (id, outbound_junction) in outbound_junctions {
      outbound_junction.validate(id)?
    }
  }
  if let Some(deploy_config) = &processor_config.deploy {
    if let Some(ref parameters) = deploy_config.parameters {
      for parameter in parameters {
        parameter.validate(parameter.id.as_str())?
      }
    }
  }
  debug!("successfully validated config");
  Ok(processor_config)
}

pub fn read_config<C>(config_file_name: &str) -> Result<C, String>
where
  C: for<'de> toml::macros::Deserialize<'de>,
{
  match fs::read_to_string(config_file_name) {
    Ok(config_string) => match toml::from_str::<C>(&config_string) {
      Ok(config) => Ok(config),
      Err(error) => Err(format!("could not parse config file '{}' ({})", config_file_name, error.message())),
    },
    Err(error) => match error.kind() {
      NotFound => Err(format!("config file '{}' not found", config_file_name)),
      _ => Err(format!("config file '{}' could not be read ({})", config_file_name, error)),
    },
  }
}

impl ProcessorConfig {
  pub(crate) fn convert_to_descriptor(&self, profiles: Vec<ProfileDescriptor>, mapping: &TemplateMapping) -> ProcessorDescriptor {
    ProcessorDescriptor {
      processor_type: ProcessorType::DshService,
      id: self.id.clone(),
      label: self.label.clone(),
      description: self.description.clone(),
      version: self.version.clone(),
      inbound_junctions: match &self.inbound_junctions {
        Some(inbound_junctions) => inbound_junctions
          .iter()
          .map(|(id, junction_config)| junction_config.convert_to_descriptor(id))
          .collect::<Vec<JunctionDescriptor>>(),
        None => vec![],
      },
      outbound_junctions: match &self.outbound_junctions {
        Some(outbound_junctions) => outbound_junctions
          .iter()
          .map(|(id, junction_config)| junction_config.convert_to_descriptor(id))
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
      metadata: self.metadata.clone().unwrap_or_default(),
      more_info_url: self.more_info_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
      metrics_url: self.metrics_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
      viewer_url: self.viewer_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
    }
  }
}

impl JunctionConfig {
  fn convert_to_descriptor(&self, id: &String) -> JunctionDescriptor {
    let (min, max) = match (self.minimum_number_of_resources, self.maximum_number_of_resources) {
      (None, None) => (1, 1),
      (None, Some(max)) => (1, max),
      (Some(min), None) => (min, u32::MAX),
      (Some(min), Some(max)) => (min, max),
    };
    JunctionDescriptor {
      id: id.to_owned(),
      label: self.label.clone(),
      description: self.description.clone(),
      minimum_number_of_resources: min,
      maximum_number_of_resources: max,
      allowed_resource_types: self.allowed_resource_types.clone(),
    }
  }
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
