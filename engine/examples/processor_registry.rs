use trifonius_engine::engine_target::EngineTarget;
use trifonius_engine::pipeline::PipelineId;
use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_realization::ProcessorRealization;
use trifonius_engine::processor::processor_registry::{ProcessorRegistry, DEFAULT_PROCESSOR_REGISTRY};
use trifonius_engine::processor::{JunctionId, ProcessorId, ProcessorRealizationId, ProcessorType};

#[tokio::main]
async fn main() {
  let pipeline_id: PipelineId = PipelineId::new("pipeline");
  let processor_realization_id: ProcessorRealizationId = ProcessorRealizationId::new("greenbox-consent-filter");
  let processor_id: ProcessorId = ProcessorId::new("consentfilter002");
  let junction_id: JunctionId = JunctionId::new("inbound-kafka-topic");

  let processor_registry: &'static ProcessorRegistry = &DEFAULT_PROCESSOR_REGISTRY;
  let dshservice_realization: &dyn ProcessorRealization = processor_registry
    .processor_realization(ProcessorType::DshService, &processor_realization_id)
    .unwrap();
  let processor_instance: Box<dyn ProcessorInstance> = dshservice_realization
    .processor_instance(Some(&pipeline_id), &processor_id, EngineTarget::default_engine_target())
    .unwrap();

  let r = processor_instance.compatible_resources(&junction_id).await;
  println!("{:#?}", r);
}
