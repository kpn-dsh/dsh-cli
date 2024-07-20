use trifonius_engine::processor::processor_registry::ProcessorRegistry;

fn main() {
  let processor_registry = ProcessorRegistry::default();
  let resource_registry = processor_registry.resource_registry();
  let resource_descriptors = resource_registry.resource_descriptors();
  println!("{}", serde_json::to_string_pretty(&resource_descriptors).unwrap());
}
