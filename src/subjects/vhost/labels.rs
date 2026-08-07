use crate::formatters::{Label, SubjectFormatter, Value};
use dsh_api::parse::AuthString;
use dsh_api::platform::VhostZone;
use dsh_api::types::{PortMapping, Vhost};
use itertools::Itertools;
use serde::Serialize;
use std::str::FromStr;

#[derive(Clone, Serialize)]
pub(crate) struct VhostValue {
  pub(crate) vhost: String,
  pub(crate) zone: Option<VhostZone>,
  pub(crate) tenant: Option<String>,
  pub(crate) kafka_flag: bool,
  pub(crate) service_id: String,
  pub(crate) instances: u64,
  pub(crate) port: String,
  pub(crate) port_mapping: PortMapping,
  pub(crate) url: Option<String>,
  pub(crate) cert: Option<String>,
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum VhostListLabel {
  Auth,
  Cert,
  Instances,
  Kind,
  Mode,
  Paths,
  Port,
  _ServiceGroup,
  ServiceId,
  Tenant,
  Tls,
  Url,
  Vhost,
  Whitelist,
  Zone,
}

impl Label for VhostListLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Auth => "auth",
      Self::Cert => "cert",
      Self::Instances => "#",
      Self::Kind => "kind",
      Self::Mode => "mode",
      Self::Paths => "paths",
      Self::Port => "port",
      Self::_ServiceGroup => "service group",
      Self::ServiceId => "service configuration",
      Self::Tenant => "tenant",
      Self::Tls => "tlc",
      Self::Url => "url",
      Self::Vhost => "vhost",
      Self::Whitelist => "whitelist",
      Self::Zone => "zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Vhost)
  }
}

