use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::resource::topic::topic_descriptor::TopicDescriptor;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ResourceType {
  #[serde(rename = "topic")]
  Topic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceDescriptor {
  #[serde(rename = "type")]
  pub resource_type: ResourceType,
  pub topic: Option<TopicDescriptor>,
}

impl Display for ResourceDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.resource_type {
      ResourceType::Topic => self.clone().topic.unwrap().fmt(f),
    }
  }
}

impl From<TopicDescriptor> for ResourceDescriptor {
  fn from(value: TopicDescriptor) -> Self {
    ResourceDescriptor { resource_type: ResourceType::Topic, topic: Some(value) }
  }
}
