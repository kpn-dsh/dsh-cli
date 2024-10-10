#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::fs;
use std::io::ErrorKind::NotFound;

use log::error;
use serde::Deserialize;
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use toml::de::Error as TomlError;

pub mod engine_target;
pub mod macros;
pub mod pipeline;
pub mod placeholder;
pub mod processor;
pub mod resource;
pub mod version;

const TRIFONIUS_CONFIG_DIR: &str = "TRIFONIUS_CONFIG_DIR";
const DEFAULT_CONFIG_DIR: &str = "config";

pub(crate) fn config_dir_name() -> String {
  std::env::var(TRIFONIUS_CONFIG_DIR).unwrap_or(DEFAULT_CONFIG_DIR.to_string())
}

identifier!(
  "trifonius_engine",
  ProfileId,
  "profile id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-profile-id",
  "invalid_profile_id",
  /// A `ProfileId` identifies a deployment profile which enables to provide some
  /// parameters of a pipeline (and its constituent processors and resources) at runtime,
  /// without having to define an entirely new pipeline.
);

pub fn read_config<C>(config_file_name: &str, config_type: &str) -> Result<C, String>
where
  C: for<'de> Deserialize<'de>,
{
  if config_file_name.ends_with(".json") {
    read_json_config(config_file_name, config_type)
  } else if config_file_name.ends_with(".toml") {
    read_toml_config(config_file_name, config_type)
  } else if config_file_name.ends_with(".yaml") || config_file_name.ends_with(".yml") {
    read_yaml_config(config_file_name, config_type)
  } else {
    Err(format!("config file '{}' has unrecognized file extension", config_file_name))
  }
}

pub fn read_json_config<C>(config_file_name: &str, config_type: &str) -> Result<C, String>
where
  C: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(config_file_name) {
    Ok(config_string) => match serde_json::from_str::<C>(&config_string) {
      Ok(config) => Ok(config),
      Err(error) => Err(format!(
        "error reading {} config file '{}', {}",
        config_type,
        config_file_name,
        parse_json_error_message(error)
      )),
    },
    Err(error) => match error.kind() {
      NotFound => Err(format!("could not find config file '{}'", config_file_name)),
      _ => Err(format!("error reading config file '{}', {}", config_file_name, error)),
    },
  }
}

fn parse_json_error_message(parse_error: JsonError) -> String {
  format!("parse error at line {}:{} ({})", parse_error.line(), parse_error.column(), parse_error)
}

pub fn read_toml_config<C>(config_file_name: &str, config_type: &str) -> Result<C, String>
where
  C: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(config_file_name) {
    Ok(config_string) => match toml::from_str::<C>(&config_string) {
      Ok(config) => Ok(config),
      Err(error) => Err(format!(
        "error reading {} config file '{}', {}",
        config_type,
        config_file_name,
        parse_toml_error_message(error)
      )),
    },
    Err(error) => match error.kind() {
      NotFound => Err(format!("could not find config file '{}'", config_file_name)),
      _ => Err(format!("error reading config file '{}', {}", config_file_name, error)),
    },
  }
}

fn parse_toml_error_message(parse_error: TomlError) -> String {
  const TOML_PARSE_ERROR_PREFIX: &str = "TOML parse error at ";
  let description = parse_error.message().lines().collect::<Vec<&str>>().join(", ");
  let binding = parse_error.to_string();
  match binding.lines().collect::<Vec<_>>().first() {
    Some(first_line_column) => {
      if let Some(stripped) = first_line_column.strip_prefix(TOML_PARSE_ERROR_PREFIX) {
        format!("parse error at {} ({})", stripped, description)
      } else {
        error!("{}", parse_error);
        description
      }
    }
    None => {
      error!("{}", parse_error);
      description
    }
  }
}

pub fn read_yaml_config<C>(config_file_name: &str, config_type: &str) -> Result<C, String>
where
  C: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(config_file_name) {
    Ok(config_string) => match serde_yaml::from_str::<C>(&config_string) {
      Ok(config) => Ok(config),
      Err(error) => Err(format!(
        "error reading {} config file '{}', {}",
        config_type,
        config_file_name,
        parse_yaml_error_message(error)
      )),
    },
    Err(error) => match error.kind() {
      NotFound => Err(format!("could not find config file '{}'", config_file_name)),
      _ => Err(format!("error reading config file '{}', {}", config_file_name, error)),
    },
  }
}

fn parse_yaml_error_message(parse_error: YamlError) -> String {
  match parse_error.location() {
    Some(location) => format!("parse error at line {}:{} ({})", location.line(), location.column(), parse_error),
    None => format!("parse error ({})", parse_error),
  }
}
