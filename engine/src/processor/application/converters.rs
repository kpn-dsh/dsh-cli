use std::collections::HashMap;

use dsh_rest_api_client::types::{
  Application as ApiApplication, ApplicationSecret as ApiApplicationSecret, ApplicationVolumes as ApiApplicationVolumes, HealthCheck as ApiHealthCheck,
  HealthCheckProtocol as ApiHealthCheckProtocol, Metrics as ApiMetrics, PathSpec, PortMapping as ApiPortMapping, PortMappingTls as ApiPortMappingTls,
};
use lazy_static::lazy_static;
use regex::Regex;

use crate::processor::application::application_config::{
  ApplicationConfig, HealthCheckConfig, HealthCheckProtocol, MetricsConfig, PortMappingConfig, PortMappingTls, ProfileConfig, SecretConfig,
};
use crate::processor::processor::ProcessorDeployParameters;
use crate::processor::processor_config::{PlaceHolder, VariableType};

impl From<ApiHealthCheck> for HealthCheckConfig {
  fn from(value: ApiHealthCheck) -> Self {
    Self { path: value.path, port: value.port, protocol: value.protocol.map(HealthCheckProtocol::from) }
  }
}

impl From<ApiHealthCheckProtocol> for HealthCheckProtocol {
  fn from(value: ApiHealthCheckProtocol) -> Self {
    match value {
      ApiHealthCheckProtocol::Http => HealthCheckProtocol::Http,
      ApiHealthCheckProtocol::Https => HealthCheckProtocol::Https,
    }
  }
}

impl From<ApiMetrics> for MetricsConfig {
  fn from(value: ApiMetrics) -> Self {
    Self { path: value.path, port: value.port }
  }
}

impl From<ApiPortMapping> for PortMappingConfig {
  fn from(value: ApiPortMapping) -> Self {
    Self {
      auth: value.auth,
      mode: value.mode,
      paths: value.paths.iter().map(|p| p.prefix.clone()).collect::<Vec<String>>(),
      service_group: value.service_group,
      tls: value.tls.map(PortMappingTls::from),
      vhost: value.vhost,
      whitelist: value.whitelist,
    }
  }
}

impl From<ApiPortMappingTls> for PortMappingTls {
  fn from(value: ApiPortMappingTls) -> Self {
    match value {
      ApiPortMappingTls::Auto => PortMappingTls::Auto,
      ApiPortMappingTls::None => PortMappingTls::None,
    }
  }
}

impl From<ApiApplicationSecret> for SecretConfig {
  fn from(value: ApiApplicationSecret) -> Self {
    Self { injections: value.injections, name: value.name }
  }
}

impl From<HealthCheckConfig> for ApiHealthCheck {
  fn from(val: HealthCheckConfig) -> Self {
    ApiHealthCheck { path: val.path, port: val.port, protocol: val.protocol.map(|p| p.into()) }
  }
}

impl From<HealthCheckProtocol> for ApiHealthCheckProtocol {
  fn from(val: HealthCheckProtocol) -> Self {
    match val {
      HealthCheckProtocol::Http => ApiHealthCheckProtocol::Http,
      HealthCheckProtocol::Https => ApiHealthCheckProtocol::Https,
    }
  }
}

impl From<MetricsConfig> for ApiMetrics {
  fn from(value: MetricsConfig) -> Self {
    ApiMetrics { path: value.path, port: value.port }
  }
}

impl From<PortMappingConfig> for ApiPortMapping {
  fn from(value: PortMappingConfig) -> Self {
    ApiPortMapping {
      auth: value.auth,
      mode: value.mode,
      paths: value.paths.iter().map(|p| PathSpec { prefix: p.to_string() }).collect(),
      service_group: value.service_group,
      tls: value.tls.map(|t| t.into()),
      vhost: value.vhost,
      whitelist: value.whitelist,
    }
  }
}

impl From<PortMappingTls> for ApiPortMappingTls {
  fn from(value: PortMappingTls) -> Self {
    match value {
      PortMappingTls::Auto => ApiPortMappingTls::Auto,
      PortMappingTls::None => ApiPortMappingTls::None,
    }
  }
}

impl From<SecretConfig> for ApiApplicationSecret {
  fn from(value: SecretConfig) -> Self {
    ApiApplicationSecret { injections: value.injections, name: value.name }
  }
}

