use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::directory::{CA_CERTIFICATE_FILENAME, CLIENT_CERTIFICATE_FILENAME, CLIENT_KEY_FILENAME};
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

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ProxyBundleLabel {
  AclGroupName,
  Brokers,
  BundleDirectory,
  BundleName,
  CaCommonName,
  ClientId,
  DnsEntries,
  GroupId,
  NumberOfDsnRecords,
  PkiCaCertificateFilename,
  PkiClientCertificateFilename,
  PkiClientKeyFilename,
  Platform,
  PlatformDomain,
  ProxyCommonName,
  ProxyName,
  SchemaStore,
  SchemaStoreEndpoint,
  Tenant,
  VhostZone,
}

impl Label for ProxyBundleLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AclGroupName => "acl group name",
      Self::Brokers => "brokers",
      Self::BundleDirectory => "bundle directory",
      Self::BundleName => "bundle",
      Self::CaCommonName => "ca common name",
      Self::ClientId => "client id",
      Self::DnsEntries => "dns entries",
      Self::GroupId => "group id",
      Self::NumberOfDsnRecords => "records",
      Self::PkiCaCertificateFilename => "ca certificate file",
      Self::PkiClientCertificateFilename => "client certificate file",
      Self::PkiClientKeyFilename => "client key file",
      Self::Platform => "platform",
      Self::PlatformDomain => "platform domain",
      Self::ProxyCommonName => "proxy common name",
      Self::ProxyName => "proxy name",
      Self::SchemaStore => "schema store",
      Self::SchemaStoreEndpoint => "schema store endpoint",
      Self::Tenant => "tenant",
      Self::VhostZone => "vhost zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, ProxyBundleLabel::BundleName)
  }
}

impl SubjectFormatter<ProxyBundleLabel> for (ProxyCertificateBundleConfig, String) {
  fn value(&self, label: &ProxyBundleLabel, target_id: &str) -> Value {
    let (config, directory) = self;
    match label {
      ProxyBundleLabel::BundleDirectory => Value::plain(directory),
      _ => config.value(label, target_id),
    }
  }
}

impl SubjectFormatter<ProxyBundleLabel> for ProxyCertificateBundleConfig {
  fn value(&self, label: &ProxyBundleLabel, target_id: &str) -> Value {
    match label {
      ProxyBundleLabel::AclGroupName => Value::some_or_hide(self.acl_group_name.clone()),
      ProxyBundleLabel::Brokers => Value::result(
        self
          .platform
          .tenant_proxy_bootstrap_servers(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), 3)
          .map(|servers| servers.join("\n")),
      ),
      ProxyBundleLabel::BundleName => Value::target(target_id),
      ProxyBundleLabel::CaCommonName => Value::plain(&self.ca_common_name),
      ProxyBundleLabel::ClientId => Value::plain(self.client_id()),
      ProxyBundleLabel::DnsEntries => Value::result(self.dns_entries().map(|dns_entry| dns_entry.join("\n"))),
      ProxyBundleLabel::GroupId => Value::plain(self.group_id(1)),
      ProxyBundleLabel::NumberOfDsnRecords => Value::plain(self.number_of_dns_records),
      ProxyBundleLabel::PkiCaCertificateFilename => Value::plain(CA_CERTIFICATE_FILENAME),
      ProxyBundleLabel::PkiClientCertificateFilename => Value::plain(CLIENT_CERTIFICATE_FILENAME),
      ProxyBundleLabel::PkiClientKeyFilename => Value::plain(CLIENT_KEY_FILENAME),
      ProxyBundleLabel::Platform => Value::target(&self.platform),
      ProxyBundleLabel::PlatformDomain => Value::result(self.domain_from_platform()),
      ProxyBundleLabel::ProxyCommonName => Value::result(self.common_name()),
      ProxyBundleLabel::ProxyName => Value::target(&self.proxy_name),
      ProxyBundleLabel::SchemaStore => {
        if self.enable_schema_store {
          Value::plain("enabled")
        } else {
          Value::plain("disabled")
        }
      }
      ProxyBundleLabel::SchemaStoreEndpoint => {
        if self.enable_schema_store {
          Value::result(self.platform.proxy_schema_store_vhost(&self.tenant, &self.proxy_name, self.vhost_zone.clone()))
        } else {
          Value::hide()
        }
      }
      ProxyBundleLabel::Tenant => Value::target(&self.tenant),
      ProxyBundleLabel::VhostZone => Value::plain(&self.vhost_zone),
      _ => Value::unreachable(),
    }
  }
}
