use std::collections::HashMap;

use trifonius_engine::processor::application::application_registry::ApplicationRegistry;
use trifonius_engine::processor::processor::ProcessorDeployParameters;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

const APPLICATION_NAME: &str = "test-0-0-2";

#[tokio::main]
async fn main() {
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = registry.application_by_name("greenbox-consent-filter").unwrap();
  let deploy_parameters = ProcessorDeployParameters {
    inbound_junctions: &HashMap::from([("inbound-kafka-topic".to_string(), "stream.reference-implementation-3p.greenbox-dev".to_string())]),
    outbound_junctions: &HashMap::from([(
      "outbound-kafka-topic".to_string(),
      "scratch.reference-implementation-compliant.greenbox-dev".to_string(),
    )]),
    parameters: &HashMap::from([
      ("identifier-picker-regex".to_string(), "(?:cancelled|created|updated):([0-9]+)".to_string()),
      ("identifier-picker-source-system".to_string(), "boss".to_string()),
      ("enable-dsh-envelope".to_string(), "true".to_string()),
      ("compliancy-agent".to_string(), "classification".to_string()),
      ("mitigation-strategy".to_string(), "block".to_string()),
    ]),
    profile_name: Some("minimal"),
  };
  let _ = application.deploy(APPLICATION_NAME, &deploy_parameters).await;
}
