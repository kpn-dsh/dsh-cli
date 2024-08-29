use trifonius_engine::engine_target::EngineTarget;
use trifonius_engine::pipeline::PipelineName;
use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_realization::ProcessorRealization;
use trifonius_engine::processor::processor_registry::{ProcessorRegistry, DEFAULT_PROCESSOR_REGISTRY};
use trifonius_engine::processor::{JunctionId, ProcessorId, ProcessorName, ProcessorType};

#[tokio::main]
async fn main() {
  let pipeline_name: PipelineName = PipelineName::new("pipeline");
  let processor_id: ProcessorId = ProcessorId::new("greenbox-consent-filter");
  let processor_name: ProcessorName = ProcessorName::new("consentfilter002");
  let junction_id: JunctionId = JunctionId::new("inbound-kafka-topic");

  let processor_registry: &'static ProcessorRegistry = &DEFAULT_PROCESSOR_REGISTRY;
  let dsh_service_realization: &dyn ProcessorRealization = processor_registry.processor_realization(ProcessorType::DshService, &processor_id).unwrap();
  let processor_instance: Box<dyn ProcessorInstance> = dsh_service_realization
    .processor_instance(Some(&pipeline_name), &processor_name, EngineTarget::default_engine_target())
    .unwrap();

  let r = processor_instance.compatible_resources(&junction_id).await;
  println!("{:#?}", r);
}
