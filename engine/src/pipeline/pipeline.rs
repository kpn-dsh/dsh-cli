#![allow(clippy::module_inception)]

pub trait Pipeline {}

// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
//
// use crate::processor::processor::{Processor, ProcessorIdentifier};
// use crate::resource::resource::{Resource, ResourceIdentifier};
//
// struct DeployablePipeline<'a> {
//   processors: HashMap<ProcessorIdentifier, &'a dyn Processor>,
//   resources: HashMap<ResourceIdentifier, &'a dyn Resource>,
//   connections: Vec<String>,
// }
//
// struct ProcessorPipeline {
//   processor_identifier: ProcessorIdentifier,
// }
//
// struct ProcessorInstance {}
//
// struct DeployableResource {
//   resource_identifier: ResourceIdentifier,
// }
//
// struct DeployableProcessor {
//   processor_identifier: ProcessorIdentifier,
//   inbound_junction_connections: HashMap<String, Vec<ResourceIdentifier>>,
//   outbound_junction_connections: HashMap<String, Vec<ResourceIdentifier>>,
//   parameters: HashMap<String, String>,
//   profile_id: Option<String>,
// }
//
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub enum Edge {
//   ProcessorToProcessor { edge_id: String, source: JunctionIdentifier, target: JunctionIdentifier },
//   ResourceToProcessor { edge_id: String, source: ResourceIdentifier, target: JunctionIdentifier },
//   ProcessorToResource { edge_id: String, source: JunctionIdentifier, target: ResourceIdentifier },
// }
//
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct JunctionIdentifier {
//   pub processor_identifier: ProcessorIdentifier,
//   pub junction_id: String,
// }
