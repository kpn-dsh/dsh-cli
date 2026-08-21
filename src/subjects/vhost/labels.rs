use crate::formatters::{Label, SubjectFormatter, Value};
use dsh_api::parse::AuthString;
use dsh_api::platform::VhostZone;
use dsh_api::types::{PortMapping, Vhost};
use itertools::Itertools;
use serde::Serialize;
use std::str::FromStr;

#[derive(Clone, Serialize)]
pub(crate) struct VhostValue {
  pub(crate) cert: Option<String>,
  pub(crate) domain: Option<String>,
  pub(crate) instances: u64,
  pub(crate) kafka_flag: bool,
  pub(crate) port: String,
  pub(crate) port_mapping: PortMapping,
  pub(crate) service_id: String,
  pub(crate) tenant: Option<String>,
  pub(crate) vhost: String,
  pub(crate) zone: Option<VhostZone>,
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum VhostListLabel {
  Auth,
  Cert,
  Domain,
  Instances,
  Kind,
  Mode,
  Paths,
  Port,
  _ServiceGroup,
  ServiceId,
  Tenant,
  Tls,
  Vhost,
  Whitelist,
  Zone,
}

impl Label for VhostListLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Auth => "auth",
      Self::Cert => "cert",
      Self::Domain => "domain",
      Self::Instances => "#",
      Self::Kind => "kind",
      Self::Mode => "mode",
      Self::Paths => "paths",
      Self::Port => "port",
      Self::_ServiceGroup => "service group",
      Self::ServiceId => "service configuration",
      Self::Tenant => "tenant",
      Self::Tls => "tlc",
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
      VhostListLabel::Domain => Value::some_or(self.domain.clone(), Value::warn("zone not specified")),
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
