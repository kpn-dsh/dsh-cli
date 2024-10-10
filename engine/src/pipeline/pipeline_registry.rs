use std::collections::HashMap;
use std::fs;

use lazy_static::lazy_static;

use crate::engine_target::{EngineTarget, DEFAULT_ENGINE_TARGET};
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::{pipeline_config_dir_name, PipelineId};
use crate::processor::processor_registry::{ProcessorRegistry, DEFAULT_PROCESSOR_REGISTRY};
use crate::resource::resource_registry::{ResourceRegistry, DEFAULT_RESOURCE_REGISTRY};

pub struct PipelineRegistry<'a> {
  pub engine_target: &'a EngineTarget,
  pub resource_registry: &'a ResourceRegistry,
  pub processor_registry: &'a ProcessorRegistry,
  pub pipelines: HashMap<PipelineId, Pipeline<'a>>,
}

impl<'a> PipelineRegistry<'a> {
  pub fn create(engine_target: &'a EngineTarget, resource_registry: &'a ResourceRegistry, processor_registry: &'a ProcessorRegistry) -> Result<PipelineRegistry<'a>, String> {
    let mut pipelines: HashMap<PipelineId, Pipeline> = HashMap::new();
    let paths = fs::read_dir(pipeline_config_dir_name()).map_err(|error| error.to_string())?;
    for path in paths {
      let config_file_name = path.unwrap().path().display().to_string();
      log::info!("reading pipeline config file {}", config_file_name);
      let pipeline = Pipeline::create(config_file_name.as_str(), engine_target.tenant(), resource_registry, processor_registry)?;
      let pipeline_id = pipeline.id.clone();
      if pipelines.contains_key(&pipeline_id) {
        return Err(format!("pipeline id '{}' in config file {} was already defined", pipeline_id, config_file_name));
      }
      pipelines.insert(pipeline.id.clone(), pipeline);
    }
    Ok(Self { engine_target, resource_registry, processor_registry, pipelines })
  }

  pub fn pipeline_realization(&self, pipeline_id: &PipelineId) -> Option<&Pipeline> {
    self.pipelines.get(pipeline_id)
  }

  pub fn pipeline_ids(&self) -> Vec<PipelineId> {
    let mut ids = self.pipelines.keys().cloned().collect::<Vec<PipelineId>>();
    ids.sort();
    ids
  }
}

lazy_static! {
  pub static ref DEFAULT_PIPELINE_REGISTRY: PipelineRegistry<'static> =
    PipelineRegistry::create(&DEFAULT_ENGINE_TARGET, &DEFAULT_RESOURCE_REGISTRY, &DEFAULT_PROCESSOR_REGISTRY).unwrap();
}
