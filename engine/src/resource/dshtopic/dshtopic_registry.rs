use std::collections::HashMap;
use std::sync::Arc;

use dsh_sdk::Properties;
use log::info;

use crate::engine_target::EngineTarget;
use crate::resource::dshtopic::dshtopic_realization::DshTopicRealization;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceIdentifier, ResourceRealizationId, ResourceTechnology};

pub(crate) struct DshTopicRealizationRegistry {
  dshtopic_realizations: HashMap<ResourceIdentifier, DshTopicRealization>,
}

impl DshTopicRealizationRegistry {
  pub(crate) fn create(engine_target: Arc<EngineTarget>) -> Result<Self, String> {
    info!("create dshtopic realization registry");
    let mut dshtopic_realizations: HashMap<ResourceIdentifier, DshTopicRealization> = HashMap::new();
    let dsh_properties: &Properties = Properties::get();
    for stream in dsh_properties.datastream().streams().values() {
      let dshtopic_realization = DshTopicRealization::create(stream, engine_target.clone())?;
      info!("  {}", dshtopic_realization.resource_identifier);
      dshtopic_realizations.insert(dshtopic_realization.resource_identifier.clone(), dshtopic_realization);
    }
    Ok(Self { dshtopic_realizations })
  }

  pub(crate) fn dshtopic_realization_by_id(&self, id: &ResourceRealizationId) -> Option<&(dyn ResourceRealization)> {
    match self
      .dshtopic_realizations
      .get(&ResourceIdentifier { resource_type: ResourceTechnology::DshTopic, id: id.clone() })
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
