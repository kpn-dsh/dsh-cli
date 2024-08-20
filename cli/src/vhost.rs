use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::PortMapping;
use trifonius_dsh_api::DshApiClient;

use crate::arguments::Flag;
use crate::command::SubjectCommand;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

pub(crate) struct VhostCommand {}

lazy_static! {
  pub static ref VHOST_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(VhostCommand {});
}

#[async_trait]
impl SubjectCommand for VhostCommand {
  fn subject(&self) -> &'static str {
    "vhost"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Vhost"
  }

  fn about(&self) -> String {
    "Show usage of vhosts.".to_string()
  }

  fn long_about(&self) -> String {
    "Show usage of hosts managed by the DSH. Note that at this time, vhosts can not be listed by this tool.".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("v")
  }

  // Vhost command does _not_ support list command
  fn supports_list(&self) -> bool {
    false
  }

  fn show_flags(&self) -> &'static [Flag] {
    &[Flag::Usage]
  }

  async fn show_default(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.show_usage(target_id, matches, dsh_api_client).await
  }

  async fn show_usage(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = dsh_api_client.get_application_configurations().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for (application_id, application) in applications {
      for (port, port_mapping) in application.exposed_ports {
        if port_mapping
          .vhost
          .clone()
          .is_some_and(|ref vh| vh.contains(&format!("'{}'", target_id)) || vh.contains(&format!("'{}.", target_id)))
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
