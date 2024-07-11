use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use log::debug;
use serde::Deserialize;

use crate::processor::application::converters::{template_resolver, validate_template};
use crate::processor::processor_config::{read_config, DeployConfig, JunctionConfig, PlaceHolder, VariableConfig, VariableType};
use crate::processor::processor_descriptor::{DeploymentParameterDescriptor, JunctionDescriptor, ProcessorDescriptor, ProfileDescriptor};
use crate::processor::ProcessorType;

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationConfig {
  #[serde(rename = "type")]
  pub processor_type: ProcessorType,

  #[serde(rename = "name")]
  pub application_name: String,

  #[serde(rename = "description")]
  pub application_description: String,

  #[serde(rename = "version")]
  pub application_version: Option<String>,

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

  pub application: ApplicationSpecificConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationSpecificConfig {
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

  pub profiles: HashMap<String, ProfileConfig>,
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
  #[serde(rename = "profile-description")]
  pub profile_description: String,
  pub cpus: f64,
  pub instances: u64,
  pub mem: u64,
  #[serde(rename = "environment-variables")]
  pub environment_variables: Option<HashMap<String, VariableConfig>>,
}

impl ProfileConfig {
  pub fn validate(&self, profile_name: &str) -> Result<(), String> {
    if self.profile_description.is_empty() {
      return Err(format!("profile '{}' has empty description", profile_name));
    }
    if self.cpus < 0.1_f64 {
      return Err(format!("profile '{}' has number of cpus smaller than 0.1", profile_name));
    }
    Ok(())
  }
}

impl Display for ProfileConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}, cpus: {}, instances: {}, mem: {}",
      &self.profile_description, &self.cpus, &self.instances, &self.mem
    )?;
    if let Some(evs) = &self.environment_variables {
      write!(f, ", [{}]", evs.iter().map(|p| p.0.to_string()).collect::<Vec<String>>().join(", "))?;
    }
    Ok(())
  }
}

impl From<(&ApplicationConfig, &HashMap<PlaceHolder, &str>)> for ProcessorDescriptor {
  // TODO Resolve template for more attributes (check validation also)
  fn from((config, mapping): (&ApplicationConfig, &HashMap<PlaceHolder, &str>)) -> Self {
    ProcessorDescriptor {
      processor_type: ProcessorType::Application,
      name: config.application_name.clone(),
      description: config.application_description.clone(),
      version: config.application_version.clone(),
      inbound_junctions: match &config.inbound_junctions {
        Some(ijsm) => ijsm.iter().map(JunctionDescriptor::from).collect::<Vec<JunctionDescriptor>>(),
        None => vec![],
      },
      outbound_junctions: match &config.outbound_junctions {
        Some(ojsm) => ojsm.iter().map(JunctionDescriptor::from).collect::<Vec<JunctionDescriptor>>(),
        None => vec![],
      },
      deployment_parameters: match &config.deploy {
        Some(deploy_config) => match &deploy_config.parameters {
          Some(parameters) => parameters
            .iter()
            .map(|h| (h.0.to_string(), h.1.clone()))
            .map(DeploymentParameterDescriptor::from)
            .collect::<Vec<DeploymentParameterDescriptor>>(),
          None => vec![],
        },
        None => vec![],
      },
      profiles: config.application.profiles.iter().map(ProfileDescriptor::from).collect::<Vec<ProfileDescriptor>>(),
      metadata: config.metadata.clone().unwrap_or_default(),
      more_info_url: config.more_info_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
      metrics_url: config.metrics_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
      viewer_url: config.viewer_url.clone().map(|ref u| template_resolver(u, mapping).unwrap_or_default()),
    }
  }
}

fn validate_config_template(template: &str, template_name: &str) -> Result<(), String> {
  static VALID_PLACEHOLDERS: [PlaceHolder; 2] = [PlaceHolder::TENANT, PlaceHolder::USER];
  if template.is_empty() {
    return Err(format!("{} cannot be empty", template_name));
  }
  validate_template(template, &VALID_PLACEHOLDERS).map_err(|m| format!("{} has {}", template_name, m))
}

