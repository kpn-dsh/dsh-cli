use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::engine_target::validate_template;
use crate::pipeline::PipelineId;
use crate::placeholder::PlaceHolder;
use crate::processor::{JunctionId, ParameterId, ProcessorId, ProcessorRealizationId};
use crate::resource::{ResourceId, ResourceRealizationId};
use crate::version::Version;
use crate::{read_config_file, ProfileId};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineConfig {
  #[serde(rename = "pipeline-id")]
  pub pipeline_id: PipelineId,
  pub version: Version,
  pub name: String,
  pub description: String,
  pub resources: Vec<PipelineResourceConfig>,
  pub processors: Vec<PipelineProcessorConfig>,
  pub connections: Vec<PipelineConnectionConfig>,
  pub dependencies: Vec<PipelineDependencyConfig>,
  pub profiles: Vec<PipelineProfileConfig>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineResourceConfig {
  #[serde(rename = "resource-id")]
  pub resource_id: ResourceId,
  pub name: String,
  #[serde(rename = "resource-realization")]
  pub resource_realization_id: ResourceRealizationId,
  #[serde(rename = "resource-realization-version", skip_serializing_if = "Option::is_none")]
  pub resource_realization_version: Option<Version>,
  pub parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineProcessorConfig {
  #[serde(rename = "processor-id")]
  pub processor_id: ProcessorId,
  pub name: String,
  #[serde(rename = "processor-realization")]
  pub processor_realization_id: ProcessorRealizationId,
  #[serde(rename = "processor-realization-version")]
  pub processor_realization_version: Version,
  pub parameters: HashMap<ParameterId, String>,
  #[serde(rename = "profile-id")]
  pub profile_id: Option<ProfileId>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PipelineConnectionConfig {
  ResourcesToProcessor {
    #[serde(rename = "source-resources")]
    source_resource_ids: Vec<ResourceId>,
    #[serde(rename = "target-processor")]
    target_processor_junction: ProcessorJunctionConfig,
  },
  ProcessorToResources {
    #[serde(rename = "source-processor")]
    source_processor_junction: ProcessorJunctionConfig,
    #[serde(rename = "target-resources")]
    target_resource_ids: Vec<ResourceId>,
  },
  ProcessorToProcessor {
    #[serde(rename = "source-processor")]
    source_processor_junction: ProcessorJunctionConfig,
    #[serde(rename = "target-processor")]
    target_processor_junction: ProcessorJunctionConfig,
  },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProcessorJunctionConfig {
  #[serde(rename = "processor-id")]
  pub processor_id: ProcessorId,
  pub junction: JunctionId,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineDependencyConfig {
  pub parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineProfileConfig {
  pub parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DependencyType {
  ProcessorOnProcessor { depended: ProcessorId, depends_on: ProcessorId },
  ProcessorOnResource { depended: ProcessorId, depends_on: Vec<ResourceId> },
  ResourceOnProcessor { depended: ResourceId, depends_on: ProcessorId },
}

impl PipelineConfig {
  fn validate(&self) -> Result<(), String> {
    if self.description.is_empty() {
      return Err("pipeline description cannot be empty".to_string());
    }
    Ok(())
  }

  pub fn add_resource_config(&mut self, resource_config: PipelineResourceConfig) -> () {
    self.resources.push(resource_config);
  }

  pub fn add_processor_config(&mut self, processor_config: PipelineProcessorConfig) -> () {
    self.processors.push(processor_config);
  }

  pub fn add_connection_config(&mut self, connection_config: PipelineConnectionConfig) -> () {
    self.connections.push(connection_config)
  }
}

impl Display for PipelineResourceConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self.resource_realization_version {
      Some(version) => write!(f, "{}:{}:{}", self.resource_id, self.resource_realization_id, version),
      None => write!(f, "{}:{}", self.resource_id, self.resource_realization_id),
    }
  }
}

impl Display for PipelineProcessorConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}:{}", self.processor_id, self.processor_realization_id, self.processor_realization_version)
  }
}

impl Display for PipelineConnectionConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PipelineConnectionConfig::ResourcesToProcessor { source_resource_ids: sources, target_processor_junction: target } => {
        write!(f, "{} -> {}", sources.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","), target)
      }
      PipelineConnectionConfig::ProcessorToResources { source_processor_junction: source, target_resource_ids: targets } => {
        write!(f, "{} -> {}", source, targets.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","))
      }
      PipelineConnectionConfig::ProcessorToProcessor { source_processor_junction: source, target_processor_junction: target } => write!(f, "{} -> {}", source, target),
    }
  }
}

