use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;

#[tokio::main]
async fn main() {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service_descriptors = processor_registry.processor_descriptors_by_type(ProcessorType::DshService);
  println!("{}", serde_json::to_string_pretty(&dsh_service_descriptors).unwrap());
}