pub fn read_application_config(config_file_name: &str) -> Result<ApplicationConfig, String> {
  debug!("read application config file: {}", config_file_name);
  let config = read_config::<ApplicationConfig>(config_file_name)?;
  debug!("successfully read and parsed application config file\n{:#?}", config);
  if config.processor_type != ProcessorType::Application {
    return Err(format!("processor type '{}' doesn't match file type ('application')", config.processor_type));
  }
  if config.application_name.is_empty() {
    return Err("application name cannot be empty".to_string());
  }
  if config.application_description.is_empty() {
    return Err("application description cannot be empty".to_string());
  }
  if config.application_version.clone().is_some_and(|ref version| version.is_empty()) {
    return Err("application version cannot be empty".to_string());
  }
  if let Some(ref url) = config.more_info_url {
    validate_config_template(url, "more-info-url template")?
  }
  if let Some(ref url) = config.metrics_url {
    validate_config_template(url, "metrics-url template")?
  }
  if let Some(ref url) = config.viewer_url {
    validate_config_template(url, "viewer-url template")?
  }
  if let (Some(inbound), Some(outbound)) = (&config.inbound_junctions, &config.outbound_junctions) {
    if let Some(ambiguous_key) = inbound.keys().find(|key| outbound.contains_key(*key)) {
      return Err(format!("'{}' used as inbound as well as outbound key", ambiguous_key));
    }
  }
  if let Some(deploy_config) = &config.deploy {
    if let Some(ref parameters) = deploy_config.parameters {
      for parameter in parameters {
        parameter.1.validate(parameter.0.as_str())?
      }
    }
  }
  if config.application.image.is_empty() {
    return Err("application image cannot be empty".to_string());
  }
  if config.application.spread_group.clone().is_some_and(|spread_group| spread_group.is_empty()) {
    return Err("spread group cannot be empty".to_string());
  }
  if config.application.exposed_ports.clone().is_some_and(|exposed_ports| exposed_ports.is_empty()) {
    return Err("exposed ports cannot be empty".to_string());
  }
  if config.application.secrets.clone().is_some_and(|secrets| secrets.is_empty()) {
    return Err("secrets cannot be empty".to_string());
  }
  if config.application.volumes.clone().is_some_and(|volumes| volumes.is_empty()) {
    return Err("volumes cannot be empty".to_string());
  }
  if let Some(ref variables) = &config.application.environment_variables {
    for (variable_name, variable) in variables {
      variable.validate(variable_name)?;
      if variable.typ == VariableType::DeploymentParameter {
        if let Some(deploy_config) = &config.deploy {
          if let Some(ref parameters) = deploy_config.parameters {
            if !parameters.contains_key(&variable.key.clone().unwrap()) {
              return Err(format!(
                "variable '{}' references unspecified deployment parameter '{}'",
                variable_name,
                variable.key.clone().unwrap()
              ));
            };
          } else {
            return Err(format!(
              "variable '{}' references deployment parameter '{}' but none are specified",
              variable_name,
              variable.key.clone().unwrap()
            ));
          }
        }
      }
    }
  }
  if config.application.profiles.is_empty() {
    return Err("no profiles defined".to_string());
  } else {
    for (name, profile) in &config.application.profiles {
      profile.validate(name)?
    }
  }
  debug!("successfully validated config");
  Ok(config)
}

