use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

pub mod dsh_topic;
pub mod resource;
pub mod resource_descriptor;
pub mod resource_registry;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ResourceType {
  #[serde(rename = "dsh-topic")]
  DshTopic,
}

impl Display for ResourceType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ResourceType::DshTopic => write!(f, "dsh-topic"),
    }
  }
}
