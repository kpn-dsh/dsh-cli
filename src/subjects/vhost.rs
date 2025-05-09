use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::subject::{Requirements, Subject};
use crate::subjects::USED_BY_LABELS_LIST;
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{PortMapping, Vhost};
use dsh_api::UsedBy;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::fmt::{Display, Formatter};

pub(crate) struct VhostSubject {}

const VHOST_SUBJECT_TARGET: &str = "vhost";

lazy_static! {
  pub static ref VHOST_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(VhostSubject {});
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
  static ref VHOST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
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

#[derive(Serialize)]
struct VhostListValue {
  vhost: String,
  zone: Option<&'static str>,
  tenant: Option<String>,
  kafka_flag: bool,
  service_id: String,
  instances: u64,
  port: String,
  port_mapping: PortMapping,
}

struct VhostList {}

#[async_trait]
impl CommandExecutor for VhostList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list configured vhosts");
    let start_instant = context.now();
    let applications = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut vhost_list_values = applications
      .iter()
      .filter(|(_, application)| (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped))
      .flat_map(|(application_id, application)| {
        application
          .exposed_ports
          .iter()
          .filter_map(|(port, port_mapping)| match port_mapping.vhost {
            Some(ref vhost_string) => match parse_vhost_string(vhost_string) {
              Ok((vhost, kafka_flag, tenant, zone)) => Some(VhostListValue {
                vhost,
                zone,
                tenant,
                kafka_flag,
                service_id: application_id.to_string(),
                instances: application.instances,
                port: port.to_string(),
                port_mapping: port_mapping.clone(),
              }),
              Err(_) => None,
            },
            None => None,
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    vhost_list_values.sort_by(|a, b| (&a.vhost, &a.service_id).cmp(&(&b.vhost, &b.service_id)));
    let mut formatter = ListFormatter::new(&VHOST_LIST_LABELS, None, context);
    formatter.push_values(&vhost_list_values);
    formatter.print(None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VhostListUsage {}

#[async_trait]
impl CommandExecutor for VhostListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list services with a vhost configuration");
    let start_instant = context.now();
    let vhosts_with_usage: Vec<(String, Vec<UsedBy>)> = client.list_vhosts_with_usage().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("vhost"), context);
    for (vhost, used_bys) in &vhosts_with_usage {
      for used_by in used_bys {
        formatter.push_target_id_value(vhost.clone(), used_by);
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum VhostListLabel {
  Auth,
  Instances,
  KafkaFlag,
  Mode,
  Paths,
  Port,
  _ServiceGroup,
  ServiceId,
  Tenant,
  Tls,
  Vhost,
  _Whitelist,
  Zone,
}

impl Label for VhostListLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Auth => "auth",
      Self::KafkaFlag => "kafka",
      Self::Instances => "#",
      Self::Mode => "mode",
      Self::Paths => "paths",
      Self::Port => "port",
      Self::_ServiceGroup => "service group",
      Self::ServiceId => "service configuration",
      Self::Tenant => "tenant",
      Self::Tls => "tlc",
      Self::Vhost => "vhost",
      Self::_Whitelist => "whitelist",
      Self::Zone => "zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Vhost)
  }
}

impl SubjectFormatter<VhostListLabel> for VhostListValue {
  fn value(&self, label: &VhostListLabel, _target_id: &str) -> String {
    match label {
      VhostListLabel::Auth => self
        .port_mapping
        .auth
        .clone()
        .and_then(|auth| parse_auth_string(&auth))
        .map(|a| a.to_string())
        .unwrap_or_default(),
      VhostListLabel::KafkaFlag => {
        if self.kafka_flag {
          "set".to_string()
        } else {
          "".to_string()
        }
      }
      VhostListLabel::Instances => self.instances.to_string(),
      VhostListLabel::Mode => self.port_mapping.mode.clone().unwrap_or_default(),
      VhostListLabel::Paths => self.port_mapping.paths.iter().map(|path_spec| path_spec.to_string()).collect::<Vec<_>>().join(", "),
      VhostListLabel::Port => self.port.clone(),
      VhostListLabel::_ServiceGroup => self.port_mapping.service_group.clone().unwrap_or_default(),
      VhostListLabel::ServiceId => self.service_id.clone(),
      VhostListLabel::Tenant => self.tenant.clone().unwrap_or_default(),
      VhostListLabel::Tls => self.port_mapping.tls.map(|tls| tls.to_string()).unwrap_or_default(),
      VhostListLabel::Vhost => self.vhost.clone(),
      VhostListLabel::_Whitelist => self.port_mapping.whitelist.clone().unwrap_or_default(),
      VhostListLabel::Zone => self.zone.map(|zone| zone.to_string()).unwrap_or_default(),
    }
  }
}

pub static VHOST_LIST_LABELS: [VhostListLabel; 11] = [
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
];

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum VhostLabel {
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
  fn value(&self, label: &VhostLabel, target_id: &str) -> String {
    match label {
      VhostLabel::Target => target_id.to_string(),
      VhostLabel::Value => self.value.to_string(),
    }
  }
}

pub static VHOST_LABELS: [VhostLabel; 2] = [VhostLabel::Target, VhostLabel::Value];

lazy_static! {
  static ref VHOST_REGEX: Regex = Regex::new(r"\{\s*vhost\(\s*'([a-zA-Z0-9_-]+)(\.kafka)?(?:\.([a-zA-Z0-9_-]+))?'\s*(,\s*'([a-zA-Z0-9_-]+)')?\s*\)\s*\}").unwrap();
}

// Parse the vhost string
//
// Returns a tuple:
// * vhost - name of the vhost
// * kafka - whether the vhost string contains '.kafka'
// * tenant - whether the vhost string contains the tenant name
// * zone - public or private
#[allow(clippy::type_complexity)]
pub(crate) fn parse_vhost_string(vhost_string: &str) -> Result<(String, bool, Option<String>, Option<&'static str>), String> {
  match VHOST_REGEX.captures(vhost_string) {
    Some(captures) => Ok((
      captures.get(1).map(|vhost_match| vhost_match.as_str().to_string()).unwrap_or_default(),
      captures.get(2).is_some(),
      captures.get(3).map(|tenant_match| tenant_match.as_str().to_string()),
      captures.get(4).and_then(|zone_match| {
        let zone_string = zone_match.as_str();
        if zone_string.contains("'private'") {
          Some("private")
        } else if zone_string.contains("'public'") {
          Some("public")
        } else {
          None
        }
      }),
    )),
    None => Err(format!("could not parse vhost string ({})", vhost_string)),
  }
}

enum Authentication {
  Basic(Option<String>, String),
  Fwd(String),
  SystemFwd(String),
}

impl Display for Authentication {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Authentication::Basic(realm, user_name) => match realm {
        Some(realm) => write!(f, "basic@{}:{}", realm, user_name),
        None => write!(f, "basic@{}", user_name),
      },
      Authentication::Fwd(auth_service_endpoint) => write!(f, "fwd@{}", auth_service_endpoint),
      Authentication::SystemFwd(roles) => write!(f, "sys-fwd@{}", roles),
    }
  }
}

// Parse the auth string
fn parse_auth_string(auth_string: &str) -> Option<Authentication> {
  if let Some(basic_authentication_string) = auth_string.strip_prefix("basic-auth@") {
    parse_basic_authentication_string(basic_authentication_string)
  } else if let Some(fwd_auth_string) = auth_string.strip_prefix("fwd-auth@") {
    if let Some([auth_service_endpoint, _]) = fwd_auth_string.split("@").collect_array() {
      Some(Authentication::Fwd(auth_service_endpoint.to_string()))
    } else {
      None
    }
  } else if let Some(roles) = auth_string.strip_prefix("system-fwd-auth@") {
    Some(Authentication::SystemFwd(roles.to_string()))
  } else {
    parse_basic_authentication_string(auth_string)
  }
}

fn parse_basic_authentication_string(basic_authentication_string: &str) -> Option<Authentication> {
  let parts = basic_authentication_string.split(":").collect::<Vec<_>>();
  if parts.len() == 2 {
    Some(Authentication::Basic(None, parts.first().map(|a| a.to_string()).unwrap()))
  } else if parts.len() == 3 {
    Some(Authentication::Basic(
      Some(parts.first().map(|a| a.to_string()).unwrap()),
      parts.get(1).map(|a| a.to_string()).unwrap(),
    ))
  } else {
    None
  }
}
