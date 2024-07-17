use std::collections::HashMap;

use trifonius_engine::processor::application::DEFAULT_TARGET_CLIENT_FACTOR;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;
use trifonius_engine::resource::resource_registry::ResourceRegistry;
use trifonius_engine::resource::ResourceType;

const SERVICE_ID: &str = "test-0-0-2";

#[tokio::main]
async fn main() {
  let processor_registry = ProcessorRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = processor_registry.processor(ProcessorType::Application, "consentfilter").unwrap();

  let resource_registry = ResourceRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let inbound_resource = resource_registry.resource(ResourceType::DshTopic, "stream.reference-implementation-3p").unwrap();
  let outbound_resource = resource_registry
    .resource(ResourceType::DshTopic, "scratch.reference-implementation-compliant")
    .unwrap();

  let inbound_junctions = HashMap::from([("inbound-kafka-topic".to_string(), inbound_resource)]);
  let outbound_junctions = HashMap::from([("outbound-kafka-topic".to_string(), outbound_resource)]);

  let parameters = HashMap::from([
    ("identifier-picker-regex".to_string(), "(?:cancelled|created|updated):([0-9]+)".to_string()),
    ("identifier-picker-source-system".to_string(), "boss".to_string()),
    ("enable-dsh-envelope".to_string(), "true".to_string()),
    ("compliancy-agent".to_string(), "whitelist".to_string()),
    ("mitigation-strategy".to_string(), "block".to_string()),
  ]);
  let profile_id = Some("minimal");

  for resource in inbound_junctions.values() {
    println!("{}", resource.descriptor());
  }

  for resource in outbound_junctions.values() {
    println!("{}", resource.descriptor());
  }

  println!("{:#?}", parameters);
  println!("{:?}", profile_id);

  let r = application
    .deploy(SERVICE_ID, &inbound_junctions, &outbound_junctions, &parameters, profile_id)
    .await;

  println!("{:?}", r);
}
