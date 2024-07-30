use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{JunctionId, ProcessorId, ProcessorType};
use trifonius_engine::target_client::DEFAULT_TARGET_CLIENT_FACTORY;

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let processor_id = ProcessorId::new("greenbox-consent-filter");
  let dsh_service_realization = processor_registry.processor_realization(ProcessorType::DshService, &processor_id).unwrap();
  let dsh_service = dsh_service_realization.processor(Some(&DEFAULT_TARGET_CLIENT_FACTORY))?;

  let inbound_kafka_topic = JunctionId::new("inbound-kafka-topic");
  let inbound_compatible_resources = dsh_service.compatible_resources(&inbound_kafka_topic).await;
  println!("{}", serde_json::to_string_pretty(&inbound_compatible_resources).unwrap());

  let outbound_kafka_topic = JunctionId::new("outbound-kafka-topic");
  let outbound_compatible_resources = dsh_service.compatible_resources(&outbound_kafka_topic).await;
  println!("{}", serde_json::to_string_pretty(&outbound_compatible_resources).unwrap());

  Ok(())
}
