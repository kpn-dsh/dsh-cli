use trifonius_engine::resource::resource_registry::ResourceRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let resource_types = ResourceRegistry::default().resource_types();
  println!("{}", serde_json::to_string_pretty(&resource_types).unwrap());
}
