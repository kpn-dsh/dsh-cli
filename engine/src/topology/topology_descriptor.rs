use std::collections::HashMap;

use crate::processor::processor::{Processor, ProcessorIdentifier};
use crate::resource::resource::{Resource, ResourceIdentifier};

struct Topology<'a> {
  processors: HashMap<ProcessorIdentifier, &'a dyn Processor>,
  resources: HashMap<ResourceIdentifier, &'a dyn Resource>,
  connections: Vec<String>,
}

struct ProcessorTopology {
  processor_identifier: ProcessorIdentifier,
}

struct ProcessorInstance {}
