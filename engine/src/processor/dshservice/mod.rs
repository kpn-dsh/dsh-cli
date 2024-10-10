use crate::identifier;
use crate::pipeline::PipelineId;
use crate::processor::ProcessorId;

pub mod dshservice_api;
pub mod dshservice_config;
pub mod dshservice_instance;
pub mod dshservice_realization;
pub mod dshservice_registry;

identifier!(
  "trifonius_engine::processor::dshservice",
  DshServiceName,
  "dsh service name",
  "^[a-z][a-z0-9]{0,17}(-[a-z][a-z0-9]{0,17})?$",
  "validname-validname",
  "validname_validname",
  /// Documentation
);
identifier!(
  "trifonius_engine::processor::dshservice",
  TaskId,
  "task identifier",
  "^[a-z0-9-._]{1,32}$",
  "84db5b4b79-6bgtl-00000000",
  "invalid task id",
  /// Documentation
);

impl TryFrom<(Option<&PipelineId>, &ProcessorId)> for DshServiceName {
  type Error = String;

  fn try_from((pipeline_id, processor_id): (Option<&PipelineId>, &ProcessorId)) -> Result<Self, Self::Error> {
    match pipeline_id {
      Some(pipeline_id) => DshServiceName::try_from(format!("{}-{}", pipeline_id, processor_id)),
      None => DshServiceName::try_from(processor_id.to_string()),
    }
  }
}
