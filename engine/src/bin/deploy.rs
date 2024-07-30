use std::collections::HashMap;

use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{JunctionId, ParameterId, ProcessorId, ProcessorType, ProfileId, ServiceName};
use trifonius_engine::resource::ResourceType;
use trifonius_engine::resource::{ResourceId, ResourceIdentifier};
use trifonius_engine::target_client::DEFAULT_TARGET_CLIENT_FACTORY;

const SERVICE_NAME: &str = "consentfilter-test002";
const PROCESSOR_ID: &str = "greenbox-consent-filter";

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();
  let processor_registry = ProcessorRegistry::default();
  let dsh_service_realization = processor_registry
    .processor_realization(ProcessorType::DshService, &ProcessorId::new(PROCESSOR_ID))
    .unwrap();
  let dsh_service = dsh_service_realization.processor(Some(&DEFAULT_TARGET_CLIENT_FACTORY))?;

  let inbound_junction = JunctionId::new("inbound-kafka-topic");
  let inbound_resource_id = ResourceId::new("stream-reference-implementation-3p");
  let inbound_resource = ResourceIdentifier { resource_type: ResourceType::DshTopic, id: inbound_resource_id };
  let inbound_junctions = HashMap::from([(inbound_junction, vec![inbound_resource])]);

  let outbound_junction = JunctionId::new("outbound-kafka-topic");
  let outbound_resource_id = ResourceId::new("scratch-reference-implementation-compliant");
  let outbound_resource = ResourceIdentifier { resource_type: ResourceType::DshTopic, id: outbound_resource_id };
  let outbound_junctions = HashMap::from([(outbound_junction, vec![outbound_resource])]);

  let parameters = HashMap::from([
    (ParameterId::new("identifier-picker-regex"), "(?:cancelled|created|updated):([0-9]+)".to_string()),
    (ParameterId::new("identifier-picker-source-system"), "boss".to_string()),
    (ParameterId::new("enable-dsh-envelope"), "true".to_string()),
    (ParameterId::new("compliancy-agent"), "whitelist".to_string()),
    (ParameterId::new("mitigation-strategy"), "block".to_string()),
  ]);
  let binding = Some(ProfileId::new("minimal"));
  let profile_id = binding.as_ref();

  let r = dsh_service
    .deploy(&ServiceName::new(SERVICE_NAME), &inbound_junctions, &outbound_junctions, &parameters, profile_id)
    .await;
  println!("{:?}", r);

  Ok(())
}
