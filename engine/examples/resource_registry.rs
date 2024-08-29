use trifonius_engine::engine_target::EngineTarget;
use trifonius_engine::pipeline::PipelineId;
use trifonius_engine::resource::resource_instance::ResourceInstance;
use trifonius_engine::resource::resource_realization::ResourceRealization;
use trifonius_engine::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};
use trifonius_engine::resource::{ResourceId, ResourceRealizationId, ResourceType};

#[tokio::main]
async fn main() {
  let pipeline_id: PipelineId = PipelineId::new("pipeline");
  let resource_realization_id: ResourceRealizationId = ResourceRealizationId::new("scratch-keyring-codomain-values-p-1ae01b6b71af0247");
  let resource_id: ResourceId = ResourceId::new("consentfilter002");

  let resource_registry: &'static ResourceRegistry = &DEFAULT_RESOURCE_REGISTRY;
  let resource_realization: &dyn ResourceRealization = resource_registry.resource_realization(ResourceType::DshTopic, &resource_realization_id).unwrap();
  let resource_instance: Box<dyn ResourceInstance> = resource_realization
    .resource_instance(Some(&pipeline_id), &resource_id, EngineTarget::default_engine_target())
    .unwrap();

  println!("{:?} {:?}", resource_instance.pipeline_id(), resource_instance.resource_id());
}
