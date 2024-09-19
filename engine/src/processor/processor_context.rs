use crate::engine_target::EngineTarget;
use crate::processor::processor_registry::ProcessorRegistry;
use crate::resource::resource_registry::ResourceRegistry;

#[derive(Default)]
pub struct ProcessorContext {
  pub engine_target: EngineTarget,
  pub resource_registry: ResourceRegistry,
  pub processor_registry: ProcessorRegistry,
}
