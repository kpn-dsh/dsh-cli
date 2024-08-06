use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;

pub(crate) const APP_COMMAND: &str = "app";
pub(crate) const APP_ARGUMENT: &str = "app-argument";

pub(crate) fn app_command() -> Command {
  Command::new(APP_COMMAND)
    .about("Show app details")
    .arg_required_else_help(true)
    .args(vec![Arg::new(APP_ARGUMENT)
      .action(ArgAction::Set)
      .value_parser(builder::NonEmptyStringValueParser::new())
      .value_name("APP")
      .help("App name")
      .long_help("App name.")])
    .long_about("Show app details")
}

pub(crate) async fn run_app_command(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> () {
  let resp = dsh_api_client.get_apps().await.unwrap();
  println!("{}", serde_json::to_string_pretty(&resp).unwrap());
}
