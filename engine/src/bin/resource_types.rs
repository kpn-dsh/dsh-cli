use trifonius_engine::resource::resource_registry::ResourceRegistry;

fn main() {
  let resource_registry = ResourceRegistry::default();
  let resource_types = resource_registry.resource_types();
  println!("{}", serde_json::to_string_pretty(&resource_types).unwrap());
}
