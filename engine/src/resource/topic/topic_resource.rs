use crate::resource::resource::{Resource, ResourceIdentifier};
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceType};
use crate::resource::topic::topic_descriptor::TopicDescriptor;
use crate::resource::topic::topic_registry::get_topic_status;
use crate::TargetClientFactory;
use async_trait::async_trait;

pub struct TopicResourceImpl<'a> {
  pub resource_identifier: ResourceIdentifier,
  pub resource_descriptor: ResourceDescriptor,
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> TopicResourceImpl<'a> {
  pub fn create(topic_descriptor: TopicDescriptor, target_client_factory: &'a TargetClientFactory) -> Result<Box<dyn Resource + 'a>, String> {
    Ok(Box::new(TopicResourceImpl {
      resource_identifier: ResourceIdentifier { resource_type: ResourceType::Topic, name: topic_descriptor.topic_name.clone() },
      resource_descriptor: ResourceDescriptor::from(topic_descriptor),
      target_client_factory,
    }))
  }
}

#[async_trait]
impl Resource for TopicResourceImpl<'_> {
  fn descriptor(&self) -> &ResourceDescriptor {
    &self.resource_descriptor
  }

  fn identifier(&self) -> &ResourceIdentifier {
    &self.resource_identifier
  }

  fn resource_type(&self) -> ResourceType {
    ResourceType::Topic
  }

  async fn status(&self, resource_name: &str) -> Result<String, String> {
    match get_topic_status(self.target_client_factory, resource_name).await? {
      Some(status) => {
        if status.provisioned {
          Ok("provisioned".to_string())
        } else {
          Ok("not-provisioned".to_string())
        }
      }
      None => Err(format!("could not get status for non-existent topic {}", resource_name)),
    }
  }
}
