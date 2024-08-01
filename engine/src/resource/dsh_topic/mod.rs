use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

pub(crate) mod dsh_topic_descriptor;
mod dsh_topic_instance;
mod dsh_topic_realization;
pub(crate) mod dsh_topic_registry;

const INTERNAL: &str = "internal";
const SCRATCH: &str = "scratch";
const STREAM: &str = "stream";

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum DshTopicType {
  #[serde(rename = "internal")]
  Internal,
  #[serde(rename = "scratch")]
  Scratch,
  #[serde(rename = "stream")]
  Stream,
}

impl Display for DshTopicType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      DshTopicType::Internal => write!(f, "{}", INTERNAL),
      DshTopicType::Scratch => write!(f, "{}", SCRATCH),
      DshTopicType::Stream => write!(f, "{}", STREAM),
    }
  }
}

impl DshTopicType {
  pub fn description(&self) -> &str {
    match self {
      DshTopicType::Internal => "Internal topic that can only be accessed by services running on the DSH platform.",
      DshTopicType::Scratch => "Private topic that can only be accessed by the tenant that created it.",
      DshTopicType::Stream => "Public topic that can additionally be exposed to the outside world through the DSH’s external-facing APIs over the MQTT and HTTP protocols",
    }
  }

  pub fn try_from_topic_name(topic_name: &str) -> Result<Self, String> {
    if topic_name.starts_with("internal.") {
      Ok(DshTopicType::Internal)
    } else if topic_name.starts_with("scratch.") {
      Ok(DshTopicType::Scratch)
    } else if topic_name.starts_with("stream.") {
      Ok(DshTopicType::Stream)
    } else {
      Err(format!("could not determine topic type from topic name '{}'", topic_name))
    }
  }

  pub fn label(&self) -> &str {
    match self {
      DshTopicType::Internal => "Internal",
      DshTopicType::Scratch => "Scratch",
      DshTopicType::Stream => "Stream",
    }
  }
}

impl TryFrom<&str> for DshTopicType {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      INTERNAL => Ok(DshTopicType::Internal),
      SCRATCH => Ok(DshTopicType::Scratch),
      STREAM => Ok(DshTopicType::Stream),
      unrecognized => Err(format!("unrecognized topic type '{}'", unrecognized)),
    }
  }
}
