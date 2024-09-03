use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use crate::engine_target::EngineTarget;
use crate::processor::dshservice::dshservice_realization::DshServiceRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};
use crate::resource::resource_registry::ResourceRegistry;

pub(crate) struct DshServiceRealizationRegistry {
  dshservice_realizations: HashMap<ProcessorIdentifier, DshServiceRealization>,
}

impl DshServiceRealizationRegistry {
  pub(crate) fn create(engine_target: Arc<EngineTarget>, resource_registry: Arc<ResourceRegistry>) -> Result<DshServiceRealizationRegistry, String> {
    let mut dshservice_realizations: HashMap<ProcessorIdentifier, DshServiceRealization> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dshservice", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let config_file_name = path.unwrap().path().display().to_string();
      let dshservice_realization = DshServiceRealization::create(config_file_name.as_str(), engine_target.clone(), resource_registry.clone())?;
      dshservice_realizations.insert(dshservice_realization.processor_identifier.clone(), dshservice_realization);
    }
    Ok(Self { dshservice_realizations })
  }

  pub(crate) fn dshservice_realization_by_id(&self, processor_realization_id: &ProcessorRealizationId) -> Option<&dyn ProcessorRealization> {
    match self
      .dshservice_realizations
      .get(&ProcessorIdentifier { processor_technology: ProcessorTechnology::DshService, processor_realization_id: processor_realization_id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dshservice_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.dshservice_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dshservice_descriptors(&self) -> Vec<ProcessorDescriptor> {
    self.dshservice_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
