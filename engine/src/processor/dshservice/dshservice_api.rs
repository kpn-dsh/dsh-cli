use std::collections::HashMap;

use trifonius_dsh_api::types::{
  Application as ApiApplication, ApplicationSecret as ApiApplicationSecret, ApplicationVolumes as ApiApplicationVolumes, HealthCheck as ApiHealthCheck,
  HealthCheckProtocol as ApiHealthCheckProtocol, Metrics as ApiMetrics, PathSpec as ApiPathSpec, PortMapping as ApiPortMapping, PortMappingTls as ApiPortMappingTls,
};

use crate::engine_target::{template_resolver, TemplateMapping};
use crate::pipeline::PipelineId;
use crate::processor::dshservice::dshservice_config::{
  DshServiceSpecificConfig, HealthCheckConfig, HealthCheckProtocol, MetricsConfig, PortMappingConfig, PortMappingTls, ProfileConfig, SecretConfig,
};
use crate::processor::dshservice::DshServiceName;
use crate::processor::processor_config::VariableType;
use crate::processor::{JunctionId, ProcessorId, ProcessorTechnology};

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
      paths: value.paths.iter().map(|p| ApiPathSpec { prefix: p.to_string() }).collect(),
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
  pipeline_id: Option<&PipelineId>,
  processor_id: &ProcessorId,
  service_name: &DshServiceName,
  dshservice_specific_config: &DshServiceSpecificConfig,
  inbound_junctions: &HashMap<JunctionId, String>,
  outbound_junctions: &HashMap<JunctionId, String>,
  parameters: &HashMap<String, String>, // TODO ParameterId?
  profile: &ProfileConfig,
  user: String,
  template_mapping: &TemplateMapping,
) -> Result<ApiApplication, String> {
  let mut environment_variables: HashMap<String, String> = HashMap::new();
  if let Some(pipeline_id) = pipeline_id {
    environment_variables.insert("TRIFONIUS_PIPELINE_ID".to_string(), pipeline_id.to_string());
  }
  environment_variables.insert("TRIFONIUS_PROCESSOR_ID".to_string(), processor_id.to_string());
  environment_variables.insert("TRIFONIUS_PROCESSOR_TECHNOLOGY".to_string(), ProcessorTechnology::DshService.to_string());
  environment_variables.insert("TRIFONIUS_PROCESSOR_REALIZATION_ID".to_string(), "TODO".to_string());
  environment_variables.insert("TRIFONIUS_SERVICE_NAME".to_string(), service_name.to_string());
  if let Some(ref configured_environment_variables) = dshservice_specific_config.environment_variables {
    for (configured_environment_variable, variable) in configured_environment_variables.clone() {
      match variable.typ {
        VariableType::InboundJunction => match variable.id {
          Some(ref junction_id) => match inbound_junctions.get(&JunctionId::try_from(junction_id.as_str())?) {
            Some(parameter_value) => {
              environment_variables.insert(configured_environment_variable, parameter_value.to_string());
            }
            None => {
              return Err(format!(
                "missing inbound junction setting '{}' for variable '{}'",
                junction_id, configured_environment_variable
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::OutboundJunction => match variable.id {
          Some(ref junction_id) => match outbound_junctions.get(&JunctionId::try_from(junction_id.as_str())?) {
            Some(parameter_value) => {
              environment_variables.insert(configured_environment_variable, parameter_value.to_string());
            }
            None => {
              return Err(format!(
                "missing outbound junction setting '{}' for variable '{}'",
                junction_id, configured_environment_variable
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::DeploymentParameter => match variable.id {
          Some(ref deployment_parameter_id) => match parameters.get(deployment_parameter_id) {
            Some(parameter_value) => {
              environment_variables.insert(configured_environment_variable, parameter_value.clone());
            }
            None => {
              return Err(format!(
                "missing deployment parameter '{}' for variable '{}'",
                deployment_parameter_id, configured_environment_variable
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::Template => match variable.value {
          Some(template) => {
            let resolved = template_resolver(template.as_str(), template_mapping)?;
            environment_variables.insert(configured_environment_variable, resolved);
          }
          None => unreachable!(),
        },
        VariableType::Value => match variable.value {
          Some(parameter_value) => {
            environment_variables.insert(configured_environment_variable, parameter_value);
          }
          None => unreachable!(),
        },
      }
    }
  }

  let api_application = ApiApplication {
    cpus: profile.cpus,
    env: environment_variables,
    exposed_ports: match dshservice_specific_config.exposed_ports {
      Some(ref m) => m
        .iter()
        .map(|e| (e.0.clone(), Into::<ApiPortMapping>::into(e.1.clone())))
        .collect::<HashMap<String, ApiPortMapping>>(),
      None => HashMap::new(),
    },
    health_check: dshservice_specific_config.health_check.as_ref().map(|hc| ApiHealthCheck::from(hc.clone())),
    image: template_resolver(dshservice_specific_config.image.as_str(), template_mapping)?,
    instances: profile.instances,
    mem: profile.mem,
    metrics: dshservice_specific_config.metrics.clone().map(|m| m.into()),
    needs_token: dshservice_specific_config.needs_token,
    readable_streams: vec![],
    secrets: match dshservice_specific_config.secrets {
      Some(ref ss) => ss.iter().map(|s| Into::<ApiApplicationSecret>::into(s.clone())).collect(),
      None => vec![],
    },
    single_instance: dshservice_specific_config.single_instance,
    spread_group: dshservice_specific_config.spread_group.clone(),
    topics: vec![],
    user,
    volumes: match dshservice_specific_config.volumes {
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
