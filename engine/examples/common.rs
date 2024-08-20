use lazy_static::lazy_static;

use trifonius_dsh_api::DshApiClientFactory;
use trifonius_engine::pipeline::PipelineName;
use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_registry::{ProcessorRegistry, DEFAULT_PROCESSOR_REGISTRY};
use trifonius_engine::processor::{ProcessorId, ProcessorName, ProcessorType};

lazy_static! {
  static ref PIPELINE_NAME: PipelineName = PipelineName::new("pipeline");
  static ref PROCESSOR_ID: ProcessorId = ProcessorId::new("greenbox-consent-filter");
  static ref PROCESSOR_NAME: ProcessorName = ProcessorName::new("consentfilter002");
}

#[allow(dead_code)]
pub fn default_dsh_service_instance<'a>() -> Box<dyn ProcessorInstance> {
  dsh_service_instance(&PROCESSOR_ID, &PIPELINE_NAME, &PROCESSOR_NAME)
}

#[allow(dead_code)]
pub fn dsh_service_instance<'a>(processor_id: &'static str, pipeline_name: &'static str, processor_name: &'static str) -> Box<dyn ProcessorInstance> {
  let processor_id = ProcessorId::new(processor_id);
  let pipeline_name = PipelineName::new(pipeline_name);
  let processor_name = ProcessorName::new(processor_name);

  let processor_registry: &'static ProcessorRegistry = &DEFAULT_PROCESSOR_REGISTRY;
  let dsh_service_realization = processor_registry.processor_realization(ProcessorType::DshService, &processor_id).unwrap();
  dsh_service_realization
    .processor_instance(Some(&pipeline_name), &processor_name, DshApiClientFactory::default())
    .unwrap()
}

#[allow(dead_code)]
fn main() {}
