#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::identifier;

pub mod pipeline;
pub mod pipeline_descriptor;
pub mod pipeline_registry;

identifier!(
  "pipeline",
  PipelineId,
  "pipeline id",
  "^[a-z][a-z0-9]{0,17}$",
  "validpipelineid",
  "invalid.pipeline.id"
);
identifier!(
  "pipeline",
  PipelineProcessorName,
  "pipeline-processor name",
  "^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$",
  "validname-validname",
  "validname_validname"
);
identifier!(
  "pipeline",
  PipelineResourceName,
  "pipeline-resource name",
  "^[a-z][a-z0-9]{0,17}-[a-z][a-z0-9]{0,17}$",
  "validname-validname",
  "validname_validname"
);

// Declared, Configured, Deleted
// NotDeployed, Deployed
// Stopped, Running, Faulty, Killed

// Declared     -             -         DECLARED     Declared
// Configured   NotDeployed   -         CONFIGURED   Configured
// Configured   Deployed      Stopped   DEPLOYED     DeployedStopped
// Configured   Deployed      Running   RUNNING      DeployedRunning
// Configured   Deployed      Faulty    FAULTY       DeployedFaulty
// Configured   Deployed      Killed    KILLED       DeployedKilled
// Deleted      -             -         DELETED      Deleted

// Declared    NotDeployed  Stopped
// Declared    NotDeployed  Running
// Declared    NotDeployed  Faulty
// Declared    NotDeployed  Killed
// Declared    Deployed     Stopped
// Declared    Deployed     Running
// Declared    Deployed     Faulty
// Declared    Deployed     Killed
// Configured  NotDeployed  Stopped
// Configured  NotDeployed  Running
// Configured  NotDeployed  Faulty
// Configured  NotDeployed  Killed
// Configured  Deployed     Stopped
// Configured  Deployed     Running
// Configured  Deployed     Faulty
// Configured  Deployed     Killed
// Deleted     NotDeployed  Stopped
// Deleted     NotDeployed  Running
// Deleted     NotDeployed  Faulty
// Deleted     NotDeployed  Killed
// Deleted     Deployed     Stopped
// Deleted     Deployed     Running
// Deleted     Deployed     Faulty
// Deleted     Deployed     Killed

const CONFIGURED: &str = "configured";
const DECLARED: &str = "declared";
const DELETED: &str = "deleted";
const DEPLOYED: &str = "deployed";
const STARTED: &str = "started";
const STOPPED: &str = "stopped";

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum PipelineState {
  #[serde(rename = "configured")]
  Configured,
  #[serde(rename = "declared")]
  Declared,
  #[serde(rename = "deleted")]
  Deleted,
  #[serde(rename = "deployed")]
  Deployed,
  #[serde(rename = "started")]
  Started,
  #[serde(rename = "stopped")]
  Stopped,
}

impl Display for PipelineState {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      Self::Configured => write!(f, "{}", CONFIGURED),
      Self::Declared => write!(f, "{}", DECLARED),
      Self::Deleted => write!(f, "{}", DELETED),
      Self::Deployed => write!(f, "{}", DEPLOYED),
      Self::Started => write!(f, "{}", STARTED),
      Self::Stopped => write!(f, "{}", STOPPED),
    }
  }
}

impl PipelineState {
  pub fn description(&self) -> &str {
    match self {
      Self::Configured => "Configured",
      Self::Declared => "Declared",
      Self::Deleted => "Deleted",
      Self::Deployed => "Deployed",
      Self::Started => "Started",
      Self::Stopped => "Stopped",
    }
  }

  pub fn label(&self) -> &str {
    match self {
      Self::Configured => "Configured",
      Self::Declared => "Declared",
      Self::Deleted => "Deleted",
      Self::Deployed => "Deployed",
      Self::Started => "Started",
      Self::Stopped => "Stopped",
    }
  }
}

impl TryFrom<&str> for PipelineState {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      CONFIGURED => Ok(Self::Configured),
      DECLARED => Ok(Self::Declared),
      DELETED => Ok(Self::Deleted),
      DEPLOYED => Ok(Self::Deployed),
      STARTED => Ok(Self::Started),
      STOPPED => Ok(Self::Stopped),
      unrecognized => Err(format!("unrecognized pipeline state '{}'", unrecognized)),
    }
  }
}
