use std::collections::HashMap;

use dsh_rest_api_client::types::{
  Application as ApiApplication, ApplicationSecret as ApiApplicationSecret, ApplicationVolumes as ApiApplicationVolumes, HealthCheck as ApiHealthCheck,
  HealthCheckProtocol as ApiHealthCheckProtocol, Metrics as ApiMetrics, PathSpec, PortMapping as ApiPortMapping, PortMappingTls as ApiPortMappingTls,
};
use lazy_static::lazy_static;
use regex::Regex;

use crate::application::application_config::{
  ApplicationConfig, ApplicationSecret, HealthCheck, HealthCheckProtocol, Metrics, PlaceHolder, PortMapping, PortMappingTls, Profile, VariableType,
};

impl From<ApiHealthCheck> for HealthCheck {
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

impl From<ApiMetrics> for Metrics {
  fn from(value: ApiMetrics) -> Self {
    Self { path: value.path, port: value.port }
  }
}

impl From<ApiPortMapping> for PortMapping {
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

impl From<ApiApplicationSecret> for ApplicationSecret {
  fn from(value: ApiApplicationSecret) -> Self {
    Self { injections: value.injections, name: value.name }
  }
}

impl From<HealthCheck> for ApiHealthCheck {
  fn from(val: HealthCheck) -> Self {
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

impl From<Metrics> for ApiMetrics {
  fn from(value: Metrics) -> Self {
    ApiMetrics { path: value.path, port: value.port }
  }
}

impl From<PortMapping> for ApiPortMapping {
  fn from(value: PortMapping) -> Self {
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

impl From<ApplicationSecret> for ApiApplicationSecret {
  fn from(value: ApplicationSecret) -> Self {
    ApiApplicationSecret { injections: value.injections, name: value.name }
  }
}

pub fn api_application(
  application_config: &ApplicationConfig,
  deployment_parameter_values: &HashMap<String, String>,
  profile: &Profile,
  user: String,
  template_mappings: &HashMap<PlaceHolder, &String>,
) -> Result<ApiApplication, String> {
  let mut environment_variables: HashMap<String, String> = HashMap::new();
  if let Some(ref envs) = application_config.environment_variables {
    for (variable_name, variable) in envs.clone() {
      match variable.typ {
        VariableType::DeploymentParameter => match variable.key {
          Some(ref deployment_parameter_key) => match deployment_parameter_values.get(deployment_parameter_key) {
            Some(parameter_value) => {
              environment_variables.insert(variable_name, parameter_value.clone());
            }
            None => {
              return Err(format!(
                "missing deployment parameter '{}' for variable '{}'",
                deployment_parameter_key, variable_name
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::Template => match variable.value {
          Some(template) => {
            let resolved = template_resolver(template.as_str(), template_mappings)?;
            environment_variables.insert(variable_name, resolved);
          }
          None => unreachable!(),
        },
        VariableType::Value => match variable.value {
          Some(parameter_value) => {
            environment_variables.insert(variable_name, parameter_value);
          }
          None => unreachable!(),
        },
      }
    }
  }

  let api_application = ApiApplication {
    cpus: profile.cpus,
    env: environment_variables,
    exposed_ports: match application_config.exposed_ports {
      Some(ref m) => m
        .iter()
        .map(|e| (e.0.clone(), Into::<ApiPortMapping>::into(e.1.clone())))
        .collect::<HashMap<String, ApiPortMapping>>(),
      None => HashMap::new(),
    },
    health_check: application_config.health_check.as_ref().map(|hc| ApiHealthCheck::from(hc.clone())),
    // health_check: match application_config.health_check {
    //   Some(ref hc) => Some(Into::<ApiHealthCheck>::into(hc.clone())),
    //   None => None,
    // },
    image: application_config.image.clone(),
    instances: profile.instances,
    mem: profile.mem,
    metrics: application_config.metrics.clone().map(|m| m.into()),
    needs_token: application_config.needs_token,
    readable_streams: vec![],
    secrets: match application_config.secrets {
      Some(ref ss) => ss.iter().map(|s| Into::<ApiApplicationSecret>::into(s.clone())).collect(),
      None => vec![],
    },
    single_instance: application_config.single_instance,
    spread_group: application_config.spread_group.clone(),
    topics: vec![],
    user,
    volumes: match application_config.volumes {
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

fn template_resolver(template: &str, template_mapping: &HashMap<PlaceHolder, &String>) -> Result<String, String> {
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

#[test]
fn resolve_template_successfully() {
  let template = "abcd${TENANT}def${USER}ghi";
  let tenant = "tenant".to_string();
  let user = "user".to_string();
  let template_mapping = HashMap::from([(PlaceHolder::TENANT, &tenant), (PlaceHolder::USER, &user)]);
  assert_eq!(template_resolver(template, &template_mapping).unwrap(), "abcdtenantdefuserghi");
}
