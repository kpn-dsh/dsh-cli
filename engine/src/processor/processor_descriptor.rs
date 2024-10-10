use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::processor::processor_config::{DeploymentParameterConfig, DeploymentParameterConfigOption, DeploymentParameterType, JunctionConfig};
use crate::processor::{JunctionDirection, JunctionId, JunctionTechnology, ParameterId, ProcessorId, ProcessorTechnology};
use crate::resource::ResourceTechnology;
use crate::version::Version;
use crate::ProfileId;

/// Describes a `Processor`
///
/// A `ProcessorDescriptor` describes some generic details of a concrete `Processor`, e.g. the
/// `Processor`s id, label, description and version, a list of required junctions and some generic
/// parameters. A `ProcessorDescriptor` can be seen as the abstract _super type_ of all concrete
/// `Processor`s descriptors. A `Processor` can be used by a control application (cli or gui) to
/// present the `Processor` to the user and to determine which parameters the user needs to provide
/// to deploy a `Processor`.
#[derive(Clone, Debug, Serialize)]
pub struct ProcessorDescriptor {
  #[serde(rename = "technology")]
  pub technology: ProcessorTechnology,
  pub id: ProcessorId,
  pub label: String,
  pub description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<Version>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub icon: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub tags: Vec<String>,
  #[serde(rename = "inbound-junctions", skip_serializing_if = "Vec::is_empty")]
  pub inbound_junctions: Vec<JunctionDescriptor>,
  #[serde(rename = "outbound-junctions", skip_serializing_if = "Vec::is_empty")]
  pub outbound_junctions: Vec<JunctionDescriptor>,
  #[serde(rename = "deployment-parameters", skip_serializing_if = "Vec::is_empty")]
  pub deployment_parameters: Vec<DeploymentParameterDescriptor>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub profiles: Vec<ProfileDescriptor>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub metadata: Vec<(String, String)>,
  #[serde(rename = "more-info-url", skip_serializing_if = "Option::is_none")]
  pub more_info_url: Option<String>,
  #[serde(rename = "metrics-url", skip_serializing_if = "Option::is_none")]
  pub metrics_url: Option<String>,
  #[serde(rename = "viewer-url", skip_serializing_if = "Option::is_none")]
  pub viewer_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JunctionDescriptor {
  pub id: JunctionId,
  pub direction: JunctionDirection,
  #[serde(rename = "junction-technology")]
  pub junction_technology: JunctionTechnology,
  pub label: String,
  pub description: String,
  #[serde(rename = "minimum-number-of-connections")]
  pub minimum_number_of_connections: u32,
  #[serde(rename = "maximum-number-of-connections")]
  pub maximum_number_of_connections: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeploymentParameterDescriptor {
  #[serde(rename = "type")]
  pub parameter_type: DeploymentParameterType,
  pub id: ParameterId,
  pub label: String,
  pub description: String,
  pub optional: bool,
  #[serde(rename = "initial-value", skip_serializing_if = "Option::is_none")]
  pub initial_value: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<Vec<DeploymentParameterOptionDescriptor>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeploymentParameterOptionDescriptor {
  pub id: String,
  pub label: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProfileDescriptor {
  pub profile_id: ProfileId, // TODO id
  pub label: String,
  pub description: String,
  pub instances: Option<u64>,
  pub cpus: Option<f64>,
  pub mem: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProcessorTypeDescriptor {
  #[serde(rename = "processor-technology")]
  pub processor_technology: ProcessorTechnology,
  pub label: String,
  pub description: String,
}

impl JunctionDescriptor {
  pub(crate) fn is_resource_technology_compatible(&self, resource_technology: &ResourceTechnology) -> bool {
    match self.junction_technology {
      JunctionTechnology::DshTopic => resource_technology == &ResourceTechnology::DshTopic,
      JunctionTechnology::Grpc => false,
    }
  }
}

impl From<&ProcessorTechnology> for ProcessorTypeDescriptor {
  fn from(processor_technology: &ProcessorTechnology) -> Self {
    ProcessorTypeDescriptor {
      processor_technology: processor_technology.clone(),
      label: processor_technology.label().to_string(),
      description: processor_technology.description().to_string(),
    }
  }
}

impl Display for ProcessorDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{} ({})", self.id, self.technology, self.label)?;
    if let Some(ref version) = self.version {
      write!(f, "\n  version {}", version)?;
    }
    if let Some(ref icon) = self.icon {
      write!(f, "\n  icon {}", icon)?;
    }
    write!(f, "\n  {}", self.description)?;
    if !&self.inbound_junctions.is_empty() {
      write!(f, "\n  inbound junctions")?;
      for inbound_junction in &self.inbound_junctions {
        write!(f, "\n    {}", inbound_junction)?
      }
    }
    if !&self.outbound_junctions.is_empty() {
      write!(f, "\n  outbound junctions")?;
      for outbound_junction in &self.outbound_junctions {
        write!(f, "\n    {}", outbound_junction)?
      }
    }
    if !&self.deployment_parameters.is_empty() {
      write!(f, "\n  deployment parameters")?;
      for deployment_parameter in &self.deployment_parameters {
        write!(f, "\n    {}", deployment_parameter)?
      }
    }
    if !&self.profiles.is_empty() {
      write!(f, "\n  profiles")?;
      for profile in &self.profiles {
        write!(f, "\n    {}", profile)?
      }
    }
    if !&self.metadata.is_empty() {
      write!(f, "\n  metadata")?;
      for (key, value) in &self.metadata {
        write!(f, "\n    {}: {}", key, value)?
      }
    }
    if let Some(ref url) = self.more_info_url {
      write!(f, "\n  more info url: {}", url)?
    }
    if let Some(ref url) = self.metrics_url {
      write!(f, "\n  metrics url: {}", url)?
    }
    if let Some(ref url) = self.viewer_url {
      write!(f, "\n  viewer url: {}", url)?
    }
    Ok(())
  }
}

impl Display for JunctionDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}:{} ({}", self.id, self.direction, self.junction_technology, self.label)?;
    if self.minimum_number_of_connections == self.maximum_number_of_connections {
      if self.minimum_number_of_connections == 1 {
        write!(f, ", 1 resource")?
      } else {
        write!(f, ", {} resources", self.minimum_number_of_connections)?
      }
    } else {
      write!(f, ", {}-{} resources", self.minimum_number_of_connections, self.maximum_number_of_connections)?
    }
    write!(f, ", {}", self.description)?;
    write!(f, ")")
  }
}

impl Display for DeploymentParameterDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{} ({}, {}", self.id, self.parameter_type, self.label, self.description)?;
    if let Some(ref initial_value) = self.initial_value {
      write!(f, ", initial value: {}", initial_value)?;
    }
    if let Some(options) = &self.options {
      write!(f, ", options: [{}]", options.iter().map(|opt| opt.to_string()).collect::<Vec<String>>().join(","))?;
    }
    if self.optional {
      write!(f, ", optional")?;
    } else {
      write!(f, ", mandatory")?;
    }
    if let Some(default) = &self.default {
      write!(f, ", default: {}", default)?;
    }
    write!(f, ")")
  }
}

