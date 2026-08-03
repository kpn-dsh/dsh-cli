pub(crate) mod capabilities;
pub(crate) mod labels;

use crate::capability::{Capability, LIST_COMMAND, LIST_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::subject::Subject;
use crate::subjects::vhost::capabilities::{VhostList, VhostListUsage};
use crate::subjects::vhost::labels::VhostListLabel;
use async_trait::async_trait;
use dsh_api::types::PortMapping;
use serde::Serialize;
use std::sync::LazyLock;

struct VhostSubject {}

const VHOST_SUBJECT_TARGET: &str = "vhost";

pub(crate) static VHOST_SUBJECT: LazyLock<Box<dyn Subject + Send + Sync>> = LazyLock::new(|| Box::new(VhostSubject {}));

#[async_trait]
impl Subject for VhostSubject {
  fn subject(&self) -> &'static str {
    VHOST_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show vhost usage.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show which DSH components use a vhost.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("v")
  }

  #[cfg(feature = "rock")]
  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    use crate::capability::{ADD_COMMAND, UPDATE_COMMAND};

    match capability_command {
      ADD_COMMAND => Some(ADD_CERTIFICATE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(VHOST_LIST_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(UPDATE_CERTIFICATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  #[cfg(not(feature = "rock"))]
  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(VHOST_LIST_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &VHOST_CAPABILITIES
  }
}

#[cfg(feature = "rock")]
static ADD_CERTIFICATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  use crate::arguments::vhost_subdomain_argument;
  use crate::capability::{ADD_COMMAND, ADD_COMMAND_ALIAS};
  use crate::global_options::expiration_option;
  use crate::subjects::proxy::options::vhost_zone_option;
  use crate::subjects::vhost::capabilities::VhostAddCertificate;

  Box::new(
    CapabilityBuilder::new(ADD_COMMAND, Some(ADD_COMMAND_ALIAS), &VhostAddCertificate {}, "Add certificate to private vhost")
      .add_target_argument(vhost_subdomain_argument().required(true))
      .add_extra_argument(certificate_authority_argument())
      .add_extra_argument(expiration_option())
      .add_extra_argument(vhost_zone_option()),
  )
});

static VHOST_LIST_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &VhostList {}, "List used vhosts")
      .set_long_about(
        "List vhosts that have been configured in one or more services. Vhosts that are \
       provisioned but are not configured in any services will not be shown.",
      )
      .add_command_executor(FlagType::Usage, &VhostListUsage {}, None)
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List vhosts configured in started services.".to_string())),
        (FilterFlagType::Stopped, Some("List vhosts configured in stopped services.".to_string())),
      ]),
  )
});

#[cfg(feature = "rock")]
static UPDATE_CERTIFICATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  use crate::arguments::vhost_subdomain_argument;
  use crate::capability::UPDATE_COMMAND;
  use crate::subjects::vhost::capabilities::VhostUpdateCertificate;

  Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &VhostUpdateCertificate {}, "Update certificate for private vhost").add_target_argument(vhost_subdomain_argument().required(true)),
  )
});

static VHOST_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
    #[cfg(feature = "rock")]
    ADD_CERTIFICATE_CAPABILITY.as_ref(),
    #[cfg(feature = "rock")]
    UPDATE_CERTIFICATE_CAPABILITY.as_ref(),
    VHOST_LIST_CAPABILITY.as_ref(),
  ]
});

#[cfg(feature = "rock")]
pub(crate) const CERTIFICATE_AUTHORITY_ARGUMENT: &str = "certificate-authority-argument";

#[cfg(feature = "rock")]
pub(crate) fn certificate_authority_argument() -> clap::Arg {
  use crate::bundle::CertificateAuthority;
  use clap::builder::EnumValueParser;

  clap::Arg::new(CERTIFICATE_AUTHORITY_ARGUMENT)
    .long("certificate-authority")
    .alias("ca")
    .action(clap::ArgAction::Set)
    .value_parser(EnumValueParser::<CertificateAuthority>::new())
    .value_name("CA")
    .long_help(
      "Use this option to specify the certificate authority that will be used to sign \
      proxy or vhost certificates. When this option is not provided --self-signed will be \
      assumed.",
    )
}

#[derive(Clone, Serialize)]
struct VhostListValue {
  vhost: String,
  zone: Option<String>,
  tenant: Option<String>,
  kafka_flag: bool,
  service_id: String,
  instances: u64,
  port: String,
  port_mapping: PortMapping,
}

static VHOST_LIST_LABELS: [VhostListLabel; 12] = [
  VhostListLabel::Vhost,
  VhostListLabel::Zone,
  VhostListLabel::ServiceId,
  VhostListLabel::Port,
  VhostListLabel::Instances,
  VhostListLabel::Auth,
  VhostListLabel::Tenant,
  VhostListLabel::Mode,
  VhostListLabel::Paths,
  VhostListLabel::Tls,
  VhostListLabel::KafkaFlag,
  VhostListLabel::Whitelist,
];
