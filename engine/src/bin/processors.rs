use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[tokio::main]
async fn main() {
  let processor_registry = ProcessorRegistry::default();
  let processor_descriptors = processor_registry.processor_descriptors();
  println!("{}", serde_json::to_string_pretty(&processor_descriptors).unwrap());
}