#[test]
fn read_application_config_proper_values() {
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/processors/applications/application-config-test.toml");
  let config = &read_application_config(path.to_str().unwrap()).unwrap();

  assert_eq!(config.processor_type, ProcessorType::Application);
  assert_eq!(config.application_name, "test");
  assert_eq!(config.application_description, "Test profiles");
  assert_eq!(config.application_version, Some("0.1.2".to_string()));
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
  assert_eq!(inbound_junctions.get("inbound-topic").unwrap().caption, "Test inbound topic");
  assert_eq!(inbound_junctions.get("inbound-topic").unwrap().description, "Test inbound topic description");
  assert_eq!(
    inbound_junctions.get("inbound-topic").unwrap().allowed_resource_types,
    vec![crate::resource::ResourceType::DshTopic]
  );

  let outbound_junctions = config.outbound_junctions.clone().unwrap();
  assert_eq!(outbound_junctions.len(), 1);
  assert_eq!(outbound_junctions.get("outbound-topic").unwrap().caption, "Test outbound topic");
  assert_eq!(outbound_junctions.get("outbound-topic").unwrap().description, "Test outbound topic description");
  assert_eq!(
    outbound_junctions.get("outbound-topic").unwrap().allowed_resource_types,
    vec![crate::resource::ResourceType::DshTopic]
  );

  let parameters: &HashMap<String, crate::processor::processor_config::DeploymentParameterConfig> = &config.deploy.clone().unwrap().parameters.unwrap();

  fn test(
    deploy_parameters: &HashMap<String, crate::processor::processor_config::DeploymentParameterConfig>,
    key: &str,
    caption: &str,
    default: Option<&str>,
    description: Option<&str>,
    initial_value: Option<&str>,
    optional: Option<bool>,
    options: Option<Vec<&str>>,
    typ: crate::processor::DeploymentParameterType,
  ) {
    let parameter = deploy_parameters.get(key).unwrap();
    assert_eq!(parameter.caption, caption);
    assert_eq!(parameter.default, default.map(|s| s.to_string()));
    assert_eq!(parameter.description, description.map(|s| s.to_string()));
    assert_eq!(parameter.initial_value, initial_value.map(|s| s.to_string()));
    assert_eq!(parameter.optional, optional);
    assert_eq!(parameter.options, options.map(|os| os.iter().map(|s| s.to_string()).collect()));
    assert_eq!(parameter.typ, typ);
  }

  fn test_b(
    deploy_parameters: &HashMap<String, crate::processor::processor_config::DeploymentParameterConfig>,
    key: &str,
    caption: &str,
    default: Option<&str>,
    description: Option<&str>,
    initial_value: Option<&str>,
    optional: Option<bool>,
  ) {
    test(
      deploy_parameters,
      key,
      caption,
      default,
      description,
      initial_value,
      optional,
      None,
      crate::processor::DeploymentParameterType::Boolean,
    )
  }

  fn test_f(
    deploy_parameters: &HashMap<String, crate::processor::processor_config::DeploymentParameterConfig>,
    key: &str,
    caption: &str,
    default: Option<&str>,
    description: Option<&str>,
    initial_value: Option<&str>,
    optional: Option<bool>,
  ) {
    test(
      deploy_parameters,
      key,
      caption,
      default,
      description,
      initial_value,
      optional,
      None,
      crate::processor::DeploymentParameterType::FreeText,
    )
  }

  fn test_s(
    deploy_parameters: &HashMap<String, crate::processor::processor_config::DeploymentParameterConfig>,
    key: &str,
    caption: &str,
    default: Option<&str>,
    description: Option<&str>,
    initial_value: Option<&str>,
    optional: Option<bool>,
    options: Option<Vec<&str>>,
  ) {
    test(
      deploy_parameters,
      key,
      caption,
      default,
      description,
      initial_value,
      optional,
      options,
      crate::processor::DeploymentParameterType::Selection,
    )
  }

  assert_eq!(parameters.len(), 11);

  test_b(parameters, "bool1", "B1", None, None, None, None);
  test_b(parameters, "bool2", "B2", Some("true"), None, None, Some(true));
  test_b(parameters, "bool3", "B3", None, None, Some("true"), None);

  test_f(parameters, "free1", "F1", None, None, None, None);
  test_f(parameters, "free2", "F2", None, None, Some("I2"), None);
  test_f(parameters, "free3", "F3", Some("D3"), None, None, Some(true));
  test_f(parameters, "free4", "F4", Some("D4"), None, Some("I4"), Some(true));

  test_s(parameters, "sel1", "S1", None, None, None, None, Some(vec!["s11"]));
  test_s(parameters, "sel2", "S2", Some("s22"), None, None, Some(true), Some(vec!["s21", "s22"]));
  test_s(parameters, "sel3", "S3", None, None, Some("s32"), None, Some(vec!["s31", "s32"]));
  test_s(parameters, "sel4", "S4", Some("s41"), None, Some("s42"), Some(true), Some(vec!["s41", "s42"]));

  assert_eq!(config.application.image, "test-image:0.1.2-SNAPSHOT");
  assert_eq!(config.application.needs_token, true);
  assert_eq!(config.application.single_instance, false);

  let metrics = config.application.metrics.clone().unwrap();
  assert_eq!(metrics.port, 9095);
  assert_eq!(metrics.path, "/metrics");

  let exposed_ports = config.application.exposed_ports.clone().unwrap().get("3000").unwrap().clone();
  assert_eq!(exposed_ports.vhost.unwrap(), "{ vhost('your-vhost-name','a-zone') }");
  assert_eq!(exposed_ports.auth.unwrap(), "app-realm:admin:$1$EZsDrd93$7g2osLFOay4.TzDgGo9bF/");
  assert_eq!(exposed_ports.mode.unwrap(), "http");
  assert_eq!(exposed_ports.whitelist.unwrap(), "0.0.0.0 127.0.0.1");
  assert_eq!(exposed_ports.paths, vec!("/abc"));
  assert_eq!(exposed_ports.service_group.unwrap(), "mygroup");

  let health_check = config.application.health_check.clone().unwrap();
  assert_eq!(health_check.port, 8080);
  assert_eq!(health_check.protocol.unwrap(), HealthCheckProtocol::Http);
  assert_eq!(health_check.path, "/healthpath");

  let secret = config.application.secrets.clone().unwrap().first().unwrap().clone();
  assert_eq!(secret.name, "secret_name");
  assert_eq!(secret.injections.first().unwrap().get("env").unwrap(), "SECRET");
  assert_eq!(config.application.spread_group.clone().unwrap(), "SPREAD_GROUP");

  let volumes = config.application.volumes.clone().unwrap();
  assert_eq!(volumes.get("/volume_path").unwrap(), "{ volume('correct_volume_name') }");

  let deployment_parameters = config.deploy.as_ref().unwrap().parameters.clone().unwrap();
  assert_eq!(deployment_parameters.len(), 11);

  let environment_variables = config.application.environment_variables.clone().unwrap();
  assert_eq!(environment_variables.len(), 5);

  let env = environment_variables.get("APPLICATION_ENV_VAR1").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "bool1");
  assert!(env.value.is_none());

  let env = environment_variables.get("APPLICATION_ENV_VAR2").unwrap().clone();
  assert_eq!(env.typ, VariableType::InboundJunction);
  assert_eq!(env.key.unwrap(), "inbound-topic");
  assert!(env.value.is_none());

  let env = environment_variables.get("APPLICATION_ENV_VAR3").unwrap().clone();
  assert_eq!(env.typ, VariableType::OutboundJunction);
  assert_eq!(env.key.unwrap(), "outbound-topic");
  assert!(env.value.is_none());

  let env = environment_variables.get("APPLICATION_ENV_VAR4").unwrap().clone();
  assert_eq!(env.typ, VariableType::Template);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "value4${TENANT}");

  let env = environment_variables.get("APPLICATION_ENV_VAR5").unwrap().clone();
  assert_eq!(env.typ, VariableType::Value);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "value5");
}

