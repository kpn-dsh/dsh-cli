use std::collections::HashMap;

use log::debug;
use serde::Deserialize;

use crate::is_valid_id;
use crate::processor::processor_config::{read_processor_config, ProcessorConfig, VariableConfig, VariableType};
use crate::processor::processor_descriptor::ProfileDescriptor;
use crate::processor::ProcessorType;

#[derive(Clone, Debug, Deserialize)]
pub struct DshServiceSpecificConfig {
  pub image: String,
  #[serde(rename = "needs-token")]
  pub needs_token: bool,
  #[serde(rename = "single-instance")]
  pub single_instance: bool,
  #[serde(rename = "spread-group")]
  pub spread_group: Option<String>,
  #[serde(rename = "exposed-ports")]
  pub exposed_ports: Option<HashMap<String, PortMappingConfig>>,
  #[serde(rename = "health-check")]
  pub health_check: Option<HealthCheckConfig>,
  pub metrics: Option<MetricsConfig>,
  pub secrets: Option<Vec<SecretConfig>>,
  pub volumes: Option<HashMap<String, String>>,
  #[serde(rename = "environment-variables")]
  pub environment_variables: Option<HashMap<String, VariableConfig>>,
  pub profiles: Vec<ProfileConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortMappingConfig {
  pub auth: Option<String>,
  pub mode: Option<String>,
  pub paths: Vec<String>,
  #[serde(rename = "service-group")]
  pub service_group: Option<String>,
  pub tls: Option<PortMappingTls>,
  pub vhost: Option<String>,
  pub whitelist: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum PortMappingTls {
  #[serde(rename = "auto")]
  Auto,
  #[serde(rename = "none")]
  None,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HealthCheckConfig {
  pub path: String,
  pub port: u64,
  pub protocol: Option<HealthCheckProtocol>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum HealthCheckProtocol {
  #[serde(rename = "http")]
  Http,
  #[serde(rename = "https")]
  Https,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsConfig {
  pub path: String,
  pub port: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SecretConfig {
  pub injections: Vec<HashMap<String, String>>,
  pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileConfig {
  pub id: String,
  pub label: String,
  pub description: String,
  pub cpus: f64,
  pub instances: u64,
  pub mem: u64,
  #[serde(rename = "environment-variables")]
  pub environment_variables: Option<HashMap<String, VariableConfig>>,
}

impl ProfileConfig {
  pub fn validate(&self, id: &str) -> Result<(), String> {
    if !is_valid_id(&self.id) {
      return Err(format!("profile has invalid identifier '{}'", id));
    }
    if self.label.is_empty() {
      return Err(format!("profile '{}' has empty label", id));
    }
    if self.description.is_empty() {
      return Err(format!("profile '{}' has empty description", id));
    }
    if self.cpus < 0.1_f64 {
      return Err(format!("profile '{}' has number of cpus smaller than 0.1", id));
    }
    Ok(())
  }
}

pub fn read_dsh_service_config(config_file_name: &str) -> Result<ProcessorConfig, String> {
  let processor_config = read_processor_config(config_file_name, ProcessorType::DshService)?;
  let dsh_service_specific_config = match processor_config.dsh_service_specific_config {
    Some(ref config) => config,
    None => return Err("dsh service configuration missing".to_string()),
  };
  if dsh_service_specific_config.image.is_empty() {
    return Err("dsh service image cannot be empty".to_string());
  }
  if dsh_service_specific_config.spread_group.clone().is_some_and(|spread_group| spread_group.is_empty()) {
    return Err("spread group cannot be empty".to_string());
  }
  if dsh_service_specific_config
    .exposed_ports
    .clone()
    .is_some_and(|exposed_ports| exposed_ports.is_empty())
  {
    return Err("exposed ports cannot be empty".to_string());
  }
  if let Some(ref variables) = &dsh_service_specific_config.environment_variables {
    for (variable_name, variable) in variables {
      variable.validate(variable_name)?;
      if variable.typ == VariableType::DeploymentParameter {
        if let Some(deploy_config) = &processor_config.deploy {
          if let Some(ref parameters) = deploy_config.parameters {
            if !parameters.iter().any(|p| p.id == variable.id.clone().unwrap()) {
              return Err(format!(
                "variable '{}' references unspecified deployment parameter '{}'",
                variable_name,
                variable.id.clone().unwrap()
              ));
            };
          } else {
            return Err(format!(
              "variable '{}' references deployment parameter '{}' but none are specified",
              variable_name,
              variable.id.clone().unwrap()
            ));
          }
        }
      }
    }
  }
  if dsh_service_specific_config.profiles.is_empty() {
    return Err("no profiles defined".to_string());
  } else {
    for profile in &dsh_service_specific_config.profiles {
      profile.validate(&profile.id)?
    }
  }
  debug!("successfully validated config");
  Ok(processor_config)
}

impl ProfileConfig {
  pub(crate) fn convert_to_descriptor(self: &ProfileConfig) -> ProfileDescriptor {
    ProfileDescriptor {
      id: self.id.clone(),
      label: self.label.clone(),
      description: self.description.clone(),
      instances: Some(self.instances),
      cpus: Some(self.cpus),
      mem: Some(self.mem),
    }
  }
}

#[test]
fn read_dsh_service_config_proper_values() {
  use crate::processor::processor_config::{DeploymentParameterConfigOption, DeploymentParameterConfigOptionLabel};

  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/processors/dsh-services/dsh-service-config-test.toml");
  let config = &read_processor_config(path.to_str().unwrap(), ProcessorType::DshService).unwrap();

  assert_eq!(config.processor_type, ProcessorType::DshService);
  assert_eq!(config.id, "test");
  assert_eq!(config.description, "Test profiles");
  assert_eq!(config.version, Some("0.1.2".to_string()));
  assert_eq!(config.more_info_url, Some("https://dsh.kpn.com".to_string()));
  assert_eq!(config.metrics_url, Some("https://grafana.com".to_string()));
  assert_eq!(config.viewer_url, Some("https://eavesdropper.kpn.com".to_string()));

  let metadata = config.metadata.clone().unwrap();
  assert_eq!(
    metadata,
    vec![("metadata1".to_string(), "METADATA1".to_string()), ("metadata2".to_string(), "METADATA2".to_string())]
  );

  let inbound_junctions = config.inbound_junctions.clone().unwrap();
  assert_eq!(inbound_junctions.len(), 1);
  assert_eq!(inbound_junctions.get("inbound-topic").unwrap().label, "Test inbound topic");
  assert_eq!(inbound_junctions.get("inbound-topic").unwrap().description, "Test inbound topic description");
  assert_eq!(
    inbound_junctions.get("inbound-topic").unwrap().allowed_resource_types,
    vec![crate::resource::ResourceType::DshTopic]
  );

  let outbound_junctions = config.outbound_junctions.clone().unwrap();
  assert_eq!(outbound_junctions.len(), 1);
  assert_eq!(outbound_junctions.get("outbound-topic").unwrap().label, "Test outbound topic");
  assert_eq!(outbound_junctions.get("outbound-topic").unwrap().description, "Test outbound topic description");
  assert_eq!(
    outbound_junctions.get("outbound-topic").unwrap().allowed_resource_types,
    vec![crate::resource::ResourceType::DshTopic]
  );

  let parameters: &Vec<crate::processor::processor_config::DeploymentParameterConfig> = &config.deploy.clone().unwrap().parameters.unwrap();

  fn test(
    deploy_parameters: &Vec<crate::processor::processor_config::DeploymentParameterConfig>,
    id: &str,
    label: &str,
    default: Option<&str>,
    description: &str,
    initial_value: Option<&str>,
    optional: Option<bool>,
    options: Option<Vec<DeploymentParameterConfigOption>>,
    typ: crate::processor::processor_config::DeploymentParameterType,
  ) {
    let parameter = deploy_parameters.iter().find(|p| p.id == id).unwrap();
    assert_eq!(parameter.label, label);
    assert_eq!(parameter.default, default.map(|s| s.to_string()));
    assert_eq!(parameter.description, description);
    assert_eq!(parameter.initial_value, initial_value.map(|s| s.to_string()));
    assert_eq!(parameter.optional, optional);
    assert_eq!(parameter.options, options);
    assert_eq!(parameter.typ, typ);
  }

  fn test_b(
    deploy_parameters: &Vec<crate::processor::processor_config::DeploymentParameterConfig>,
    id: &str,
    label: &str,
    default: Option<&str>,
    description: &str,
    initial_value: Option<&str>,
    optional: Option<bool>,
  ) {
    test(
      deploy_parameters,
      id,
      label,
      default,
      description,
      initial_value,
      optional,
      None,
      crate::processor::processor_config::DeploymentParameterType::Boolean,
    )
  }

  fn test_f(
    deploy_parameters: &Vec<crate::processor::processor_config::DeploymentParameterConfig>,
    id: &str,
    label: &str,
    default: Option<&str>,
    description: &str,
    initial_value: Option<&str>,
    optional: Option<bool>,
  ) {
    test(
      deploy_parameters,
      id,
      label,
      default,
      description,
      initial_value,
      optional,
      None,
      crate::processor::processor_config::DeploymentParameterType::FreeText,
    )
  }

  fn test_s(
    deploy_parameters: &Vec<crate::processor::processor_config::DeploymentParameterConfig>,
    id: &str,
    label: &str,
    default: Option<&str>,
    description: &str,
    initial_value: Option<&str>,
    optional: Option<bool>,
    options: Option<Vec<DeploymentParameterConfigOption>>,
  ) {
    test(
      deploy_parameters,
      id,
      label,
      default,
      description,
      initial_value,
      optional,
      options,
      crate::processor::processor_config::DeploymentParameterType::Selection,
    )
  }

  assert_eq!(parameters.len(), 11);

  test_b(parameters, "bool1", "B1", None, "DB1", None, None);
  test_b(parameters, "bool2", "B2", Some("true"), "DB2", None, Some(true));
  test_b(parameters, "bool3", "B3", None, "DB3", Some("true"), None);

  test_f(parameters, "free1", "F1", None, "DF1", None, None);
  test_f(parameters, "free2", "F2", None, "DF2", Some("I2"), None);
  test_f(parameters, "free3", "F3", Some("D3"), "DF3", None, Some(true));
  test_f(parameters, "free4", "F4", Some("D4"), "DF4", Some("I4"), Some(true));

  let option_id = DeploymentParameterConfigOption::Id { 0: "s11".to_string() };
  let option_label1 = DeploymentParameterConfigOption::Label(DeploymentParameterConfigOptionLabel { id: "sx1".to_string(), label: "SX1".to_string(), description: None });
  let option_label2 =
    DeploymentParameterConfigOption::Label(DeploymentParameterConfigOptionLabel { id: "sx2".to_string(), label: "SX2".to_string(), description: Some("D2".to_string()) });

  test_s(parameters, "sel1", "S1", None, "DS1", None, None, Some(vec![option_id.clone()]));
  test_s(
    parameters,
    "sel2",
    "S2",
    Some("sx2"),
    "DS2",
    None,
    Some(true),
    Some(vec![option_label1.clone(), option_label2]),
  );
  test_s(
    parameters,
    "sel3",
    "S3",
    None,
    "DS3",
    Some("s11"),
    None,
    Some(vec![option_label1, option_id.clone()]),
  );
  test_s(parameters, "sel4", "S4", Some("s11"), "DS4", Some("s11"), Some(true), Some(vec![option_id]));

  let dsh_service_specific_config = config.dsh_service_specific_config.as_ref().unwrap();

  assert_eq!(dsh_service_specific_config.image, "test-image:0.1.2-SNAPSHOT");
  assert_eq!(dsh_service_specific_config.needs_token, true);
  assert_eq!(dsh_service_specific_config.single_instance, false);

  let metrics = dsh_service_specific_config.metrics.clone().unwrap();
  assert_eq!(metrics.port, 9095);
  assert_eq!(metrics.path, "/metrics");

  let exposed_ports = dsh_service_specific_config.exposed_ports.clone().unwrap().get("3000").unwrap().clone();
  assert_eq!(exposed_ports.vhost.unwrap(), "{ vhost('your-vhost-name','a-zone') }");
  assert_eq!(exposed_ports.auth.unwrap(), "app-realm:admin:$1$EZsDrd93$7g2osLFOay4.TzDgGo9bF/");
  assert_eq!(exposed_ports.mode.unwrap(), "http");
  assert_eq!(exposed_ports.whitelist.unwrap(), "0.0.0.0 127.0.0.1");
  assert_eq!(exposed_ports.paths, vec!("/abc"));
  assert_eq!(exposed_ports.service_group.unwrap(), "mygroup");

  let health_check = dsh_service_specific_config.health_check.clone().unwrap();
  assert_eq!(health_check.port, 8080);
  assert_eq!(health_check.protocol.unwrap(), HealthCheckProtocol::Http);
  assert_eq!(health_check.path, "/healthpath");

  let secret = dsh_service_specific_config.secrets.clone().unwrap().first().unwrap().clone();
  assert_eq!(secret.name, "secret_name");
  assert_eq!(secret.injections.first().unwrap().get("env").unwrap(), "SECRET");
  assert_eq!(dsh_service_specific_config.spread_group.clone().unwrap(), "SPREAD_GROUP");

  let volumes = dsh_service_specific_config.volumes.clone().unwrap();
  assert_eq!(volumes.get("/volume_path").unwrap(), "{ volume('correct_volume_name') }");

  let deployment_parameters = config.deploy.as_ref().unwrap().parameters.clone().unwrap();
  assert_eq!(deployment_parameters.len(), 11);

  let environment_variables = dsh_service_specific_config.environment_variables.clone().unwrap();
  assert_eq!(environment_variables.len(), 5);

  let env = environment_variables.get("DSH_SERVICE_ENV_VAR1").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.id.unwrap(), "bool1");
  assert!(env.value.is_none());

  let env = environment_variables.get("DSH_SERVICE_ENV_VAR2").unwrap().clone();
  assert_eq!(env.typ, VariableType::InboundJunction);
  assert_eq!(env.id.unwrap(), "inbound-topic");
  assert!(env.value.is_none());

  let env = environment_variables.get("DSH_SERVICE_ENV_VAR3").unwrap().clone();
  assert_eq!(env.typ, VariableType::OutboundJunction);
  assert_eq!(env.id.unwrap(), "outbound-topic");
  assert!(env.value.is_none());

  let env = environment_variables.get("DSH_SERVICE_ENV_VAR4").unwrap().clone();
  assert_eq!(env.typ, VariableType::Template);
  assert!(env.id.is_none());
  assert_eq!(env.value.unwrap(), "value4${TENANT}");

  let env = environment_variables.get("DSH_SERVICE_ENV_VAR5").unwrap().clone();
  assert_eq!(env.typ, VariableType::Value);
  assert!(env.id.is_none());
  assert_eq!(env.value.unwrap(), "value5");
}

#[test]
fn read_dsh_service_config_profile_proper_values() {
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/processors/dsh-services/dsh-service-config-test.toml");
  let config = &read_processor_config(path.to_str().unwrap(), ProcessorType::DshService).unwrap();
  let dsh_service_specific_config = config.dsh_service_specific_config.as_ref().unwrap();

  let profile1 = dsh_service_specific_config.profiles.iter().find(|p| p.id == "profile-1").unwrap().clone();
  assert_eq!(profile1.description, "Profile 1");
  assert_eq!(profile1.cpus, 1.0);
  assert_eq!(profile1.mem, 1);
  assert_eq!(profile1.instances, 1);

  let env1 = profile1.environment_variables.unwrap().clone();

  let env = env1.get("PROFILE1_ENV_VAR1").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.id.unwrap(), "free1");
  assert!(env.value.is_none());

  let env = env1.get("PROFILE1_ENV_VAR2").unwrap().clone();
  assert_eq!(env.typ, VariableType::InboundJunction);
  assert_eq!(env.id.unwrap(), "inbound-topic");
  assert!(env.value.is_none());

  let env = env1.get("PROFILE1_ENV_VAR3").unwrap().clone();
  assert_eq!(env.typ, VariableType::OutboundJunction);
  assert_eq!(env.id.unwrap(), "outbound-topic");
  assert!(env.value.is_none());

  let env = env1.get("PROFILE1_ENV_VAR4").unwrap().clone();
  assert_eq!(env.typ, VariableType::Template);
  assert!(env.id.is_none());
  assert_eq!(env.value.unwrap(), "value14${TENANT}");

  let env = env1.get("PROFILE1_ENV_VAR5").unwrap().clone();
  assert_eq!(env.typ, VariableType::Value);
  assert!(env.id.is_none());
  assert_eq!(env.value.unwrap(), "value15");
}