impl Display for ProfileDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} ({}, {}", self.profile_id, self.label, self.description)?;
    if let Some(instances) = &self.instances {
      write!(f, ", instances: {}", instances)?;
    }
    if let Some(cpus) = &self.cpus {
      write!(f, ", cpus: {}", cpus)?;
    }
    if let Some(mem) = &self.mem {
      write!(f, ", mem: {}", mem)?;
    }
    write!(f, ")")
  }
}

impl Display for DeploymentParameterOptionDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.description {
      Some(ref description) => write!(f, "{} ({}, {})", self.id, self.label, description),
      None => write!(f, "{} ({})", self.id, self.label),
    }
  }
}

impl From<(String, JunctionDirection, JunctionConfig)> for JunctionDescriptor {
  fn from((id, direction, config): (String, JunctionDirection, JunctionConfig)) -> Self {
    JunctionDescriptor::from((&id, &direction, &config))
  }
}

impl From<(&String, &JunctionDirection, &JunctionConfig)> for JunctionDescriptor {
  fn from((id, direction, config): (&String, &JunctionDirection, &JunctionConfig)) -> Self {
    let (min, max) = match (config.minimum_number_of_connections, config.maximum_number_of_connections) {
      (None, None) => (1, 1),
      (None, Some(max)) => (1, max),
      (Some(min), None) => (min, u32::MAX),
      (Some(min), Some(max)) => (min, max),
    };
    JunctionDescriptor {
      id: JunctionId(id.to_owned()), // TODO
      direction: direction.clone(),
      junction_technology: config.junction_technology.clone(),
      label: config.label.clone(),
      description: config.description.clone(),
      minimum_number_of_connections: min,
      maximum_number_of_connections: max,
    }
  }
}

impl From<&DeploymentParameterConfigOption> for DeploymentParameterOptionDescriptor {
  fn from(option_config: &DeploymentParameterConfigOption) -> Self {
    match option_config {
      DeploymentParameterConfigOption::Label(label) => {
        DeploymentParameterOptionDescriptor { id: label.id.clone(), label: label.label.clone(), description: label.description.clone() }
      }
      DeploymentParameterConfigOption::Id(id) => DeploymentParameterOptionDescriptor { id: id.clone(), label: id.clone(), description: None },
    }
  }
}

impl From<(String, &DeploymentParameterConfig)> for DeploymentParameterDescriptor {
  fn from((id, config): (String, &DeploymentParameterConfig)) -> Self {
    DeploymentParameterDescriptor {
      parameter_type: config.typ.clone(),
      id: ParameterId(id), // TODO
      label: config.label.to_string(),
      description: config.description.to_string(),
      initial_value: config.initial_value.clone(),
      options: config
        .options
        .as_ref()
        .map(|opts| opts.iter().map(DeploymentParameterOptionDescriptor::from).collect()),
      optional: config.optional.unwrap_or(false),
      default: config.default.clone(),
    }
  }
}
