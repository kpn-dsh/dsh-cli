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
use lazy_static::lazy_static;
use serde::Serialize;

struct VhostSubject {}

const VHOST_SUBJECT_TARGET: &str = "vhost";

lazy_static! {
  pub(crate) static ref VHOST_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(VhostSubject {});
}

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

lazy_static! {
  static ref VHOST_LIST_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &VhostList {}, "List used vhosts")
      .set_long_about(
        "List vhosts that have been configured in one or more services. Vhosts that are \
       provisioned but are not configured in any services will not be shown."
      )
      .add_command_executor(FlagType::Usage, &VhostListUsage {}, None)
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List vhosts configured in started services.".to_string())),
        (FilterFlagType::Stopped, Some("List vhosts configured in stopped services.".to_string()))
      ])
  );
  static ref VHOST_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![VHOST_LIST_CAPABILITY.as_ref()];
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
