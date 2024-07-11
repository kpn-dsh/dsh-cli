use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::ErrorKind::NotFound;

use serde::Deserialize;

use crate::processor::DeploymentParameterType;
use crate::resource::ResourceType;

#[derive(Clone, Debug, Deserialize)]
pub struct JunctionConfig {
  pub caption: String,
  pub description: String,
  #[serde(rename = "allowed-resource-types")]
  pub allowed_resource_types: Vec<ResourceType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeployConfig {
  pub parameters: Option<HashMap<String, DeploymentParameterConfig>>,
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
  pub key: Option<String>,
  pub value: Option<String>,
}

#[derive(Eq, Hash, PartialEq)]
pub enum PlaceHolder {
  INSTANCE,
  TENANT,
  USER,
}

impl Display for PlaceHolder {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      PlaceHolder::INSTANCE => write!(f, "INSTANCE"),
      PlaceHolder::TENANT => write!(f, "TENANT"),
      PlaceHolder::USER => write!(f, "USER"),
    }
  }
}

impl TryFrom<&str> for PlaceHolder {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "TENANT" => Ok(PlaceHolder::TENANT),
      "USER" => Ok(PlaceHolder::USER),
      unrecognized => Err(format!("unrecognized placeholder '{}'", unrecognized)),
    }
  }
}

impl VariableConfig {
  pub fn validate(&self, attribute_name: &str) -> Result<(), String> {
    match &self.typ {
      VariableType::InboundJunction => match &self.key {
        Some(key) => {
          if key.is_empty() {
            Err(format!(
              "variable '{}' referencing inbound junction requires a non-empty 'key' attribute",
              attribute_name
            ))
          } else {
            Ok(())
          }
        }
        None => Err(format!("variable '{}' referencing inbound junction requires a 'key' attribute", attribute_name)),
      },
      VariableType::OutboundJunction => match &self.key {
        Some(key) => {
          if key.is_empty() {
            Err(format!(
              "variable '{}' referencing outbound junction requires a non-empty 'key' attribute",
              attribute_name
            ))
          } else {
            Ok(())
          }
        }
        None => Err(format!("variable '{}' referencing outbound junction requires a 'key' attribute", attribute_name)),
      },
      VariableType::DeploymentParameter => match &self.key {
        Some(key) => {
          if key.is_empty() {
            Err(format!(
              "variable '{}' referencing deployment parameter requires a non-empty 'key' attribute",
              attribute_name
            ))
          } else {
            Ok(())
          }
        }
        None => Err(format!("variable '{}' referencing deployment parameter requires a 'key' attribute", attribute_name)),
      },
      VariableType::Template | VariableType::Value => match &self.value {
        Some(_) => Ok(()),
        None => Err(format!("variable '{}' requires a 'value' attribute", attribute_name)),
      },
    }
  }
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeploymentParameterConfig {
  #[serde(rename = "type")]
  pub typ: DeploymentParameterType,
  pub description: Option<String>,
  pub caption: String,
  #[serde(rename = "initial-value")]
  pub initial_value: Option<String>,
  pub options: Option<Vec<String>>,
  pub optional: Option<bool>,
  pub default: Option<String>,
}

impl DeploymentParameterConfig {
  // TODO More validation?
  pub fn validate(&self, parameter_name: &str) -> Result<(), String> {
    if self.caption.is_empty() {
      return Err(format!("empty caption for parameter '{}'", parameter_name));
    }
    if let Some(opt) = &self.optional {
      if *opt && self.default.is_none() {
        return Err(format!("optional parameter '{}' requires default value", parameter_name));
      }
    }
    match self.typ {
      DeploymentParameterType::Selection => match &self.options {
        Some(opts) => {
          if opts.is_empty() {
            Err(format!("empty options list for parameter '{}'", parameter_name))
          } else {
            Ok(())
          }
        }
        None => Err(format!("missing options attribute for parameter '{}'", parameter_name)),
      },
      _ => Ok(()),
    }
  }
}

impl Display for DeploymentParameterConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}, {}", &self.caption, &self.typ)?;
    match &self.typ {
      DeploymentParameterType::Boolean => {}
      DeploymentParameterType::FreeText => {}
      DeploymentParameterType::Selection => write!(f, ", [{}]", &self.options.as_ref().unwrap().join(", "))?,
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
