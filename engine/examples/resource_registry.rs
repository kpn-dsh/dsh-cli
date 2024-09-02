use trifonius_engine::resource::resource_instance::ResourceInstance;
use trifonius_engine::resource::resource_realization::ResourceRealization;
use trifonius_engine::resource::resource_registry::ResourceRegistry;
use trifonius_engine::resource::{ResourceId, ResourceType};

use crate::common::{pipeline_id, resource_realization_id};

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
  let resource_registry = ResourceRegistry::default();
  let resource_realization: &dyn ResourceRealization = resource_registry.resource_realization(ResourceType::DshTopic, &resource_realization_id()).unwrap();
  let resource_instance: Box<dyn ResourceInstance> = resource_realization
    .resource_instance(Some(pipeline_id()), ResourceId::new("consentfilter002"))
    .unwrap();
  println!("{:?} {:?}", resource_instance.pipeline_id(), resource_instance.resource_id());
}
