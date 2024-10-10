use std::collections::HashMap;
use std::fs;

use dsh_api::dsh_api_tenant::DshApiTenant;

use crate::processor::dshapp::dshapp_realization::DshAppRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorIdentifier, ProcessorRealizationId};

#[derive(Debug)]
pub(crate) struct DshAppRealizationRegistry {
  realizations: HashMap<ProcessorRealizationId, DshAppRealization>,
}

impl<'a> DshAppRealizationRegistry {
  pub(crate) fn create() -> Result<DshAppRealizationRegistry, String> {
    let mut realizations: HashMap<ProcessorRealizationId, DshAppRealization> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dshapp", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let config_file_name = path.unwrap().path().display().to_string();
      let dshapp_realization = DshAppRealization::create(config_file_name.as_str())?;
      if realizations.contains_key(dshapp_realization.processor_realization_id()) {
        return Err(format!(
          "dshapp processor realization id '{}' is defined more than once",
          dshapp_realization.processor_realization_id()
        ));
      }
      realizations.insert(dshapp_realization.processor_realization_id().clone(), dshapp_realization);
    }
    Ok(Self { realizations })
  }

  pub(crate) fn processor_realization_by_id(&'a self, processor_realization_id: &ProcessorRealizationId) -> Option<&(dyn ProcessorRealization + 'a)> {
    match self.realizations.get(processor_realization_id) {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn processor_realization_ids(&self) -> Vec<&ProcessorRealizationId> {
    let mut ids: Vec<_> = self.realizations.keys().collect();
    ids.sort();
    ids
  }

  pub(crate) fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn descriptors(&self, dsh_api_tenant: &DshApiTenant) -> Vec<ProcessorDescriptor> {
    self.realizations.values().map(|realization| realization.descriptor(dsh_api_tenant)).collect()
  }
}
