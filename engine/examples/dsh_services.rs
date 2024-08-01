use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;

#[path = "common.rs"]
mod common;

fn main() {
  let dsh_service_descriptors = ProcessorRegistry::default().processor_descriptors_by_type(ProcessorType::DshService);
  println!("{}", serde_json::to_string_pretty(&dsh_service_descriptors).unwrap());
}
