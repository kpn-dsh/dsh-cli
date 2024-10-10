use trifonius_engine::pipeline::pipeline_registry::DEFAULT_PIPELINE_REGISTRY;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
  let pipeline_registry = &DEFAULT_PIPELINE_REGISTRY;

  // println!("{}", pipeline_registry.processor_registry);
  // println!("{}", pipeline_registry.resource_registry);

  for pipeline_id in pipeline_registry.pipeline_ids() {
    let pipeline = pipeline_registry.pipelines.get(&pipeline_id).unwrap();
    println!("{}", pipeline);
  }

  // for processor_id in pipeline_registry.processor_registry.processor_descriptor() {}
}