impl Display for ProcessorJunctionConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.processor_id, self.junction)
  }
}

pub fn read_pipeline_config(config_file_name: &str) -> Result<PipelineConfig, String> {
  debug!("read pipeline config file: {}", config_file_name);
  let pipeline_config = read_config_file::<PipelineConfig>(config_file_name, "pipeline")?;
  debug!("successfully read and parsed pipeline config file\n{:#?}", pipeline_config);
  pipeline_config.validate()?;
  debug!("successfully validated config");
  Ok(pipeline_config)
}

fn validate_config_template(template: &str, template_id: &str) -> Result<(), String> {
  static VALID_PLACEHOLDERS: [PlaceHolder; 10] = [
    PlaceHolder::AppDomain,
    PlaceHolder::ConsoleUrl,
    PlaceHolder::MonitoringUrl,
    PlaceHolder::Platform,
    PlaceHolder::Realm,
    PlaceHolder::RestAccessTokenUrl,
    PlaceHolder::RestApiUrl,
    PlaceHolder::Tenant,
    PlaceHolder::User,
    PlaceHolder::PublicVhostsDomain,
  ];
  if template.is_empty() {
    return Err(format!("{} cannot be empty", template_id));
  }
  validate_template(template, &VALID_PLACEHOLDERS).map_err(|message| format!("{} has {}", template_id, message))
}

fn print_config(config: &PipelineConfig) {
  println!("{}:{} '{}'", config.pipeline_id, config.version, config.name);
  if !config.resources.is_empty() {
    println!("resources");
    for resource in &config.resources {
      println!("  {}", resource);
    }
  }
  if !config.processors.is_empty() {
    println!("processors");
    for processor in &config.processors {
      println!("  {}", processor);
    }
  }
  if !config.connections.is_empty() {
    println!("connections");
    for connection in &config.connections {
      println!("  {}", connection);
    }
  }
  // if !config.dependencies.is_empty() {
  //   println!("dependencies");
  //   for dependency in &config.dependencies {
  //     println!("  {}", dependency);
  //   }
  // }
  // if !config.profiles.is_empty() {
  //   println!("profiles");
  //   for profile in &config.profiles {
  //     println!("  {}", profile);
  //   }
  // }
}

// #[test]
// fn read_dshservice_config_json() {
//   test_config("pipeline-config-test.json");
// }

#[test]
fn read_dshservice_config_toml() {
  test_config("pipeline-config-test-1.toml");
}

// #[test]
// fn read_dshservice_config_yaml() {
//   test_config("pipeline-config-test.yaml");
// }

#[cfg(test)]
fn test_config(config_file_name: &str) {
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push(format!("tests/pipelines/{}", config_file_name));
  let config = &read_pipeline_config(path.to_str().unwrap()).unwrap();
  print_config(config);
  assert_eq!(config.pipeline_id, PipelineId::new("pipeline1"));
  assert_eq!(config.version, Version::new(1, 2, 3));
  assert_eq!(config.name, "Test pipeline");
  assert_eq!(config.description, "Test pipeline description");
}

// #[test]
// fn read_dshservice_config_compare_formats() {
//   let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//   path.push("tests/pipelines/pipeline-config-test.json");
//   let config_json = &read_pipeline_config(path.to_str().unwrap()).unwrap();
//   let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//   path.push("tests/pipelines/pipeline-config-test.toml");
//   let config_toml = &read_pipeline_config(path.to_str().unwrap()).unwrap();
//   let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//   path.push("tests/pipelines/pipeline-config-test.yaml");
//   let config_yaml = &read_pipeline_config(path.to_str().unwrap()).unwrap();
//   assert_eq!(config_json, config_toml);
//   assert_eq!(config_toml, config_yaml);
// }
