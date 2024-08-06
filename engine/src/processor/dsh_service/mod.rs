use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::identifier;
use crate::pipeline::PipelineName;
use crate::processor::ProcessorName;

pub mod dsh_service_api;
pub mod dsh_service_config;
pub mod dsh_service_instance;
pub mod dsh_service_realization;
pub mod dsh_service_registry;

identifier!(
  "processor::dsh_service",
  DshServiceName,
  "dsh service name",
  "^[a-z][a-z0-9]{0,17}(-[a-z][a-z0-9]{0,17})?$",
  "validname-validname",
  "validname_validname"
);
identifier!(
  "processor::dsh_service",
  TaskId,
  "task identifier",
  "^[a-z0-9-._]{1,32}$",
  "84db5b4b79-6bgtl-00000000",
  "invalid task id"
);

impl TryFrom<(Option<&PipelineName>, &ProcessorName)> for DshServiceName {
  type Error = String;

  fn try_from((pipeline_name, processor_name): (Option<&PipelineName>, &ProcessorName)) -> Result<Self, Self::Error> {
    match pipeline_name {
      Some(pn) => DshServiceName::try_from(format!("{}-{}", pn, processor_name)),
      None => DshServiceName::try_from(processor_name.to_string()),
    }
  }
}
