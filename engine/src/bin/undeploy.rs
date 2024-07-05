use trifonius_engine::application::application_registry::ApplicationRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

const APPLICATION_NAME: &str = "abcdefgh1";

#[tokio::main]
async fn main() {
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = registry.application("greenbox-consent-filter").unwrap();
  let _ = application.undeploy(APPLICATION_NAME);
}
