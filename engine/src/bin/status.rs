use trifonius_engine::application::application_registry::ApplicationRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

#[tokio::main]
async fn main() {
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let application = registry.application("greenbox-consent-filter").unwrap();
  let response = application.status("abcdefgh1").unwrap();
  println!("response: {:#?}", response);
}
