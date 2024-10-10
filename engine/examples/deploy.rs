use std::collections::HashMap;

use trifonius_engine::processor::{JunctionId, ParameterId};
use trifonius_engine::resource::ResourceTechnology;
use trifonius_engine::resource::{ResourceIdentifier, ResourceRealizationId};
use trifonius_engine::ProfileId;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  let dshservice_instance = crate::common::dshservice_instance();

  let inbound_junction = JunctionId::new("inbound-dsh-topic");
  let inbound_resource_id = ResourceRealizationId::new("stream-reference-implementation-3p");
  let inbound_resource = ResourceIdentifier { resource_type: ResourceTechnology::DshTopic, id: inbound_resource_id };
  let inbound_junctions = HashMap::from([(inbound_junction, vec![inbound_resource])]);

  let outbound_junction = JunctionId::new("outbound-dsh-topic");
  let outbound_resource_id = ResourceRealizationId::new("scratch-reference-implementation-compliant");
  let outbound_resource = ResourceIdentifier { resource_type: ResourceTechnology::DshTopic, id: outbound_resource_id };
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

  // let r = dshservice_instance.deploy(&inbound_junctions, &outbound_junctions, &parameters, profile_id).await;
  // println!("{:?}", r);

  Ok(())
}
