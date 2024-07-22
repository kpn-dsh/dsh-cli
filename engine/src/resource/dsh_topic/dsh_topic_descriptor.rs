use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DshTopicDescriptor {
  pub topic: String,
  pub partitions: u32,
  pub replication: u32,
  pub dsh_envelope: bool,
  pub read: String,
  pub write: String,
  pub read_pattern: Option<String>,
  pub write_pattern: Option<String>,
  pub partitioner: String,
  #[serde(rename = "partitioning-depth")]
  pub partitioning_depth: u32,
  #[serde(rename = "can-retain")]
  pub can_retain: bool,
  pub cluster: String,
}

impl Display for DshTopicDescriptor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "\n  topic: {}", self.topic)?;
    write!(f, "\n  partitions: {}", self.partitions)?;
    write!(f, "\n  replication: {}", self.replication)?;
    if self.dsh_envelope {
      write!(f, "\n  dsh envelope")?;
    }
    write!(f, "\n  read: {}", self.read)?;
    write!(f, "\n  write: {}", self.write)?;
    if let Some(ref read_pattern) = self.read_pattern {
      write!(f, "\n  read pattern: {}", read_pattern)?;
    }
    if let Some(ref write_pattern) = self.write_pattern {
      write!(f, "\n  write pattern: {}", write_pattern)?;
    }
    write!(f, "\n  partitioner: {}", self.partitioner)?;
    write!(f, "\n  partitioning depth: {}", self.partitioning_depth)?;
    if self.can_retain {
      write!(f, "\n  can retain")?;
    }
    write!(f, "\n  cluster: {}", self.cluster)?;
    Ok(())
  }
}
