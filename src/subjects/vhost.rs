use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::subject::Subject;
use crate::subjects::USED_BY_LABELS_LIST;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::Vhost;
use dsh_api::UsedBy;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

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

  fn requires_dsh_api_client(&self, _sub_matches: &ArgMatches) -> bool {
    true
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
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List configured vhosts")
      .set_long_about("List applications that have vhosts configured. Vhosts that are provisioned but are not configured in any applications will not be shown.")
      .set_default_command_executor(&VhostListUsage {})
  );
  // pub static ref VHOST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
  //   CapabilityBuilder::new(CapabilityType::Show, "Show vhost usage")
  //     .set_default_command_executor(&VhostShowUsage {})
  //     .add_target_argument(target_argument(VHOST_SUBJECT_TARGET, None))
  // );
  static ref VHOST_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![VHOST_LIST_CAPABILITY.as_ref()];
}

struct VhostListUsage {}

#[async_trait]
impl CommandExecutor for VhostListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list applications with a vhost configuration");
    let start_instant = Instant::now();
    let vhosts_with_usage: Vec<(String, Vec<UsedBy>)> = context.dsh_api_client.as_ref().unwrap().list_vhosts_with_usage().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("vhost"), context);
    for (vhost, used_bys) in &vhosts_with_usage {
      let mut first = true;
      for used_by in used_bys {
        if first {
          formatter.push_target_id_value(vhost.clone(), used_by);
        } else {
          formatter.push_target_id_value("".to_string(), used_by);
        }
        first = false;
      }
    }
    formatter.print()?;
    Ok(())
  }
}

// struct VhostShowUsage {}

// #[async_trait]
// impl CommandExecutor for VhostShowUsage {
//   async fn execute(&self, _target: Option<String>, _: Option<String>, _: &ArgMatches, _context: &Context) -> DshCliResult {
// let vhost_target = target.unwrap_or_else(|| unreachable!());
// context.print_explanation(format!("show the applications that use vhost '{}'", vhost_target));
// let start_instant = Instant::now();
// let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
// context.print_execution_time(start_instant);
// let mut builder = StringTableBuilder::new(&["application", "port", "a-zone"], context);
// for (application_id, application) in &applications {
//   for (port, port_mapping) in &application.exposed_ports {
//     if let Some(vhost_string) = port_mapping.vhost.clone() {
//       if let Some((vhost, a_zone)) = vhost::parse_vhost_string(&vhost_string) {
//         if vhost_target == vhost {
//           builder.vec(&vec![application_id.clone(), port.clone(), a_zone.unwrap_or_default()]);
//         }
//       }
//     }
//   }
// }
// builder.print_show();
// Ok(())
// }
// }

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

  fn target_label(&self) -> Option<VhostLabel> {
    Some(VhostLabel::Target)
  }
}

pub static VHOST_LABELS: [VhostLabel; 2] = [VhostLabel::Target, VhostLabel::Value];
