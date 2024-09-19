use std::collections::HashMap;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::engine_target::validate_template;
use crate::placeholder::PlaceHolder;
use crate::read_config;
use crate::version::Version;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineConfig {
  #[serde(rename = "pipeline-id")]
  pub pipeline_id: String,
  #[serde(deserialize_with = "crate::version::deserialize_version_from_representation", serialize_with = "crate::version::serialize_version_to_representation")]
  pub version: Version,
  pub name: String,
  pub description: String,
  pub icon: Option<String>,
  pub tags: Option<Vec<String>>,
  #[serde(rename = "more-info-url")]
  pub more_info_url: Option<String>,
  #[serde(rename = "metrics-url")]
  pub metrics_url: Option<String>,
  #[serde(rename = "viewer-url")]
  pub viewer_url: Option<String>,
  pub metadata: Option<Vec<(String, String)>>,
  pub resources: Vec<PipelineResource>,
  pub processors: Vec<PipelineProcessor>,
  pub connections: Vec<PipelineConnection>,
  pub dependencies: Vec<PipelineDependency>,
  pub profiles: Vec<PipelineProfile>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineResource {
  #[serde(rename = "resource-id")]
  pub resource_id: String,
  #[serde(rename = "resource-realization")]
  pub resource_realization: String,
  pub parameters: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineProcessor {
  #[serde(rename = "processor-id")]
  pub processor_id: String,
  #[serde(rename = "processor-realization")]
  pub processor_realization: String,
  pub parameters: HashMap<String, String>,
  #[serde(rename = "profile-id")]
  pub profile_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PipelineConnection {
  ResourceToProcessor { source: Vec<String>, target: ProcessorJunction },
  ProcessorToResource { source: ProcessorJunction, target: Vec<String> },
  ProcessorToProcessor { source: ProcessorJunction, target: ProcessorJunction },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProcessorJunction {
  #[serde(rename = "processor-id")]
  pub processor_id: String,
  pub junction: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineDependency {
  pub parameters: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PipelineProfile {
  pub parameters: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DependencyType {
  ProcessorOnProcessor { depended: String, depends_on: String },
  ProcessorOnResource { depended: String, depends_on: Vec<String> },
  ResourceOnProcessor { depended: String, depends_on: String },
}

impl PipelineConfig {
  fn validate(&self) -> Result<(), String> {
    if self.description.is_empty() {
      return Err("pipeline description cannot be empty".to_string());
    }
    if let Some(ref url) = self.more_info_url {
      validate_config_template(url, "more-info-url template")?
    }
    if let Some(ref url) = self.metrics_url {
      validate_config_template(url, "metrics-url template")?
    }
    if let Some(ref url) = self.viewer_url {
      validate_config_template(url, "viewer-url template")?
    }
    Ok(())
  }
}

pub fn read_pipeline_config(config_file_name: &str) -> Result<PipelineConfig, String> {
  debug!("read pipeline config file: {}", config_file_name);
  let pipeline_config = read_config::<PipelineConfig>(config_file_name, "pipeline")?;
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

#[test]
fn read_dshservice_config_json() {
  test_config("pipeline-config-test.json");
}

#[test]
fn read_dshservice_config_toml() {
  test_config("pipeline-config-test.toml");
}

#[test]
fn read_dshservice_config_yaml() {
  test_config("pipeline-config-test.yaml");
}

#[cfg(test)]
fn test_config(config_file_name: &str) {
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push(format!("tests/pipelines/{}", config_file_name));
  let config = &read_pipeline_config(path.to_str().unwrap()).unwrap();
  assert_eq!(config.pipeline_id, "test");
  assert_eq!(config.version, Version::new(1, 2, 3));
  assert_eq!(config.name, "Test pipeline");
  assert_eq!(config.description, "Test pipeline description");
  assert_eq!(config.icon, Some("ICON".to_string()));
  assert_eq!(config.tags, Some(vec!["TAG1".to_string(), "TAG2".to_string()]));
  assert_eq!(config.more_info_url, Some("https://dsh.kpn.com".to_string()));
  assert_eq!(config.metrics_url, Some("https://grafana.com".to_string()));
  assert_eq!(config.viewer_url, Some("https://eavesdropper.kpn.com".to_string()));
}

#[test]
fn read_dshservice_config_compare_formats() {
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/pipelines/pipeline-config-test.json");
  let config_json = &read_pipeline_config(path.to_str().unwrap()).unwrap();
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/pipelines/pipeline-config-test.toml");
  let config_toml = &read_pipeline_config(path.to_str().unwrap()).unwrap();
  let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  path.push("tests/pipelines/pipeline-config-test.yaml");
  let config_yaml = &read_pipeline_config(path.to_str().unwrap()).unwrap();
  assert_eq!(config_json, config_toml);
  assert_eq!(config_toml, config_yaml);
}
