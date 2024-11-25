use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
use regex::Regex;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::formatters::formatter::StringTableBuilder;
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

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

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, VHOST_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, VHOST_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref VHOST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List configured vhosts")
      .set_long_about("List applications that have vhosts configured. Vhosts that are provisioned but are not configured in any applications will not be shown.")
      .set_default_command_executor(&VhostListUsage {})
  );
  pub static ref VHOST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show vhost usage")
      .set_default_command_executor(&VhostShowUsage {})
      .add_target_argument(target_argument(VHOST_SUBJECT_TARGET, None))
  );
}

struct VhostListUsage {}

#[async_trait]
impl CommandExecutor for VhostListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list applications with a vhost configuration");
    }
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    application_ids.sort();
    let mut inverse = HashMap::<String, Vec<(String, String, String)>>::new();
    for application_id in &application_ids {
      let application = applications.get(application_id).unwrap();
      for (port, port_mapping) in &application.exposed_ports {
        if let Some(vhost_string) = port_mapping.vhost.clone() {
          if let Some((vhost, a_zone)) = parse_vhost_string(&vhost_string) {
            if let Some(v) = inverse.get_mut(&vhost) {
              v.push((application_id.clone(), port.clone(), a_zone.unwrap_or_default()));
            } else {
              inverse.insert(vhost, vec![(application_id.clone(), port.clone(), a_zone.unwrap_or_default())]);
            }
          }
        }
      }
    }
    let mut vhosts = inverse.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    vhosts.sort();
    let mut builder = StringTableBuilder::new(&["vhost", "application", "port", "a-zone"], context);
    for vhost in &vhosts {
      let mut first = true;
      for vec in inverse.get(vhost).unwrap() {
        if first {
          builder.vec(&vec![vhost.clone(), vec.0.clone(), vec.1.clone(), vec.2.clone()]);
        } else {
          builder.vec(&vec!["".to_string(), vec.0.clone(), vec.1.clone(), vec.2.clone()]);
        }
        first = false;
      }
    }
    builder.print_list();
    Ok(false)
  }
}

struct VhostShowUsage {}

#[async_trait]
impl CommandExecutor for VhostShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let vhost_target = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the applications that use vhost '{}'", vhost_target);
    }
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    let mut builder = StringTableBuilder::new(&["application", "port", "a-zone"], context);
    for (application_id, application) in &applications {
      for (port, port_mapping) in &application.exposed_ports {
        if let Some(vhost_string) = port_mapping.vhost.clone() {
          if let Some((vhost, a_zone)) = parse_vhost_string(&vhost_string) {
            if vhost_target == vhost {
              builder.vec(&vec![application_id.clone(), port.clone(), a_zone.unwrap_or_default()]);
            }
          }
        }
      }
    }
    builder.print_show();
    Ok(false)
  }
}

fn a_zone(a_zone_string: String) -> Option<String> {
  if a_zone_string.contains("'private'") {
    Some("private".to_string())
  } else if a_zone_string.contains("'public'") {
    Some("public".to_string())
  } else {
    None
  }
}

lazy_static! {
  static ref VHOST_REGEX: Regex = Regex::new(r"\{\s*vhost\(\s*'([a-zA-Z0-9_\.-]+)'\s*(,\s*'([a-zA-Z0-9_-]+)')?\s*\)\s*\}").unwrap();
}

pub(crate) fn parse_vhost_string(vhost_string: &str) -> Option<(String, Option<String>)> {
  VHOST_REGEX.captures(vhost_string).map(|captures| {
    (
      captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
      captures.get(2).and_then(|m| a_zone(m.as_str().to_string())),
    )
  })
}
