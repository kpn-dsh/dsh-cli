use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let resource_descriptors = ProcessorRegistry::default().resource_registry().resource_descriptors();
  println!("{}", serde_json::to_string_pretty(&resource_descriptors).unwrap());
}
