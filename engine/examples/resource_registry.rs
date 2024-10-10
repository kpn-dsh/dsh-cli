use trifonius_engine::resource::resource_instance::ResourceInstance;
use trifonius_engine::resource::resource_realization::ResourceRealization;
use trifonius_engine::resource::resource_registry::DEFAULT_RESOURCE_REGISTRY;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
  for resource_descriptor in DEFAULT_RESOURCE_REGISTRY.resource_descriptors() {
    println!("{}", resource_descriptor);
  }

  // let resource_registry = ResourceRegistry::default();
  // let resource_realization: &dyn ResourceRealization = resource_registry.resource_realization(ResourceType::DshTopic, &resource_realization_id()).unwrap();
  // let resource_instance: Box<dyn ResourceInstance> = resource_realization
  //   .resource_instance(Some(pipeline_id()), ResourceId::new("consentfilter002"))
  //   .unwrap();
  // println!("{:?} {:?}", resource_instance.pipeline_id(), resource_instance.resource_id());
}
