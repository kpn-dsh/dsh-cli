use trifonius_engine::resource::resource_registry::ResourceRegistry;
use trifonius_engine::resource::ResourceTechnology;

#[path = "common.rs"]
mod common;

fn main() {
  let resource_registry = ResourceRegistry::default();
  let topic_descriptors = resource_registry.resource_descriptors_by_type(&ResourceTechnology::DshTopic);
  println!("{}", serde_json::to_string_pretty(&topic_descriptors).unwrap());
}
