use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::PortMapping;
use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::subject::Subject;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

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
    capabilities.insert(CapabilityType::Show, VHOST_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
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

struct VhostShowUsage {}

#[async_trait]
impl CommandExecutor for VhostShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let vhost_id = target.unwrap_or_else(|| unreachable!());
    let applications = dsh_api_client.get_application_configurations().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for (application_id, application) in applications {
      for (port, port_mapping) in application.exposed_ports {
        if port_mapping
          .vhost
          .clone()
          .is_some_and(|ref vh| vh.contains(&format!("'{}'", vhost_id)) || vh.contains(&format!("'{}.", vhost_id)))
        {
          table.push(vec![application_id.clone(), port, PortMappingWrapper(port_mapping).to_string()]);
        }
      }
    }
    for line in make_tabular_with_headers(&["application", "port", "port mapping"], table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct PortMappingWrapper(PortMapping);

impl Display for PortMappingWrapper {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self.0.vhost {
      Some(vhost) => write!(f, "{}", vhost),
      None => write!(f, "no-vhost"),
    }
  }
}
