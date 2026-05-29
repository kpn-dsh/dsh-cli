use crate::formatters::Value;
use crate::formatters::{ColumnAlignment, Label, SubjectFormatter};
use crate::proxy_bundles::ProxyCertificateBundleConfig;
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
#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum BundleLabel {
  AclGroupName,
  BundleDirectory,
  BundleName,
  CaCommonName,
  DnsEntries,
  GroupId,
  NumberOfDsnRecords,
  Platform,
  PlatformDomain,
  ProxyCommonName,
  ProxyName,
  SchemaStore,
  Tenant,
  VhostZone,
}

impl Label for BundleLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AclGroupName => "acl group name",
      Self::BundleDirectory => "directory",
      Self::BundleName => "bundle",
      Self::CaCommonName => "ca common name",
      Self::DnsEntries => "dns entries",
      Self::GroupId => "group id",
      Self::NumberOfDsnRecords => "records",
      Self::Platform => "platform",
      Self::PlatformDomain => "platform domain",
      Self::ProxyCommonName => "proxy common name",
      Self::ProxyName => "proxy name",
      Self::SchemaStore => "schema store",
      Self::Tenant => "tenant",
      Self::VhostZone => "vhost zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, BundleLabel::BundleName)
  }
}

impl SubjectFormatter<BundleLabel> for (ProxyCertificateBundleConfig, String) {
  fn value(&self, label: &BundleLabel, target_id: &str) -> Value {
    let (config, directory) = self;
    match label {
      BundleLabel::BundleDirectory => Value::plain(directory),
      _ => config.value(label, target_id),
    }
  }
}

impl SubjectFormatter<BundleLabel> for ProxyCertificateBundleConfig {
  fn value(&self, label: &BundleLabel, target_id: &str) -> Value {
    match label {
      BundleLabel::AclGroupName => Value::some_or_hide(self.acl_group_name.clone()),
      BundleLabel::BundleDirectory => Value::unreachable(),
      BundleLabel::BundleName => Value::target(target_id),
      BundleLabel::CaCommonName => Value::plain(&self.ca_common_name),
      BundleLabel::DnsEntries => Value::result(self.dns_entries().map(|dns_entry| dns_entry.join("\n"))),
      BundleLabel::GroupId => Value::plain(self.group_id(1)),
      BundleLabel::NumberOfDsnRecords => Value::plain(self.number_of_dns_records),
      BundleLabel::Platform => Value::target(&self.platform),
      BundleLabel::PlatformDomain => Value::result(self.domain_from_platform()),
      BundleLabel::ProxyCommonName => Value::result(self.common_name()),
      BundleLabel::ProxyName => Value::target(&self.proxy_name),
      BundleLabel::SchemaStore => {
        if self.enable_schema_store {
          Value::plain("enabled")
        } else {
          Value::plain("disabled")
        }
      }
      BundleLabel::Tenant => Value::target(&self.tenant),
      BundleLabel::VhostZone => Value::plain(&self.vhost_zone),
    }
  }
}
