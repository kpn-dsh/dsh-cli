use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service = processor_registry.processor(ProcessorType::DshService, "consentfilter").ok_or("")?;
  let inbound_compatible_resources = dsh_service.compatible_resources("inbound-kafka-topic").await;
  println!("{}", serde_json::to_string_pretty(&inbound_compatible_resources).unwrap());
  let outbound_compatible_resources = dsh_service.compatible_resources("outbound-kafka-topic").await;
  println!("{}", serde_json::to_string_pretty(&outbound_compatible_resources).unwrap());
  Ok(())
}
