use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::processor::application::application_config::ProfileConfig;
use crate::processor::processor_config::{DeploymentParameterConfig, JunctionConfig};
use crate::processor::{DeploymentParameterType, ProcessorType};
use crate::resource::ResourceType;

/// Describes a `Processor`
///
/// A `ProcessorDescriptor` describes some generic details of a concrete `Processor`, e.g. the
/// `Processor`s name, description and version, a list of required junctions and some generic
/// parameters. A `ProcessorDescriptor` can be seen as the abstract _super type_ of all concrete
/// `Processor`s descriptors. A `Processor` can be used by a control application (cli or gui) to
/// present the `Processor` to the user and to determine which parameters the user needs to provide
/// to deploy a `Processor`.
#[derive(Clone, Debug, Serialize)]
pub struct ProcessorDescriptor {
  #[serde(rename = "type")]
  pub processor_type: ProcessorType,
  pub name: String,
  pub description: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
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
  pub name: String,
  pub caption: String,
  pub description: Option<String>,
  pub allowed_resource_types: Vec<ResourceType>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeploymentParameterDescriptor {
  #[serde(rename = "type")]
  pub parameter_typ: DeploymentParameterType,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(rename = "initial-value", skip_serializing_if = "Option::is_none")]
  pub initial_value: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub options: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub optional: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub default: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProfileDescriptor {
  pub name: String,
  pub description: Option<String>,
  pub instances: Option<u64>,
  pub cpus: Option<f64>,
  pub mem: Option<u64>,
}

impl From<(String, JunctionConfig)> for JunctionDescriptor {
  fn from((name, config): (String, JunctionConfig)) -> Self {
    JunctionDescriptor { name, caption: config.caption, description: Some(config.description), allowed_resource_types: config.allowed_resource_types }
  }
}

impl From<(&String, &JunctionConfig)> for JunctionDescriptor {
  fn from((name, config): (&String, &JunctionConfig)) -> Self {
    let c = config.clone();
    JunctionDescriptor { name: name.to_owned(), caption: c.caption, description: Some(c.description), allowed_resource_types: c.allowed_resource_types }
  }
}

impl From<(String, DeploymentParameterConfig)> for DeploymentParameterDescriptor {
  fn from((name, config): (String, DeploymentParameterConfig)) -> Self {
    DeploymentParameterDescriptor {
      parameter_typ: config.typ,
      name,
      description: config.description,
      initial_value: config.initial_value,
      options: config.options,
      optional: config.optional,
      default: config.default,
    }
  }
}

impl From<(String, ProfileConfig)> for ProfileDescriptor {
  fn from((name, config): (String, ProfileConfig)) -> Self {
    ProfileDescriptor { name, description: Some(config.profile_description), instances: Some(config.instances), cpus: Some(config.cpus), mem: Some(config.mem) }
  }
}

impl From<(&String, &ProfileConfig)> for ProfileDescriptor {
  fn from((name, config): (&String, &ProfileConfig)) -> Self {
    let c = config.clone();
    ProfileDescriptor { name: name.to_owned(), description: Some(c.profile_description), instances: Some(c.instances), cpus: Some(c.cpus), mem: Some(c.mem) }
  }
}

impl Display for ProcessorDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(ref version) = self.version {
      write!(f, "{}:{}", self.name, version)?;
    } else {
      write!(f, "{}", self.name)?;
    }
    write!(f, "\n  {}", self.processor_type)?;
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

impl Display for DeploymentParameterDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", &self.name, &self.parameter_typ)?;
    if let Some(description) = &self.description {
      write!(f, " ({})", description)?;
    }
    if let Some(initial_value) = &self.initial_value {
      write!(f, ", initial value: {}", initial_value)?;
    }
    if let Some(options) = &self.options {
      write!(f, ", options: [{}]", options.join(","))?;
    }
    if let Some(optional) = &self.optional {
      write!(f, ", optional: {}", optional)?;
    }
    if let Some(default) = &self.default {
      write!(f, ", default: {}", default)?;
    }
    Ok(())
  }
}

impl Display for JunctionDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self.name)?;
    if let Some(description) = &self.description {
      write!(f, " ({})", description)?;
    }
    write!(
      f,
      ", allowed resource types: {}",
      &self.allowed_resource_types.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
    )?;
    Ok(())
  }
}

impl Display for ProfileDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", &self.name)?;
    if let Some(description) = &self.description {
      write!(f, " ({})", description)?;
    }
    if let Some(instances) = &self.instances {
      write!(f, ", instances: {}", instances)?;
    }
    if let Some(cpus) = &self.cpus {
      write!(f, ", cpus: {}", cpus)?;
    }
    if let Some(mem) = &self.mem {
      write!(f, ", mem: {}", mem)?;
    }
    Ok(())
  }
}
