use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::processor::application::application_config::DeploymentParameter;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationDescriptor {
  #[serde(rename = "application-name")]
  pub application_name: String,
  #[serde(rename = "application-description")]
  pub application_description: String,
  #[serde(rename = "application-version")]
  pub application_version: String,
  #[serde(rename = "grafana-url", skip_serializing_if = "Option::is_none")]
  pub grafana_url: Option<String>,
  #[serde(rename = "deployment-parameters", skip_serializing_if = "Option::is_none")]
  pub deployment_parameters: Option<HashMap<String, DeploymentParameter>>,
  pub deployment_profiles: Vec<(String, String)>,
}

impl Display for ApplicationDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.application_name, self.application_version)?;
    write!(f, "\n  {}", self.application_description)?;
    if let Some(dps) = &self.deployment_parameters {
      write!(f, "\n  parameters")?;
      for dp in dps {
        write!(f, "\n    {}: {}", dp.0, dp.1)?
      }
    }
    write!(f, "\n  profiles")?;
    for p in &self.deployment_profiles {
      write!(f, "\n    {}: {}", p.0, p.1)?
    }
    Ok(())
  }
}
