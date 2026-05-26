use crate::formatters::Value;
use crate::formatters::{ColumnAlignment, Label, SubjectFormatter};
use dsh_api::types::KafkaProxy;
use itertools::Itertools;
use serde::Serialize;

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum KafkaProxyLabel {
  AclGroupsEnabled,
  CaChainSecretName,
  Certificate,
  Cpus,
  Instances,
  Mem,
  Name,
  SchemaStore,
  Target,
  Validations,
  Zone,
}

impl Label for KafkaProxyLabel {
  fn as_str(&self) -> &str {
    match self {
      KafkaProxyLabel::AclGroupsEnabled => "acl groups",
      KafkaProxyLabel::CaChainSecretName => "ca certificate secret",
      KafkaProxyLabel::Certificate => "certificate",
      KafkaProxyLabel::Cpus => "cpus",
      KafkaProxyLabel::Instances => "instances",
      KafkaProxyLabel::Mem => "memory",
      KafkaProxyLabel::Name => "name",
      KafkaProxyLabel::SchemaStore => "schema store",
      KafkaProxyLabel::Target => "proxy id",
      KafkaProxyLabel::Validations => "validations",
      KafkaProxyLabel::Zone => "zone",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      KafkaProxyLabel::AclGroupsEnabled => "acl groups",
      KafkaProxyLabel::CaChainSecretName => "ca certificate",
      KafkaProxyLabel::Certificate => "certificate",
      KafkaProxyLabel::Cpus => "cpus",
      KafkaProxyLabel::Instances => "instances",
      KafkaProxyLabel::Mem => "memory",
      KafkaProxyLabel::Name => "proxy name",
      KafkaProxyLabel::SchemaStore => "schema store",
      KafkaProxyLabel::Target => "proxy id",
      KafkaProxyLabel::Validations => "validations",
      KafkaProxyLabel::Zone => "zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }

  fn column_alignment(&self) -> ColumnAlignment {
    match self {
      Self::Mem => ColumnAlignment::Right,
      _ => ColumnAlignment::default(),
    }
  }
}

impl SubjectFormatter<KafkaProxyLabel> for KafkaProxy {
  fn value(&self, label: &KafkaProxyLabel, target_id: &str) -> Value {
    match label {
      KafkaProxyLabel::AclGroupsEnabled => Value::some_or_hide(self.enable_kafka_acl_groups.map(|enabled| if enabled { "enabled" } else { "disabled" })),
      KafkaProxyLabel::CaChainSecretName => Value::target(&self.secret_name_ca_chain),
      KafkaProxyLabel::Certificate => Value::target(&self.certificate),
      KafkaProxyLabel::Cpus => Value::plain(self.cpus),
      KafkaProxyLabel::Instances => Value::plain(self.instances),
      KafkaProxyLabel::Mem => Value::plain(self.mem),
      KafkaProxyLabel::Name => Value::some_or_empty(self.name.clone()),
      KafkaProxyLabel::SchemaStore => Value::some_or(
        self.schema_store.map(|enabled| {
          if enabled {
            format!(
              "enabled (cpus: {}, mem: {})",
              self.schema_store_cpus.map(|cpus| cpus.to_string()).unwrap_or("NA".to_string()),
              self.schema_store_mem.map(|mem| mem.to_string()).unwrap_or("NA".to_string())
            )
          } else {
            "disabled".to_string()
          }
        }),
        "NA",
      ),
      KafkaProxyLabel::Target => Value::target(target_id),
      KafkaProxyLabel::Validations => {
        if self.validations.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            self
              .validations
              .iter()
              .map(|validation| validation.common_name.clone().unwrap_or_default())
              .join("\n"),
          )
        }
      }
      KafkaProxyLabel::Zone => Value::plain(self.zone),
    }
  }
}
