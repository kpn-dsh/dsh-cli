use trifonius_engine::processor::application::application_registry::ApplicationRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

const APPLICATION_NAME: &str = "test-0-0-2";

#[tokio::main]
async fn main() {
  env_logger::init();
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = registry.application_by_name("greenbox-consent-filter").unwrap();
  let response = application.status(APPLICATION_NAME).await.unwrap();
  println!("response: {:#?}", response);
}
