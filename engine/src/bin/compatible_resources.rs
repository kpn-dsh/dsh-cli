use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{JunctionId, ProcessorId, ProcessorType};

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let processor_id = ProcessorId::new("consentfilter");
  let dsh_service = processor_registry.processor(ProcessorType::DshService, &processor_id).unwrap();

  let inbound_kafka_topic = JunctionId::new("inbound-kafka-topic");
  let inbound_compatible_resources = dsh_service.compatible_resources(&inbound_kafka_topic).await;
  println!("{}", serde_json::to_string_pretty(&inbound_compatible_resources).unwrap());

  let outbound_kafka_topic = JunctionId::new("outbound-kafka-topic");
  let outbound_compatible_resources = dsh_service.compatible_resources(&outbound_kafka_topic).await;
  println!("{}", serde_json::to_string_pretty(&outbound_compatible_resources).unwrap());

  Ok(())
}
