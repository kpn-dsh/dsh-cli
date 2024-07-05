use trifonius_engine::application::application_registry::ApplicationRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

#[tokio::main]
async fn main() {
  let registry = ApplicationRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let descriptors = registry.application_descriptors();
  for descriptor in descriptors {
    println!("{}", serde_json::to_string_pretty(&descriptor).unwrap());
    // println!("{}", &descriptor);
  }
}
