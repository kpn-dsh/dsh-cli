use dsh_api::types::KafkaProxy;

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum ProxyLabel {
  Certificate,
  Cpus,
  Instances,
  KafkaProxyZone,
  Mem,
  Name,
  SchemaStore,
  SchemaStoreEnabled,
  SecretNameCaChain,
  Target,
  Validations,
}

impl Label for ProxyLabel {
  fn label_for_show(&self) -> &str {
    match self {
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "number of cpus",
      ProxyLabel::Instances => "number of instances",
      ProxyLabel::KafkaProxyZone => "kafka proxy zone",
      ProxyLabel::Mem => "available memory",
      ProxyLabel::Name => "certificate name",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::SchemaStoreEnabled => "schema store enabled",
      ProxyLabel::SecretNameCaChain => "secret name ca chain",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validation",
    }
  }

  fn is_target_label(&self) -> bool {
    *self == Self::Target
  }
}

impl SubjectFormatter<ProxyLabel> for KafkaProxy {
  fn value(&self, label: &ProxyLabel, target_id: &str) -> String {
    match label {
      ProxyLabel::Certificate => self.certificate.clone(),
      ProxyLabel::Cpus => self.cpus.to_string(),
      ProxyLabel::Instances => self.instances.to_string(),
      ProxyLabel::KafkaProxyZone => self.zone.to_string(),
      ProxyLabel::Mem => self.mem.to_string(),
      ProxyLabel::Name => self.name.clone().unwrap_or_default(),
      ProxyLabel::SchemaStore => {
        if self.schema_store.is_some_and(|enabled| enabled) {
          format!(
            "cpus: {}, mem: {}",
            self.schema_store_cpus.unwrap_or_default(),
            self.schema_store_mem.unwrap_or_default()
          )
        } else {
          "NA".to_string()
        }
      }
      ProxyLabel::SchemaStoreEnabled => self.schema_store.map(|enabled| enabled.to_string()).unwrap_or("NA".to_string()),
      ProxyLabel::SecretNameCaChain => self.secret_name_ca_chain.to_string(),
      ProxyLabel::Target => target_id.to_string(),
      ProxyLabel::Validations => self
        .validations
        .iter()
        .map(|validation| validation.common_name.clone().unwrap_or_default())
        .collect::<Vec<String>>()
        .join("\n"),
    }
  }

  fn target_label(&self) -> Option<ProxyLabel> {
    Some(ProxyLabel::Target)
  }
}

pub static PROXY_LABELS_LIST: [ProxyLabel; 6] =
  [ProxyLabel::Target, ProxyLabel::Certificate, ProxyLabel::Cpus, ProxyLabel::Mem, ProxyLabel::KafkaProxyZone, ProxyLabel::SchemaStoreEnabled];

pub static PROXY_LABELS_SHOW: [ProxyLabel; 11] = [
  ProxyLabel::Target,
  ProxyLabel::Certificate,
  ProxyLabel::Cpus,
  ProxyLabel::Instances,
  ProxyLabel::KafkaProxyZone,
  ProxyLabel::Mem,
  ProxyLabel::Name,
  ProxyLabel::SchemaStoreEnabled,
  ProxyLabel::SchemaStore,
  ProxyLabel::SecretNameCaChain,
  ProxyLabel::Validations,
];
