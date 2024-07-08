use trifonius_engine::resource::resource_descriptor::ResourceType;
use trifonius_engine::resource::resource_registry::ResourceRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

#[tokio::main]
async fn main() {
  let resource_registry = ResourceRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
  let topic_descriptors = resource_registry.resource_descriptors_by_type(ResourceType::Topic).await.unwrap();
  println!("{}", serde_json::to_string_pretty(&topic_descriptors).unwrap());
  // for topic_descriptor in topic_descriptors {
  //   println!("{}", topic_descriptor);
  // }
}
