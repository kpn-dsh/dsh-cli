use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::ErrorKind::NotFound;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PortMappingTls {
  #[serde(rename = "auto")]
  Auto,
  #[serde(rename = "none")]
  None,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PortMapping {
  pub auth: Option<String>,
  pub mode: Option<String>,
  pub paths: Vec<String>,
  #[serde(rename = "service-group")]
  pub service_group: Option<String>,
  pub tls: Option<PortMappingTls>,
  pub vhost: Option<String>,
  pub whitelist: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum HealthCheckProtocol {
  #[serde(rename = "http")]
  Http,
  #[serde(rename = "https")]
  Https,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HealthCheck {
  pub path: String,
  pub port: u64,
  pub protocol: Option<HealthCheckProtocol>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metrics {
  pub path: String,
  pub port: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationSecret {
  pub injections: Vec<HashMap<String, String>>,
  pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
  #[serde(rename = "profile-name")]
  pub profile_name: String,
  #[serde(rename = "profile-description")]
  pub profile_description: String,
  pub cpus: f64,
  pub instances: u64,
  pub mem: u64,
  #[serde(rename = "environment-variables")]
  pub environment_variables: Option<HashMap<String, Variable>>,
}

impl Display for Profile {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}, {}, cpus {}, instances {}, mem {}",
      &self.profile_name, &self.profile_description, &self.cpus, &self.instances, &self.mem
    )?;
    if let Some(evs) = &self.environment_variables {
      write!(f, ", [{}]", evs.iter().map(|p| p.0.to_string()).collect::<Vec<String>>().join(", "))?;
    }
    Ok(())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum VariableType {
  #[serde(rename = "deployment-parameter")]
  DeploymentParameter,
  #[serde(rename = "template")]
  Template,
  #[serde(rename = "value")]
  Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
  #[serde(rename = "type")]
  pub typ: VariableType,
  pub key: Option<String>,
  pub value: Option<String>,
}

#[derive(Eq, Hash, PartialEq)]
pub enum PlaceHolder {
  TENANT,
  USER,
}

impl Display for PlaceHolder {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
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

impl Variable {
  fn validate(&self, attribute_name: &str) -> Result<(), String> {
    match &self.typ {
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DeploymentParameterType {
  #[serde(rename = "boolean")]
  Boolean,
  #[serde(rename = "free-text")]
  FreeText,
  #[serde(rename = "selection")]
  Selection,
  #[serde(rename = "sink-topic")]
  SinkTopic,
  #[serde(rename = "source-topic")]
  SourceTopic,
}

impl Display for DeploymentParameterType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      DeploymentParameterType::Boolean => write!(f, "boolean"),
      DeploymentParameterType::FreeText => write!(f, "free-text"),
      DeploymentParameterType::Selection => write!(f, "selection"),
      DeploymentParameterType::SinkTopic => write!(f, "sink-topic"),
      DeploymentParameterType::SourceTopic => write!(f, "source-topic"),
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeploymentParameter {
  #[serde(rename = "type")]
  typ: DeploymentParameterType,
  caption: String,
  #[serde(rename = "initial-value")]
  initial_value: Option<String>,
  options: Option<Vec<String>>,
  optional: Option<bool>,
  default: Option<String>,
}

impl DeploymentParameter {
  fn validate(&self, parameter_name: &str) -> Result<(), String> {
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

impl Display for DeploymentParameter {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}, {}", &self.caption, &self.typ)?;
    match &self.typ {
      DeploymentParameterType::Boolean => {}
      DeploymentParameterType::FreeText => {}
      DeploymentParameterType::Selection => write!(f, ", [{}]", &self.options.as_ref().unwrap().join(", "))?,
      DeploymentParameterType::SinkTopic => {}
      DeploymentParameterType::SourceTopic => {}
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationConfig {
  #[serde(rename = "application-name")]
  pub name: String,
  #[serde(rename = "application-description")]
  pub description: String,
  #[serde(rename = "application-version")]
  pub version: String,
  #[serde(rename = "grafana-url")]
  pub grafana_url: Option<String>,
  pub image: String,
  #[serde(rename = "needs-token")]
  pub needs_token: bool,
  #[serde(rename = "single-instance")]
  pub single_instance: bool,
  #[serde(rename = "exposed-ports")]
  pub exposed_ports: Option<HashMap<String, PortMapping>>,
  #[serde(rename = "health-check")]
  pub health_check: Option<HealthCheck>,
  pub metrics: Option<Metrics>,
  pub secrets: Option<Vec<ApplicationSecret>>,
  #[serde(rename = "spread-group")]
  pub spread_group: Option<String>,
  pub volumes: Option<HashMap<String, String>>,
  #[serde(rename = "deployment-parameters")]
  pub deployment_parameters: Option<HashMap<String, DeploymentParameter>>,
  #[serde(rename = "environment-variables")]
  pub environment_variables: Option<HashMap<String, Variable>>,
  pub profiles: HashMap<String, Profile>,
}

fn read_config<C>(config_file_name: &str) -> Result<C, String>
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

pub fn read_application_config(config_file_name: &str) -> Result<ApplicationConfig, String> {
  let config = read_config::<ApplicationConfig>(config_file_name)?;
  if config.name.is_empty() {
    return Err("application name cannot be empty".to_string());
  }
  if config.description.is_empty() {
    return Err("application description cannot be empty".to_string());
  }
  if config.version.is_empty() {
    return Err("application version cannot be empty".to_string());
  }
  if config.image.is_empty() {
    return Err("application image cannot be empty".to_string());
  }
  if config.profiles.is_empty() {
    return Err("no profiles defined".to_string());
  }
  if let Some(ref parameters) = config.deployment_parameters {
    for (parameter_name, parameter) in parameters {
      parameter.validate(parameter_name)?;
    }
  }
  if let Some(ref variables) = config.environment_variables {
    for (variable_name, variable) in variables {
      variable.validate(variable_name)?;
      if variable.typ == VariableType::DeploymentParameter {
        match config.deployment_parameters {
          Some(ref deployment_parameters) => {
            if !deployment_parameters.contains_key(&variable.key.clone().unwrap()) {
              return Err(format!(
                "variable '{}' references unspecified deployment parameter '{}'",
                variable_name,
                variable.key.clone().unwrap()
              ));
            };
          }
          None => {
            return Err(format!(
              "variable '{}' references deployment parameter '{}' but none are specified",
              variable_name,
              variable.key.clone().unwrap()
            ))
          }
        }
      }
    }
  }
  Ok(config)
}

#[test]
fn read_application_config_proper_values() {
  let config = read_application_config("test-config/applications/application-config-test.toml").unwrap();
  // let config = read_application_config("/Users/wilbert/Workspaces/trifonius/trifonius-engine/test-config/applications/application-config-test.toml").unwrap();

  println!("{:?}", config);
  assert_eq!(config.name, "test");
  assert_eq!(config.description, "Test profiles");
  assert_eq!(config.version, "0.1.2");
  assert_eq!(config.grafana_url.unwrap(), "www.kpn.com");
  assert_eq!(config.image, "test-image:0.1.2-SNAPSHOT");
  assert_eq!(config.needs_token, true);
  assert_eq!(config.single_instance, false);
  let metrics = config.metrics.clone().unwrap();
  assert_eq!(metrics.port, 9095);
  assert_eq!(metrics.path, "/metrics");
  let exposed_ports = config.exposed_ports.clone().unwrap().get("3000").unwrap().clone();
  assert_eq!(exposed_ports.vhost.unwrap(), "{ vhost('your-vhost-name','a-zone') }");
  assert_eq!(exposed_ports.auth.unwrap(), "app-realm:admin:$1$EZsDrd93$7g2osLFOay4.TzDgGo9bF/");
  assert_eq!(exposed_ports.mode.unwrap(), "http");
  assert_eq!(exposed_ports.whitelist.unwrap(), "0.0.0.0 127.0.0.1");
  assert_eq!(exposed_ports.paths, vec!("/abc"));
  assert_eq!(exposed_ports.service_group.unwrap(), "mygroup");
  let health_check = config.health_check.clone().unwrap();
  assert_eq!(health_check.port, 8080);
  assert_eq!(health_check.protocol.unwrap(), HealthCheckProtocol::Http);
  assert_eq!(health_check.path, "/healthpath");
  let secret = config.secrets.clone().unwrap().first().unwrap().clone();
  assert_eq!(secret.name, "secret_name");
  assert_eq!(secret.injections.first().unwrap().get("env").unwrap(), "SECRET");
  assert_eq!(config.spread_group.clone().unwrap(), "SPREAD_GROUP");
  let volumes = config.volumes.clone().unwrap();
  assert_eq!(volumes.get("/volume_path").unwrap(), "{ volume('correct_volume_name') }");

  let deployment_parameters = config.deployment_parameters.clone().unwrap();
  assert_eq!(deployment_parameters.len(), 5);

  let par = deployment_parameters.get("parameter-boolean").unwrap().clone();
  assert_eq!(par.typ, DeploymentParameterType::Boolean);
  assert_eq!(par.caption, "Boolean");
  assert!(par.optional.unwrap());
  assert_eq!(par.default.unwrap(), "false");
  assert!(par.options.is_none());

  let par = deployment_parameters.get("parameter-free-text").unwrap().clone();
  assert_eq!(par.typ, DeploymentParameterType::FreeText);
  assert_eq!(par.caption, "Free text");
  assert!(par.optional.is_none());
  assert!(par.default.is_none());
  assert!(par.options.is_none());

  let par = deployment_parameters.get("parameter-selection").unwrap().clone();
  assert_eq!(par.typ, DeploymentParameterType::Selection);
  assert_eq!(par.caption, "Selection");
  assert!(par.optional.is_none());
  assert!(par.default.is_none());
  assert_eq!(par.options.unwrap(), vec!["option1".to_string(), "option2".to_string()]);

  let par = deployment_parameters.get("parameter-sink-topic").unwrap().clone();
  assert_eq!(par.typ, DeploymentParameterType::SinkTopic);
  assert_eq!(par.caption, "Sink topic");
  assert!(par.optional.is_none());
  assert!(par.default.is_none());
  assert!(par.options.is_none());

  let par = deployment_parameters.get("parameter-source-topic").unwrap().clone();
  assert_eq!(par.typ, DeploymentParameterType::SourceTopic);
  assert_eq!(par.caption, "Source topic");
  assert!(par.optional.is_none());
  assert!(par.default.is_none());
  assert!(par.options.is_none());

  let environment_variables = config.environment_variables.clone().unwrap();
  assert_eq!(environment_variables.len(), 8);

  let env = environment_variables.get("ENV_VAR_DEPLOYMENT_PARAMETER_BOOLEAN").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "parameter-boolean");
  assert!(env.value.is_none());

  let env = environment_variables.get("ENV_VAR_DEPLOYMENT_PARAMETER_FREE_TEXT").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "parameter-free-text");
  assert!(env.value.is_none());

  let env = environment_variables.get("ENV_VAR_DEPLOYMENT_PARAMETER_SELECTION").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "parameter-selection");
  assert!(env.value.is_none());

  let env = environment_variables.get("ENV_VAR_DEPLOYMENT_PARAMETER_SINK_TOPIC").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "parameter-sink-topic");
  assert!(env.value.is_none());

  let env = environment_variables.get("ENV_VAR_DEPLOYMENT_PARAMETER_SOURCE_TOPIC").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "parameter-source-topic");
  assert!(env.value.is_none());

  let env = environment_variables.get("ENV_VAR_TEMPLATE_LITERAL").unwrap().clone();
  assert_eq!(env.typ, VariableType::Template);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "abcdefghijkl");

  let env = environment_variables.get("ENV_VAR_TEMPLATE_TENANT_USER").unwrap().clone();
  assert_eq!(env.typ, VariableType::Template);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "abcd${TENANT}efgh${USER}ijkl");

  let env = environment_variables.get("ENV_VAR_VALUE").unwrap().clone();
  assert_eq!(env.typ, VariableType::Value);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "VALUE");
}

#[test]
fn read_application_config_profile_proper_values() {
  let config = read_application_config("test-config/applications/application-config-test.toml").unwrap();
  // let config = read_application_config("/Users/wilbert/Workspaces/trifonius/trifonius-engine/test-config/applications/application-config-test.toml").unwrap();

  let profile1 = config.profiles.clone().get("profile-1").unwrap().clone();
  println!("{:?}", profile1);
  assert_eq!(profile1.profile_name, "profile-1");
  assert_eq!(profile1.profile_description, "Profile 1");
  assert_eq!(profile1.cpus, 1.0);
  assert_eq!(profile1.mem, 1);
  assert_eq!(profile1.instances, 1);

  let env1 = profile1.environment_variables.unwrap().clone();

  let env11 = env1.get("PROFILE1_ENV_VAR1").unwrap().clone();
  println!("{:?}", env11);
  assert_eq!(env11.typ, VariableType::DeploymentParameter);
  assert_eq!(env11.key.unwrap(), "parameter1");
  assert!(env11.value.is_none());

  let env12 = env1.get("PROFILE1_ENV_VAR2").unwrap().clone();
  println!("{:?}", env12);
  assert_eq!(env12.typ, VariableType::Value);
  assert!(env12.key.is_none());
  assert_eq!(env12.value.unwrap(), "value1");

  let profile2 = config.profiles.get("profile-2").unwrap().clone();
  println!("{:?}", profile2);
  assert_eq!(profile2.profile_name, "profile-2");
  assert_eq!(profile2.profile_description, "Profile 2");
  assert_eq!(profile2.cpus, 2.0);
  assert_eq!(profile2.mem, 2);
  assert_eq!(profile2.instances, 2);

  let env2 = profile2.environment_variables.unwrap().clone();

  let env21 = env2.get("PROFILE2_ENV_VAR1").unwrap().clone();
  println!("{:?}", env21);
  assert_eq!(env21.typ, VariableType::DeploymentParameter);
  assert_eq!(env21.key.unwrap(), "parameter2");
  assert!(env21.value.is_none());

  let env22 = env2.get("PROFILE2_ENV_VAR2").unwrap().clone();
  println!("{:?}", env22);
  assert_eq!(env22.typ, VariableType::Value);
  assert!(env22.key.is_none());
  assert_eq!(env22.value.unwrap(), "value2");
}
