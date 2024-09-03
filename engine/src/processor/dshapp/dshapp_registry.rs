use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use crate::engine_target::EngineTarget;
use crate::processor::dshapp::dshapp_realization::DshAppRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};
use crate::resource::resource_registry::ResourceRegistry;

pub(crate) struct DshAppRealizationRegistry {
  dshapp_realizations: HashMap<ProcessorIdentifier, DshAppRealization>,
}

impl<'a> DshAppRealizationRegistry {
  pub(crate) fn create(engine_target: Arc<EngineTarget>, resource_registry: Arc<ResourceRegistry>) -> Result<DshAppRealizationRegistry, String> {
    let mut dshapp_realizations: HashMap<ProcessorIdentifier, DshAppRealization> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dshapp", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let config_file_name = path.unwrap().path().display().to_string();
      let dshapp_realization = DshAppRealization::create(config_file_name.as_str(), engine_target.clone(), resource_registry.clone())?;
      dshapp_realizations.insert(
        ProcessorIdentifier { processor_technology: ProcessorTechnology::DshApp, processor_realization_id: dshapp_realization.processor_realization_id().clone() },
        dshapp_realization,
      );
    }
    Ok(Self { dshapp_realizations })
  }

  pub(crate) fn dshapp_realization_by_id(&'a self, processor_realization_id: &ProcessorRealizationId) -> Option<&(dyn ProcessorRealization + 'a)> {
    match self
      .dshapp_realizations
      .get(&ProcessorIdentifier { processor_technology: ProcessorTechnology::DshApp, processor_realization_id: processor_realization_id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dshapp_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dshapp_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dshapp_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dshapp_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