impl SubjectFormatter<VhostListLabel> for VhostValue {
  fn value(&self, label: &VhostListLabel, _target_id: &str) -> Value {
    match label {
      VhostListLabel::Auth => Value::some_or_hide(self.port_mapping.auth.clone().and_then(|auth| AuthString::from_str(&auth).ok())),
      VhostListLabel::Cert => match &self.cert {
        Some(cert) => Value::target(cert),
        None => Value::hide(),
      },
      VhostListLabel::Instances => Value::plain(self.instances),
      VhostListLabel::Kind => {
        if self.kafka_flag {
          Value::plain("proxy")
        } else {
          Value::plain("service")
        }
      }
      VhostListLabel::Mode => Value::some_or_hide(self.port_mapping.mode.as_ref()),
      VhostListLabel::Paths => Value::plain(self.port_mapping.paths.iter().map(|path_spec| path_spec.to_string()).join(", ")),
      VhostListLabel::Port => Value::plain(&self.port),
      VhostListLabel::_ServiceGroup => Value::some_or_hide(self.port_mapping.service_group.as_ref()),
      VhostListLabel::ServiceId => Value::target(&self.service_id),
      VhostListLabel::Tenant => Value::some_or_hide(self.tenant.as_ref()),
      VhostListLabel::Tls => Value::some_or_hide(self.port_mapping.tls),
      VhostListLabel::Url => Value::some_or(self.url.clone(), Value::warn("zone not specified")),
      VhostListLabel::Vhost => Value::plain(&self.vhost),
      VhostListLabel::Whitelist => Value::some_or_hide(self.port_mapping.whitelist.as_ref()),
      VhostListLabel::Zone => Value::some_or(self.zone.as_ref(), Value::warn("none")),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum VhostLabel {
  Target,
  Value,
}

impl Label for VhostLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Target => "vhost id",
      Self::Value => "vhost",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<VhostLabel> for Vhost {
  fn value(&self, label: &VhostLabel, target_id: &str) -> Value {
    match label {
      VhostLabel::Target => Value::target(target_id),
      VhostLabel::Value => Value::plain(&self.value),
    }
  }
}

// #[derive(Eq, Hash, PartialEq, Serialize)]
// pub(crate) enum VhostBundleLabel {
//   BundleDirectory,
//   BundleName,
//   CaCommonName,
//   PkiClientKeyFilename,
//   PkiCsrFilename,
//   Platform,
//   PlatformDomain,
//   Tenant,
//   VhostZone,
// }

// impl Label for VhostBundleLabel {
//   fn as_str(&self) -> &str {
//     match self {
//       Self::BundleDirectory => "bundle directory",
//       Self::BundleName => "bundle",
//       Self::CaCommonName => "ca common name",
//       Self::PkiClientKeyFilename => "client key file",
//       Self::PkiCsrFilename => "csr file",
//       Self::Platform => "platform",
//       Self::PlatformDomain => "platform domain",
//       Self::Tenant => "tenant",
//       Self::VhostZone => "vhost zone",
//     }
//   }
//
//   fn is_target_label(&self) -> bool {
//     matches!(self, VhostBundleLabel::BundleName)
//   }
// }

// impl SubjectFormatter<VhostBundleLabel> for (VhostCertificateBundleConfig, String) {
//   fn value(&self, label: &VhostBundleLabel, target_id: &str) -> Value {
//     let (config, directory) = self;
//     match label {
//       VhostBundleLabel::BundleDirectory => Value::plain(directory),
//       _ => config.value(label, target_id),
//     }
//   }
// }

// impl SubjectFormatter<VhostBundleLabel> for VhostCertificateBundleConfig {
//   fn value(&self, label: &VhostBundleLabel, target_id: &str) -> Value {
//     match label {
//       VhostBundleLabel::BundleName => Value::target(target_id),
//       VhostBundleLabel::CaCommonName => Value::plain(&self.ca_common_name),
//       VhostBundleLabel::PkiClientKeyFilename => Value::plain(CLIENT_KEY_FILENAME),
//       VhostBundleLabel::PkiCsrFilename => Value::plain(CSR_FILENAME),
//       VhostBundleLabel::Platform => Value::target(&self.platform),
//       VhostBundleLabel::PlatformDomain => Value::result(self.domain_from_platform()),
//       VhostBundleLabel::Tenant => Value::target(&self.tenant),
//       VhostBundleLabel::VhostZone => Value::plain(&self.vhost_zone),
//       _ => Value::unreachable(),
//     }
//   }
// }

#[cfg(feature = "rock")]
#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum RockCertificateLabel {
  AdministrativeGroup,
  AltNames,
  CommonName,
  ConnectorName,
  Id,
  ManagedByGroup,
  NotAfter,
  NotBefore,
  Status,
}

#[cfg(feature = "rock")]
impl Label for RockCertificateLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AdministrativeGroup => "administrative group",
      Self::AltNames => "alt names",
      Self::CommonName => "common name",
      Self::ConnectorName => "connector name",
      Self::Id => "id",
      Self::ManagedByGroup => "managed by group",
      Self::NotAfter => "not after",
      Self::NotBefore => "not before",
      Self::Status => "status",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, RockCertificateLabel::Id)
  }
}

#[cfg(feature = "rock")]
impl SubjectFormatter<RockCertificateLabel> for (&rock_api::types::Certificate, Option<u64>) {
  fn value(&self, label: &RockCertificateLabel, _target_id: &str) -> Value {
    use chrono::{DateTime, Utc};
    use time::UtcDateTime;
    fn time_to_chrono(time: &UtcDateTime) -> DateTime<Utc> {
      DateTime::from_timestamp_secs(time.unix_timestamp()).unwrap_or_default()
    }
    let (certificate, days): (&rock_api::types::Certificate, Option<u64>) = *self;
    match label {
      RockCertificateLabel::AdministrativeGroup => Value::plain(certificate.ad_group.clone()),
      RockCertificateLabel::AltNames => match &certificate.alt_names {
        Some(alt_names) => Value::non_empty_or_hide(&alt_names.iter().map(|alt_name| alt_name.cn.to_string()).collect_vec()),
        None => Value::hide(),
      },
      RockCertificateLabel::CommonName => Value::plain(certificate.cn.clone()),
      RockCertificateLabel::ConnectorName => Value::plain(certificate.connector_name.clone()),
      RockCertificateLabel::Id => Value::plain(certificate.id),
      RockCertificateLabel::ManagedByGroup => Value::plain(certificate.managed_by_group.clone()),
      RockCertificateLabel::NotAfter => Value::datetime_expired(&time_to_chrono(&certificate.not_after), days),
      RockCertificateLabel::NotBefore => Value::datetime(&time_to_chrono(&certificate.not_before)),
      RockCertificateLabel::Status => Value::plain(certificate.status.clone()),
    }
  }
}
