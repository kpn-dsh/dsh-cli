use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;

use crate::{to_command_error, CommandResult};

pub(crate) const TASK_COMMAND: &str = "task";
pub(crate) const TASK_ARGUMENT: &str = "task-argument";

pub(crate) fn task_command() -> Command {
  Command::new(TASK_COMMAND)
    .about("Show task details")
    .args(vec![Arg::new(TASK_ARGUMENT)
      .action(ArgAction::Set)
      .value_parser(builder::NonEmptyStringValueParser::new())
      .value_name("APP")
      .help("App name")
      .long_help("App name.")])
    .arg_required_else_help(true)
    .long_about("Show app details")
}

pub(crate) async fn run_task_command(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match dsh_api_client.get_apps().await {
    Ok(resp) => {
      println!("{}", serde_json::to_string_pretty(&resp).unwrap());
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}
