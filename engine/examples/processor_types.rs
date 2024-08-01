use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let processor_types = ProcessorRegistry::default().processor_types();
  println!("{}", serde_json::to_string_pretty(&processor_types).unwrap());
}