pub fn into_api_application(
  application_config: &ApplicationConfig,
  deploy_parameters: &ProcessorDeployParameters,
  profile: &ProfileConfig,
  user: String,
  template_mappings: &HashMap<PlaceHolder, &str>,
) -> Result<ApiApplication, String> {
  let mut environment_variables: HashMap<String, String> = HashMap::new();
  if let Some(ref envs) = application_config.application.environment_variables {
    for (environment_variable_name, variable) in envs.clone() {
      match variable.typ {
        VariableType::InboundJunction => match variable.key {
          Some(ref junction_name) => match deploy_parameters.inbound_junctions.get(junction_name) {
            Some(parameter_value) => {
              environment_variables.insert(environment_variable_name, parameter_value.to_string());
            }
            None => {
              return Err(format!(
                "missing inbound junction setting '{}' for variable '{}'",
                junction_name, environment_variable_name
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::OutboundJunction => match variable.key {
          Some(ref junction_name) => match deploy_parameters.outbound_junctions.get(junction_name) {
            Some(parameter_value) => {
              environment_variables.insert(environment_variable_name, parameter_value.to_string());
            }
            None => {
              return Err(format!(
                "missing outbound junction setting '{}' for variable '{}'",
                junction_name, environment_variable_name
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::DeploymentParameter => match variable.key {
          Some(ref deployment_parameter_key) => match deploy_parameters.parameters.get(deployment_parameter_key) {
            Some(parameter_value) => {
              environment_variables.insert(environment_variable_name, parameter_value.clone());
            }
            None => {
              return Err(format!(
                "missing deployment parameter '{}' for variable '{}'",
                deployment_parameter_key, environment_variable_name
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::Template => match variable.value {
          Some(template) => {
            let resolved = template_resolver(template.as_str(), template_mappings)?;
            environment_variables.insert(environment_variable_name, resolved);
          }
          None => unreachable!(),
        },
        VariableType::Value => match variable.value {
          Some(parameter_value) => {
            environment_variables.insert(environment_variable_name, parameter_value);
          }
          None => unreachable!(),
        },
      }
    }
  }

  let api_application = ApiApplication {
    cpus: profile.cpus,
    env: environment_variables,
    exposed_ports: match application_config.application.exposed_ports {
      Some(ref m) => m
        .iter()
        .map(|e| (e.0.clone(), Into::<ApiPortMapping>::into(e.1.clone())))
        .collect::<HashMap<String, ApiPortMapping>>(),
      None => HashMap::new(),
    },
    health_check: application_config.application.health_check.as_ref().map(|hc| ApiHealthCheck::from(hc.clone())),
    image: application_config.application.image.clone(),
    instances: profile.instances,
    mem: profile.mem,
    metrics: application_config.application.metrics.clone().map(|m| m.into()),
    needs_token: application_config.application.needs_token,
    readable_streams: vec![],
    secrets: match application_config.application.secrets {
      Some(ref ss) => ss.iter().map(|s| Into::<ApiApplicationSecret>::into(s.clone())).collect(),
      None => vec![],
    },
    single_instance: application_config.application.single_instance,
    spread_group: application_config.application.spread_group.clone(),
    topics: vec![],
    user,
    volumes: match application_config.application.volumes {
      Some(ref vs) => vs
        .iter()
        .map(|e| (e.0.clone(), ApiApplicationVolumes { name: e.1.clone() }))
        .collect::<HashMap<String, ApiApplicationVolumes>>(),
      None => HashMap::new(),
    },
    writable_streams: vec![],
  };
  Ok(api_application)
}

lazy_static! {
  static ref TEMPLATE_REGEX: Regex = Regex::new("\\$\\{([A-Z][A-Z0-9_]*)\\}").unwrap();
}

pub(crate) fn template_resolver(template: &str, template_mapping: &HashMap<PlaceHolder, &str>) -> Result<String, String> {
  let mut new = String::with_capacity(template.len());
  let mut last_match = 0;
  for caps in TEMPLATE_REGEX.captures_iter(template) {
    let m = caps.get(0).unwrap();
    new.push_str(&template[last_match..m.start()]);
    let place_holder = PlaceHolder::try_from(caps.get(1).unwrap().as_str())?;
    match template_mapping.get(&place_holder) {
      Some(value) => {
        new.push_str(value);
      }
      None => return Err(format!("template resolution failed because placeholder '{}' has no value", place_holder)),
    }
    last_match = m.end();
  }
  new.push_str(&template[last_match..]);
  Ok(new)
}

pub(crate) fn validate_template(template: &str, template_mapping: &[PlaceHolder]) -> Result<(), String> {
  for caps in TEMPLATE_REGEX.captures_iter(template) {
    // let m = caps.get(0).unwrap();
    let place_holder = PlaceHolder::try_from(caps.get(1).unwrap().as_str())?;
    if !template_mapping.contains(&place_holder) {
      return Err(format!("invalid template because placeholder '{}' is not allowed", place_holder));
    }
  }
  Ok(())
}

#[test]
fn resolve_template_successfully() {
  let template = "abcd${TENANT}def${USER}ghi";
  let tenant = "tenant";
  let user = "user";
  let template_mapping = HashMap::from([(PlaceHolder::TENANT, tenant), (PlaceHolder::USER, user)]);
  assert_eq!(template_resolver(template, &template_mapping).unwrap(), "abcdtenantdefuserghi");
}

#[test]
fn validate_template_succesfully() {
  assert!(validate_template("abcd${TENANT}def${USER}ghi", &[PlaceHolder::TENANT, PlaceHolder::USER]).is_ok());
  assert!(validate_template("abcd${TENANT}def${USER}ghi", &[PlaceHolder::TENANT]).is_err());
  assert!(validate_template("abcd{TENANT}def{USER}ghi", &[PlaceHolder::TENANT]).is_ok());
  assert!(validate_template("abcdefghijkl", &[PlaceHolder::TENANT]).is_ok());
  assert!(validate_template("", &[PlaceHolder::TENANT]).is_ok());
}
