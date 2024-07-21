use std::fmt::{Display, Formatter};
use std::fs;
use std::io::ErrorKind::NotFound;

use serde::Deserialize;

use crate::is_valid_id;
use crate::processor::DeploymentParameterType;
use crate::resource::ResourceType;

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
