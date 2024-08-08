use std::collections::HashMap;

use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::tabular::make_tabular_with_headers;

pub(crate) const PROCESSOR_COMMAND: &str = "processor";
const PROCESSOR_ARGUMENT: &str = "processor-argument";

const PROCESSOR_LIST_SUBCOMMAND: &str = "list";

pub(crate) fn processor_command() -> Command {
  Command::new(PROCESSOR_COMMAND)
    .about("Show Trifonius processor details")
    .long_about("Show Trifonius processor details")
    .arg_required_else_help(true)
    .subcommands(vec![processor_list_subcommand()])
}

pub(crate) async fn run_processor_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.subcommand() {
    Some((PROCESSOR_LIST_SUBCOMMAND, sub_matches)) => run_processor_list_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

fn processor_list_subcommand() -> Command {
  Command::new(PROCESSOR_LIST_SUBCOMMAND)
    .about("List Trifonius processors")
    .after_help("List Trifonius processors")
    .after_long_help("List Trifonius processors.")
    .args(vec![processor_argument()])
}

const TRIFONIUS_PIPELINE_NAME: &str = "TRIFONIUS_PIPELINE_NAME";
const TRIFONIUS_PROCESSOR_ID: &str = "TRIFONIUS_PROCESSOR_ID";
const TRIFONIUS_PROCESSOR_NAME: &str = "TRIFONIUS_PROCESSOR_NAME";
const TRIFONIUS_PROCESSOR_TYPE: &str = "TRIFONIUS_PROCESSOR_TYPE";
const TRIFONIUS_SERVICE_NAME: &str = "TRIFONIUS_SERVICE_NAME";

async fn run_processor_list_subcommand(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match dsh_api_client.get_applications().await {
    Ok(applications) => {
      let mut table: Vec<Vec<String>> = vec![];
      for (application_id, application) in applications {
        if let Some(trifonius_parameters) = find_trifonius_parameters(&application) {
          let parameters = vec![
            application_id,
            trifonius_parameters.get(TRIFONIUS_PIPELINE_NAME).cloned().unwrap_or("-".to_string()),
            trifonius_parameters.get(TRIFONIUS_PROCESSOR_NAME).cloned().unwrap_or("-".to_string()),
          ];
          table.push(parameters);
        }
      }
      for line in make_tabular_with_headers(vec!["application", "pipeline", "processor"], table) {
        println!("{}", line)
      }
    }
    Err(error) => println!("unexpected error {}", error),
  }
}

fn find_trifonius_parameters(application: &Application) -> Option<HashMap<&'static str, String>> {
  let mut parameters: HashMap<&'static str, String> = HashMap::new();
  if let Some(pipeline_name) = application.env.get(TRIFONIUS_PIPELINE_NAME) {
    parameters.insert(TRIFONIUS_PIPELINE_NAME, pipeline_name.to_string());
  }
  if let Some(processor_name) = application.env.get(TRIFONIUS_PROCESSOR_ID) {
    parameters.insert(TRIFONIUS_PROCESSOR_ID, processor_name.to_string());
  }
  if let Some(processor_name) = application.env.get(TRIFONIUS_PROCESSOR_NAME) {
    parameters.insert(TRIFONIUS_PROCESSOR_NAME, processor_name.to_string());
  }
  if let Some(processor_name) = application.env.get(TRIFONIUS_PROCESSOR_TYPE) {
    parameters.insert(TRIFONIUS_PROCESSOR_TYPE, processor_name.to_string());
  }
  if let Some(service_name) = application.env.get(TRIFONIUS_SERVICE_NAME) {
    parameters.insert(TRIFONIUS_SERVICE_NAME, service_name.to_string());
  }
  if parameters.is_empty() {
    None
  } else {
    Some(parameters)
  }
}

fn processor_argument() -> Arg {
  Arg::new(PROCESSOR_ARGUMENT)
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VHOST")
    .help("Vhost")
    .long_help("Vhost.")
}
