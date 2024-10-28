use dsh_api::types::{InternalManagedStream, PublicManagedStream, PublicManagedStreamContractPartitioner};

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum ManagedStreamLabel {
  CanBeRetained,
  KafkaProperties,
  Kind,
  Partitioner,
  Partitions,
  ReplicationFactor,
  Target,
}

impl Label for ManagedStreamLabel {
  fn label_for_list(&self) -> &str {
    match self {
      Self::CanBeRetained => "ret",
      Self::KafkaProperties => "props",
      Self::Kind => "kind",
      Self::Partitioner => "partnr",
      Self::Partitions => "parts",
      Self::ReplicationFactor => "repl",
      Self::Target => "id",
    }
  }

  fn label_for_show(&self) -> &str {
    match self {
      Self::CanBeRetained => "can be retained",
      Self::KafkaProperties => "kafka properties",
      Self::Kind => "kind",
      Self::Partitioner => "partitioner",
      Self::Partitions => "partitions",
      Self::ReplicationFactor => "replication factor",
      Self::Target => "stream id",
    }
  }

  fn is_target_label(&self) -> bool {
    *self == ManagedStreamLabel::Target
  }
}

pub enum ManagedStream<'a> {
  Internal(&'a InternalManagedStream),
  Public(&'a PublicManagedStream),
}

impl SubjectFormatter<ManagedStreamLabel> for ManagedStream<'_> {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
    match self {
      ManagedStream::Internal(internal) => internal.value(label, target_id),
      ManagedStream::Public(public) => public.value(label, target_id),
    }
  }

  fn target_label(&self) -> Option<ManagedStreamLabel> {
    match self {
      ManagedStream::Internal(internal) => internal.target_label(),
      ManagedStream::Public(public) => public.target_label(),
    }
  }
}

impl SubjectFormatter<ManagedStreamLabel> for InternalManagedStream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
    match label {
      ManagedStreamLabel::CanBeRetained => "NA".to_string(),
      ManagedStreamLabel::KafkaProperties => "PROPERTIES".to_string(),
      ManagedStreamLabel::Kind => self.kind.to_string(),
      ManagedStreamLabel::Partitioner => "NA".to_string(),
      ManagedStreamLabel::Partitions => self.partitions.to_string(),
      ManagedStreamLabel::ReplicationFactor => self.replication_factor.to_string(),
      ManagedStreamLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<ManagedStreamLabel> {
    Some(ManagedStreamLabel::Target)
  }
}

impl SubjectFormatter<ManagedStreamLabel> for PublicManagedStream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
    match label {
      ManagedStreamLabel::CanBeRetained => self.contract.can_be_retained.to_string(),
      ManagedStreamLabel::KafkaProperties => "PROPERTIES".to_string(),
      ManagedStreamLabel::Kind => self.kind.to_string(),
      ManagedStreamLabel::Partitioner => match &self.contract.partitioner {
        PublicManagedStreamContractPartitioner::TopicLevelPartitioner(partitioner) => format!("topic level partitioner ({})", partitioner.topic_level),
        PublicManagedStreamContractPartitioner::KafkaDefaultPartitioner(_) => "kafka default partitioner".to_string(),
      },
      ManagedStreamLabel::Partitions => self.partitions.to_string(),
      ManagedStreamLabel::ReplicationFactor => self.replication_factor.to_string(),
      ManagedStreamLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<ManagedStreamLabel> {
    Some(ManagedStreamLabel::Target)
  }
}

pub static INTERNAL_STREAM_LABELS: [ManagedStreamLabel; 4] =
  [ManagedStreamLabel::Target, ManagedStreamLabel::Partitions, ManagedStreamLabel::ReplicationFactor, ManagedStreamLabel::KafkaProperties];

pub static PUBLIC_STREAM_LABELS: [ManagedStreamLabel; 7] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::Kind,
  ManagedStreamLabel::Partitioner,
  ManagedStreamLabel::CanBeRetained,
  ManagedStreamLabel::KafkaProperties,
];

pub static STREAM_LABELS: [ManagedStreamLabel; 7] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::Kind,
  ManagedStreamLabel::Partitioner,
  ManagedStreamLabel::CanBeRetained,
  ManagedStreamLabel::KafkaProperties,
];
