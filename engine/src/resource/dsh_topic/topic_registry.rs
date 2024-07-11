use std::collections::HashMap;

use dsh_sdk::Properties;

use crate::resource::dsh_topic::topic_resource::TopicResourceImpl;
use crate::resource::resource::{Resource, ResourceIdentifier, ResourceStatus};
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::ResourceType;
use crate::TargetClientFactory;

pub struct TopicRegistry<'a> {
  resources: HashMap<ResourceIdentifier, TopicResourceImpl<'a>>,
}

impl<'a> TopicRegistry<'a> {
  pub fn create(target_client_factory: &'a TargetClientFactory) -> Result<TopicRegistry, String> {
    let mut resources: HashMap<ResourceIdentifier, TopicResourceImpl<'a>> = HashMap::new();
    let dsh_properties: &Properties = Properties::get();
    for stream in dsh_properties.datastream().streams().values() {
      let resource = TopicResourceImpl::create(stream, target_client_factory).unwrap();
      resources.insert(resource_identifier(stream.name().to_owned()), resource);
    }
    Ok(TopicRegistry { resources })
  }

  pub fn resource_by_name(&self, name: &str) -> Option<&dyn Resource> {
    match self.resources.get(&resource_identifier(name.to_string())) {
      Some(a) => Some(a),
      None => None,
    }
  }

  pub fn resource_identifiers(&self) -> Vec<&ResourceIdentifier> {
    self.resources.values().map(|resource| resource.identifier()).collect()
  }

  pub fn resource_descriptors(&self) -> Vec<&ResourceDescriptor> {
    self.resources.values().map(|resource| resource.descriptor()).collect()
  }

  pub async fn resource_descriptors_with_status(&self) -> Result<Vec<(&ResourceDescriptor, ResourceStatus)>, String> {
    let mut descriptors: Vec<(&ResourceDescriptor, ResourceStatus)> = Vec::new();
    for resource in self.resources.values() {
      descriptors.push((resource.descriptor(), resource.status().await?))
    }
    Ok(descriptors)
  }
}

pub(crate) fn resource_identifier(name: String) -> ResourceIdentifier {
  ResourceIdentifier { resource_type: ResourceType::DshTopic, name }
}
