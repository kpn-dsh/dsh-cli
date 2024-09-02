use trifonius_engine::resource::resource_registry::ResourceRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let resource_descriptors = ResourceRegistry::default().resource_descriptors();
  println!("{}", serde_json::to_string_pretty(&resource_descriptors).unwrap());
}
