use async_trait::async_trait;

use clap::ArgMatches;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use trifonius_dsh_api::dsh_api_client::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::formatter::StringTableBuilder;
use crate::subject::Subject;
use crate::{TcliContext, TcliResult};

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

  fn subject_first_upper(&self) -> &'static str {
    "Vhost"
  }

  fn subject_command_about(&self) -> String {
    "Show vhost usage.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show which DSH component use a vhost.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("v")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, VHOST_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, VHOST_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref VHOST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List configured vhosts".to_string(),
    command_long_about: Some(
      "List applications that have vhosts configured. Vhosts that are provisioned but are not configured in any applications will not be shown.".to_string()
    ),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::Usage, &VhostListUsage {}, None)],
    default_command_executor: Some(&VhostListUsage {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref VHOST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show vhost usage".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::Usage, &VhostShowUsage {}, None)],
    default_command_executor: Some(&VhostShowUsage {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct VhostListUsage {}

#[async_trait]
impl CommandExecutor for VhostListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &TcliContext, dsh_api_client: &DshApiClient<'_>) -> TcliResult {
    if context.show_capability_explanation() {
      println!("list applications with a vhost configuration");
    }
    let applications = dsh_api_client.get_application_configurations().await?;
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
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &TcliContext, dsh_api_client: &DshApiClient<'_>) -> TcliResult {
    let vhost_target = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the applications that use vhost '{}'", vhost_target);
    }
    let applications = dsh_api_client.get_application_configurations().await?;
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
    builder.print_list();
    Ok(false)
  }
}

lazy_static! {
  static ref VHOST_REGEX: Regex = Regex::new(r"\{\s*vhost\(\s*'([a-zA-Z0-9_\.-]+)'\s*(,\s*'([a-zA-Z0-9_-]+)')?\s*\)\s*\}").unwrap();
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

pub(crate) fn parse_vhost_string(vhost_string: &str) -> Option<(String, Option<String>)> {
  VHOST_REGEX.captures(vhost_string).map(|captures| {
    (
      captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
      captures.get(2).and_then(|m| a_zone(m.as_str().to_string())),
    )
  })
}

// #[test]
// fn test_parse_vhost() {
//   const VHOST_STRINGS: [&str; 26] = [
//     "{ vhost('broker-schema-store.kafka.greenbox-dev', 'private') }",
//     "{ vhost('cmd.greenbox-dev', 'private') }",
//     "{ vhost('dsh-schema-store.greenbox-dev','public') }",
//     "{ vhost('eavesdropper-092.greenbox-dev', 'public') }",
//     "{ vhost('simple-keyring-service', 'private') }",
//     "{ vhost('simple-keyring-service', 'private') }",
//     "{ vhost('greenbox-lab','private') }",
//     "{ vhost('monitor-dashboard','public') }",
//     "{ vhost('greenbox-backend-dev') }",
//     "{ vhost('greenbox-tutorial-jupyter', 'public') }",
//     "{ vhost('greenbox','public') }",
//     "{ vhost('keyring-050.greenbox-dev', 'public') }",
//     "{ vhost('keyring-app2.greenbox-dev', 'public') }",
//     "{ vhost('keyring-dev','public') }",
//     "{ vhost('keyring-dev-proxy.greenbox-dev', 'public') }",
//     "{ vhost('keyring-documentation', 'public') }",
//     "{ vhost('keyring-service-dev') }",
//     "{ vhost('simple-keyring-service','private') }",
//     "{ vhost('keyring-dex','private') }",
//     "{ vhost('schema-registry-dev') }",
//     "{ vhost('schema-registry-console-dev', 'public') }",
//     "{ vhost('schema-registry-doc-dev') }",
//     "{ vhost('trifonius-be','private') }",
//     "{ vhost('greenbox-dev','public') }",
//     "{ vhost('cmd.greenbox-dev','private') }",
//     "{ vhost('whoami.greenbox-dev', 'public') }",
//   ];
//   for vhost_string in VHOST_STRINGS {
//     match parse_vhost_string(vhost_string) {
//       Some((vhost, a_zone)) => println!("{} -> {} -> {:?}", vhost_string, vhost, a_zone),
//       None => println!(">>>>>>>>>> {}", vhost_string),
//     }
//   }
// }
