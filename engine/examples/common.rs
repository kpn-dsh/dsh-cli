use lazy_static::lazy_static;

use trifonius_engine::engine_target::EngineTarget;
use trifonius_engine::pipeline::PipelineId;
use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_registry::{ProcessorRegistry, DEFAULT_PROCESSOR_REGISTRY};
use trifonius_engine::processor::{ProcessorId, ProcessorRealizationId, ProcessorType};

lazy_static! {
  static ref PIPELINE_ID: PipelineId = PipelineId::new("pipeline");
  static ref PROCESSOR_REALIZATION_ID: ProcessorRealizationId = ProcessorRealizationId::new("greenbox-consent-filter");
  static ref PROCESSOR_ID: ProcessorId = ProcessorId::new("consentfilter002");
}

#[allow(dead_code)]
pub fn default_dshservice_instance<'a>() -> Box<dyn ProcessorInstance> {
  dshservice_instance(&PROCESSOR_REALIZATION_ID, &PIPELINE_ID, &PROCESSOR_ID)
}

#[allow(dead_code)]
pub fn dshservice_instance<'a>(processor_id: &'static str, pipeline_name: &'static str, processor_name: &'static str) -> Box<dyn ProcessorInstance> {
  let processor_id = ProcessorRealizationId::new(processor_id);
  let pipeline_name = PipelineId::new(pipeline_name);
  let processor_name = ProcessorId::new(processor_name);

  let processor_registry: &'static ProcessorRegistry = &DEFAULT_PROCESSOR_REGISTRY;
  let dshservice_realization = processor_registry.processor_realization(ProcessorType::DshService, &processor_id).unwrap();
  dshservice_realization
    .processor_instance(Some(&pipeline_name), &processor_name, EngineTarget::default_engine_target())
    .unwrap()
}

#[allow(dead_code)]
fn main() {}
