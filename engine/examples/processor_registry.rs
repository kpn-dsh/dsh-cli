use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_realization::ProcessorRealization;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;

use crate::common::{junction_id, pipeline_id, processor_id, processor_realization_id};

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
  let processor_registry = ProcessorRegistry::default();
  let dshservice_realization: &dyn ProcessorRealization = processor_registry
    .processor_realization(ProcessorType::DshService, &processor_realization_id())
    .unwrap();
  let processor_instance: Box<dyn ProcessorInstance> = dshservice_realization.processor_instance(Some(pipeline_id()), processor_id()).unwrap();

  let r = processor_instance.compatible_resources(&junction_id()).await;
  println!("{:#?}", r);
}
