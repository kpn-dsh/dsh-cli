use std::collections::HashMap;

use dsh_sdk::Properties;

use crate::engine_target::EngineTarget;
use crate::resource::dshtopic::dshtopic_realization::DshTopicRealization;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceIdentifier, ResourceRealizationId, ResourceType};

pub(crate) struct DshTopicRealizationRegistry {
  dshtopic_realizations: HashMap<ResourceIdentifier, DshTopicRealization>,
}

impl<'a> DshTopicRealizationRegistry {
  pub(crate) fn create(engine_target: &'a EngineTarget) -> Result<Self, String> {
    let mut dshtopic_realizations: HashMap<ResourceIdentifier, DshTopicRealization> = HashMap::new();
    let dsh_properties: &Properties = Properties::get();
    for stream in dsh_properties.datastream().streams().values() {
      let dshtopic_realization = DshTopicRealization::create(stream, engine_target)?;
      dshtopic_realizations.insert(dshtopic_realization.resource_identifier.clone(), dshtopic_realization);
    }
    Ok(Self { dshtopic_realizations })
  }

  pub(crate) fn dshtopic_realization_by_id(&self, id: &ResourceRealizationId) -> Option<&(dyn ResourceRealization)> {
    match self
      .dshtopic_realizations
      .get(&ResourceIdentifier { resource_type: ResourceType::DshTopic, id: id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dshtopic_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.dshtopic_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dshtopic_descriptors(&self) -> Vec<ResourceDescriptor> {
    self.dshtopic_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
