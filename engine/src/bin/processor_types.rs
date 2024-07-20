use trifonius_engine::processor::processor_registry::ProcessorRegistry;

fn main() {
  let processor_registry = ProcessorRegistry::default();
  let processor_types = processor_registry.processor_types();
  println!("{}", serde_json::to_string_pretty(&processor_types).unwrap());
}
