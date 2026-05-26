use crate::formatters::Value;
use crate::formatters::{Label, SubjectFormatter};
use dsh_api::types::KafkaAclGroupTopicKind;
use serde::Serialize;

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum AclGroupLabel {
  AclGroupName,
  Kind,
  Readable,
  StreamName,
  Writable,
}

impl Label for AclGroupLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AclGroupName => "acl group",
      Self::Kind => "kind",
      Self::Readable => "readable",
      Self::Writable => "writable",
      Self::StreamName => "stream",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::AclGroupName)
  }
}

impl SubjectFormatter<AclGroupLabel> for Option<(&str, &KafkaAclGroupTopicKind, bool, bool)> {
  fn value(&self, label: &AclGroupLabel, target_id: &str) -> Value {
    match self {
      Some((name, kind, readable, writable)) => match label {
        AclGroupLabel::AclGroupName => Value::target(target_id),
        AclGroupLabel::Kind => Value::plain(kind),
        AclGroupLabel::StreamName => Value::target(name),
        AclGroupLabel::Readable => Value::plain(readable),
        AclGroupLabel::Writable => Value::plain(writable),
      },
      None => match label {
        AclGroupLabel::AclGroupName => Value::target(target_id),
        AclGroupLabel::StreamName => Value::warn("none"),
        _ => Value::empty(),
      },
    }
  }
}
