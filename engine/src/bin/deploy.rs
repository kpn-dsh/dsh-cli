use std::collections::HashMap;

use trifonius_engine::application::application_registry::ApplicationRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

const APPLICATION_NAME: &str = "abcdefgh1";

#[tokio::main]
async fn main() {
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = registry.application("greenbox-consent-filter").unwrap();
  let mut deployment_config: HashMap<String, String> = HashMap::new();
  deployment_config.insert("sink-topic".to_string(), "scratch.reference-implementation-compliant.greenbox-dev".to_string());
  deployment_config.insert("identifier-picker-regex".to_string(), "(?:cancelled|created|updated):([0-9]+)".to_string());
  deployment_config.insert("identifier-picker-source-system".to_string(), "boss".to_string());
  deployment_config.insert("enable-dsh-envelope".to_string(), "true".to_string());
  deployment_config.insert("source-topic".to_string(), "stream.reference-implementation.greenbox-dev".to_string());
  deployment_config.insert("compliancy-agent".to_string(), "classification".to_string());
  deployment_config.insert("mitigation-strategy".to_string(), "clip".to_string());
  let _ = application.deploy(APPLICATION_NAME, &deployment_config, Some("minimal"));
}
