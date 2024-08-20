use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let processor_descriptors = ProcessorRegistry::default().processor_descriptors();
  println!("{}", serde_json::to_string_pretty(&processor_descriptors).unwrap());
}
