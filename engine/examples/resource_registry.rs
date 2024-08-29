use trifonius_engine::engine_target::EngineTarget;
use trifonius_engine::pipeline::PipelineName;
use trifonius_engine::processor::processor_instance::ProcessorInstance;
use trifonius_engine::processor::processor_realization::ProcessorRealization;
use trifonius_engine::resource::resource_instance::ResourceInstance;
use trifonius_engine::resource::resource_realization::ResourceRealization;
use trifonius_engine::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};
use trifonius_engine::resource::{ResourceId, ResourceName, ResourceType};

#[tokio::main]
async fn main() {
  let pipeline_name: PipelineName = PipelineName::new("pipeline");
  let resource_id: ResourceId = ResourceId::new("scratch-keyring-codomain-values-p-1ae01b6b71af0247");
  let resource_name: ResourceName = ResourceName::new("consentfilter002");

  let resource_registry: &'static ResourceRegistry = &DEFAULT_RESOURCE_REGISTRY;
  let resource_realization: &dyn ResourceRealization = resource_registry.resource_realization(ResourceType::DshTopic, &resource_id).unwrap();
  let resource_instance: Box<dyn ResourceInstance> = resource_realization
    .resource_instance(Some(&pipeline_name), &resource_name, EngineTarget::default_engine_target())
    .unwrap();

  println!("{:?} {:?}", resource_instance.pipeline_name(), resource_instance.resource_name());
}
