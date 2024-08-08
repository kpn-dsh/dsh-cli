use std::collections::HashMap;

use dsh_sdk::Properties;

use crate::engine_target::EngineTarget;
use crate::resource::dsh_topic::dsh_topic_realization::DshTopicRealization;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceType};

pub(crate) struct DshTopicRealizationRegistry {
  dsh_topic_realizations: HashMap<ResourceIdentifier, DshTopicRealization>,
}

impl<'a> DshTopicRealizationRegistry {
  pub(crate) fn create(engine_target: &'a EngineTarget) -> Result<Self, String> {
    let mut dsh_topic_realizations: HashMap<ResourceIdentifier, DshTopicRealization> = HashMap::new();
    let dsh_properties: &Properties = Properties::get();
    for stream in dsh_properties.datastream().streams().values() {
      let dsh_topic_realization = DshTopicRealization::create(stream, engine_target)?;
      dsh_topic_realizations.insert(dsh_topic_realization.resource_identifier.clone(), dsh_topic_realization);
    }
    Ok(Self { dsh_topic_realizations })
  }

  pub(crate) fn dsh_topic_realization_by_id(&self, id: &ResourceId) -> Option<&(dyn ResourceRealization)> {
    match self
      .dsh_topic_realizations
      .get(&ResourceIdentifier { resource_type: ResourceType::DshTopic, id: id.clone() })
    {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn dsh_topic_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.dsh_topic_realizations.values().map(|realization| realization.identifier()).collect()
  }

  pub(crate) fn dsh_topic_descriptors(&self) -> Vec<ResourceDescriptor> {
    self.dsh_topic_realizations.values().map(|realization| realization.descriptor()).collect()
  }
}