#[test]
fn read_application_config_profile_proper_values() {
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/processors/applications/application-config-test.toml");
  let config = &read_application_config(path.to_str().unwrap()).unwrap();

  let profile1 = config.application.profiles.clone().get("profile-1").unwrap().clone();
  assert_eq!(profile1.profile_description, "Profile 1");
  assert_eq!(profile1.cpus, 1.0);
  assert_eq!(profile1.mem, 1);
  assert_eq!(profile1.instances, 1);

  let env1 = profile1.environment_variables.unwrap().clone();

  let env = env1.get("PROFILE1_ENV_VAR1").unwrap().clone();
  assert_eq!(env.typ, VariableType::DeploymentParameter);
  assert_eq!(env.key.unwrap(), "free1");
  assert!(env.value.is_none());

  let env = env1.get("PROFILE1_ENV_VAR2").unwrap().clone();
  assert_eq!(env.typ, VariableType::InboundJunction);
  assert_eq!(env.key.unwrap(), "inbound-topic");
  assert!(env.value.is_none());

  let env = env1.get("PROFILE1_ENV_VAR3").unwrap().clone();
  assert_eq!(env.typ, VariableType::OutboundJunction);
  assert_eq!(env.key.unwrap(), "outbound-topic");
  assert!(env.value.is_none());

  let env = env1.get("PROFILE1_ENV_VAR4").unwrap().clone();
  assert_eq!(env.typ, VariableType::Template);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "value14${TENANT}");

  let env = env1.get("PROFILE1_ENV_VAR5").unwrap().clone();
  assert_eq!(env.typ, VariableType::Value);
  assert!(env.key.is_none());
  assert_eq!(env.value.unwrap(), "value15");
}
