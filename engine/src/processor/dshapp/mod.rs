use crate::identifier;
use crate::pipeline::PipelineId;
use crate::processor::ProcessorId;

pub mod dshapp_api;
pub mod dshapp_config;
pub mod dshapp_instance;
pub mod dshapp_realization;
pub mod dshapp_registry;

identifier!(
  "trifonius_engine::processor::dshapp",
  DshAppName,
  "dsh app name",
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

impl TryFrom<(Option<&PipelineId>, &ProcessorId)> for DshAppName {
  type Error = String;

  fn try_from((pipeline_id, processor_id): (Option<&PipelineId>, &ProcessorId)) -> Result<Self, Self::Error> {
    match pipeline_id {
      Some(pipeline_id) => DshAppName::try_from(format!("{}-{}", pipeline_id, processor_id)),
      None => DshAppName::try_from(processor_id.to_string()),
    }
  }
}
