use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let application = processor_registry.processor(ProcessorType::Application, "consentfilter").ok_or("")?;
  let inbound_compatible_resources = application.compatible_resources("inbound-kafka-topic").await;
  println!("{}", serde_json::to_string_pretty(&inbound_compatible_resources).unwrap());
  let outbound_compatible_resources = application.compatible_resources("outbound-kafka-topic").await;
  println!("{}", serde_json::to_string_pretty(&outbound_compatible_resources).unwrap());
  Ok(())
}
