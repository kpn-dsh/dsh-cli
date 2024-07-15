use trifonius_engine::processor::application::application_registry::ApplicationRegistry;
use trifonius_engine::processor::application::DEFAULT_TARGET_CLIENT_FACTOR;

const INSTANCE_ID: &str = "test-0-0-2";

#[tokio::main]
async fn main() {
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = registry.application_by_id("greenbox-consent-filter").unwrap();
  let _ = application.undeploy(INSTANCE_ID).await;
}
