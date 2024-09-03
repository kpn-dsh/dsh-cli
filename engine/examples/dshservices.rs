use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorTechnology;

#[path = "common.rs"]
mod common;

fn main() {
  let dshservice_descriptors = ProcessorRegistry::default().processor_descriptors_by_type(ProcessorTechnology::DshService);
  println!("{}", serde_json::to_string_pretty(&dshservice_descriptors).unwrap());
}
