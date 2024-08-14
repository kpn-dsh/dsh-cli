use std::fmt::{Display, Formatter};

use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::types::PortMapping;
use trifonius_dsh_api::DshApiClient;

use crate::tabular::make_tabular_with_headers;
use crate::{to_command_error_missing_id, to_command_error_with_id, CommandResult};

pub(crate) const VHOST_COMMAND: &str = "vhost";
const VHOST_ARGUMENT: &str = "vhost-argument";

const WHAT: &str = "vhost";

const VHOST_USAGE_SUBCOMMAND: &str = "usage";

pub(crate) fn vhost_command() -> Command {
  Command::new(VHOST_COMMAND)
    .about("Show vhost details")
    .alias("v")
    .long_about("Show vhost details")
    .arg_required_else_help(true)
    .subcommands(vec![vhost_usage_subcommand()])
}

fn vhost_usage_subcommand() -> Command {
  Command::new(VHOST_USAGE_SUBCOMMAND)
    .about("Show vhost usage")
    .after_help("Show vhost usage")
    .after_long_help("Show vhost usage.")
    .args(vec![vhost_argument()])
}

pub(crate) async fn run_vhost_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.subcommand() {
    Some((VHOST_USAGE_SUBCOMMAND, sub_matches)) => run_vhost_usage_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

async fn run_vhost_usage_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(VHOST_ARGUMENT) {
    Some(vhost_id) => match dsh_api_client.get_applications().await {
      Ok(applications) => {
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
      Err(error) => to_command_error_with_id(error, WHAT, vhost_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

fn vhost_argument() -> Arg {
  Arg::new(VHOST_ARGUMENT)
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VHOST")
    .help("Vhost")
    .long_help("Vhost.")
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
