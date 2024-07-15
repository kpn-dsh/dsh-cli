use trifonius_engine::processor::application::DEFAULT_TARGET_CLIENT_FACTOR;
use trifonius_engine::resource::resource_registry::ResourceRegistry;

fn main() {
  let resource_registry = ResourceRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let resource_descriptors = resource_registry.resource_descriptors();
  println!("{}", serde_json::to_string_pretty(&resource_descriptors).unwrap());
}
