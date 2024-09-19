use trifonius_engine::engine_target::EngineTarget;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let engine_target = EngineTarget::default();
  let processor_descriptors = ProcessorRegistry::default().processor_descriptors(&engine_target);
  println!("{}", serde_json::to_string_pretty(&processor_descriptors).unwrap());
}
