use std::collections::HashMap;

use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;
use trifonius_engine::resource::resource::ResourceIdentifier;
use trifonius_engine::resource::ResourceType;

const SERVICE_ID: &str = "test-0-0-2";

#[tokio::main]
async fn main() {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service = processor_registry.processor(ProcessorType::DshService, "consentfilter").unwrap();

  let inbound_junctions = HashMap::from([(
    "inbound-kafka-topic".to_string(),
    vec![ResourceIdentifier { resource_type: ResourceType::DshTopic, id: "stream-reference-implementation-3p".to_string() }],
  )]);
  let outbound_junctions = HashMap::from([(
    "outbound-kafka-topic".to_string(),
    vec![ResourceIdentifier { resource_type: ResourceType::DshTopic, id: "scratch-reference-implementation-compliant".to_string() }],
  )]);

  let parameters = HashMap::from([
    ("identifier-picker-regex".to_string(), "(?:cancelled|created|updated):([0-9]+)".to_string()),
    ("identifier-picker-source-system".to_string(), "boss".to_string()),
    ("enable-dsh-envelope".to_string(), "true".to_string()),
    ("compliancy-agent".to_string(), "whitelist".to_string()),
    ("mitigation-strategy".to_string(), "block".to_string()),
  ]);
  let profile_id = Some("minimal");

  let r = dsh_service
    .deploy(SERVICE_ID, &inbound_junctions, &outbound_junctions, &parameters, profile_id)
    .await;

  println!("{:?}", r);
}
