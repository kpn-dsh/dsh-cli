#![allow(clippy::module_inception)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::pipeline::PipelineName;
use crate::processor::{JunctionId, ParameterId, ProcessorIdentifier, ProcessorName, ProfileId};
use crate::resource::ResourceIdentifier;

pub struct Pipeline {
  name: PipelineName,
  resources: Vec<PipelineResource>,
  processors: Vec<PipelineProcessor>,
  junctions: Vec<PipelineJunction>,
  dependencies: Vec<PipelineDependency>,
}

pub struct PipelineResource {
  resource: ResourceIdentifier,
  parameters: HashMap<ParameterId, String>,
}

pub struct PipelineProcessor {
  processor: ProcessorIdentifier,
  name: ProcessorName,
  parameters: HashMap<ParameterId, String>,
  profile_id: Option<ProfileId>,
}

pub struct PipelineJunction {
  junction: JunctionType,
  parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum JunctionType {
  ResourceToProcessor { source: Vec<ResourceIdentifier>, target: JunctionIdentifier },
  ProcessorToResource { source: JunctionIdentifier, target: Vec<ResourceIdentifier> },
  ProcessorToProcessor { source: JunctionIdentifier, target: JunctionIdentifier },
}

struct PipelineDependency {
  parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DependencyType {
  ProcessorOnProcessor { depended: ProcessorIdentifier, depends_on: ProcessorIdentifier },
  ProcessorOnResource { depended: JunctionIdentifier, depends_on: Vec<ResourceIdentifier> },
  ResourceOnProcessor { depended: ProcessorIdentifier, depends_on: ResourceIdentifier },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JunctionIdentifier {
  pub processor_identifier: ProcessorIdentifier,
  pub junction_id: JunctionId,
}
