use lazy_static::lazy_static;

use trifonius_engine::pipeline::PipelineId;
use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_realization::ProcessorRealization;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{JunctionId, ProcessorId, ProcessorIdentifier, ProcessorRealizationId, ProcessorType};
use trifonius_engine::resource::ResourceRealizationId;

lazy_static! {
  pub static ref PROCESSOR_REALIZATION_ID: ProcessorRealizationId = ProcessorRealizationId::new("greenbox-consent-filter");
  pub static ref PROCESSOR_ID: ProcessorId = ProcessorId::new("consentfilter002");
  pub static ref PIPELINE_ID: PipelineId = PipelineId::new("pipeline");
  pub static ref PROCESSOR_IDENTIFIER: ProcessorIdentifier = ProcessorIdentifier::new(ProcessorType::DshService, ProcessorRealizationId::new("greenbox-consent-filter"));
  pub static ref PROCESSOR_REGISTRY: ProcessorRegistry = ProcessorRegistry::default();
}

pub fn dshservice_instance() -> Box<dyn ProcessorInstance> {
  dshservice_realization().processor_instance(Some(pipeline_id()), processor_id()).unwrap()
}

pub fn dshservice_realization() -> &'static dyn ProcessorRealization {
  PROCESSOR_REGISTRY.processor_realization_by_identifier(&PROCESSOR_IDENTIFIER).unwrap()
}

pub fn processor_realization_id() -> ProcessorRealizationId {
  ProcessorRealizationId::new("greenbox-consent-filter")
}

pub fn processor_id() -> ProcessorId {
  ProcessorId::new("consentfilter002")
}

pub fn pipeline_id() -> PipelineId {
  PipelineId::new("pipeline")
}

pub fn processor_identifier() -> ProcessorIdentifier {
  ProcessorIdentifier::new(ProcessorType::DshService, processor_realization_id())
}

pub fn junction_id() -> JunctionId {
  JunctionId::new("inbound-kafka-topic")
}

pub fn resource_realization_id() -> ResourceRealizationId {
  ResourceRealizationId::new("scratch-keyring-codomain-values-p-1ae01b6b71af0247")
}

#[allow(dead_code)]
fn main() {}
