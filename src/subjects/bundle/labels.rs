use crate::formatters::Value;
use crate::formatters::{Label, SubjectFormatter};
use crate::proxy_bundles::ProxyCertificateBundleConfig;

use serde::Serialize;

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
