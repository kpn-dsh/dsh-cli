use std::collections::HashMap;

use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{JunctionId, ParameterId, ProcessorId, ProcessorIdentifier, ProcessorType, ProfileId, ServiceName};
use trifonius_engine::resource::ResourceType;
use trifonius_engine::resource::{ResourceId, ResourceIdentifier};

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  let processor_identifier = ProcessorIdentifier::new(ProcessorType::DshService, ProcessorId::new("greenbox-consent-filter"));
  let service_name = ServiceName::new("consentfilter-test002");

  let processor_registry = ProcessorRegistry::default();
  let dsh_service = processor_registry.processor_by_identifier(&processor_identifier).unwrap()?;

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

  let config = dsh_service
    .deploy_dry_run(&service_name, &inbound_junctions, &outbound_junctions, &parameters, profile_id)
    .await;
  println!("{}", config.unwrap());
  Ok(())
}
