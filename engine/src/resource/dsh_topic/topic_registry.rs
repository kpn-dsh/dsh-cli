use std::collections::HashMap;

use dsh_sdk::Properties;

use crate::resource::dsh_topic::topic_resource::{topic_resource_identifier, TopicResourceImpl};
use crate::resource::resource::{Resource, ResourceIdentifier, ResourceStatus};
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::target_client::TargetClientFactory;

pub(crate) struct TopicRegistry<'a> {
  resources: HashMap<ResourceIdentifier, TopicResourceImpl<'a>>,
}

impl<'a> TopicRegistry<'a> {
  pub(crate) fn create(target_client_factory: &'a TargetClientFactory) -> Result<TopicRegistry, String> {
    let mut resources: HashMap<ResourceIdentifier, TopicResourceImpl<'a>> = HashMap::new();
    let dsh_properties: &Properties = Properties::get();
    for stream in dsh_properties.datastream().streams().values() {
      let resource = TopicResourceImpl::create(stream, target_client_factory).unwrap();
      resources.insert(topic_resource_identifier(stream.name().to_owned()), resource);
    }
    Ok(TopicRegistry { resources })
  }

  pub(crate) fn resource_by_id(&self, id: &str) -> Option<&(dyn Resource + Sync)> {
    match self.resources.get(&topic_resource_identifier(id.to_string())) {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub(crate) fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.resources.values().map(|resource| resource.identifier()).collect()
  }

  pub(crate) fn resource_descriptors(&self) -> Vec<&ResourceDescriptor> {
    self.resources.values().map(|resource| resource.descriptor()).collect()
  }

  pub(crate) async fn resource_descriptors_with_status(&self) -> Result<Vec<(&ResourceDescriptor, ResourceStatus)>, String> {
    let mut descriptors: Vec<(&ResourceDescriptor, ResourceStatus)> = Vec::new();
    for resource in self.resources.values() {
      descriptors.push((resource.descriptor(), resource.status().await?))
    }
    Ok(descriptors)
  }
}
