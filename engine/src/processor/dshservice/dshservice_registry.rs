use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;

use dsh_api::dsh_api_tenant::DshApiTenant;

use crate::processor::dshservice::dshservice_realization::DshServiceRealization;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{processor_config_dir_name, ProcessorIdentifier, ProcessorRealizationId};

#[derive(Debug)]
pub(crate) struct DshServiceRealizationRegistry {
  realizations: HashMap<ProcessorRealizationId, DshServiceRealization>,
}

impl DshServiceRealizationRegistry {
  pub(crate) fn create() -> Result<DshServiceRealizationRegistry, String> {
    let mut realizations: HashMap<ProcessorRealizationId, DshServiceRealization> = HashMap::new();
    let paths = fs::read_dir(format!("{}/dshservice", processor_config_dir_name())).map_err(|error| error.to_string())?;
    for path in paths {
      let config_file_name = path.unwrap().path().display().to_string();
      let dshservice_realization = DshServiceRealization::create(config_file_name.as_str())?;
      if realizations.contains_key(dshservice_realization.processor_realization_id()) {
        return Err(format!(
          "dshservice processor realization id '{}' is defined more than once",
          dshservice_realization.processor_realization_id()
        ));
      }
      realizations.insert(dshservice_realization.processor_realization_id().clone(), dshservice_realization);
    }
    Ok(Self { realizations })
  }

  pub(crate) fn processor_realization_by_id(&self, processor_realization_id: &ProcessorRealizationId) -> Option<&dyn ProcessorRealization> {
    match self.realizations.get(processor_realization_id) {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn processor_realization_ids(&self) -> Vec<&ProcessorRealizationId> {
    self.realizations.keys().collect()
  }

  pub(crate) fn processor_identifiers(&self) -> Vec<&ProcessorIdentifier> {
    self.realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn descriptors(&self, dsh_api_tenant: &DshApiTenant) -> Vec<ProcessorDescriptor> {
    self.realizations.values().map(|realization| realization.descriptor(dsh_api_tenant)).collect()
  }
}

impl Display for DshServiceRealizationRegistry {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    for id in self.processor_realization_ids() {
      writeln!(f, "{}", self.processor_realization_by_id(id).unwrap())?
    }
    Ok(())
  }
}
