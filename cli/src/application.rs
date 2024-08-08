use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;
use trifonius_dsh_api::DshApiError;

use crate::tabular::make_tabular_with_headers;

pub(crate) const APPLICATION_COMMAND: &str = "application";
const APPLICATION_ARGUMENT: &str = "application-argument";

const APPLICATION_LIST_SUBCOMMAND: &str = "list";
const APPLICATION_SHOW_SUBCOMMAND: &str = "show";
const APPLICATION_STATUS_SUBCOMMAND: &str = "status";
const APPLICATION_TASKS_SUBCOMMAND: &str = "tasks";

pub(crate) fn application_command() -> Command {
  Command::new(APPLICATION_COMMAND)
    .alias("a")
    .about("Show application details")
    .long_about("Show application details")
    .arg_required_else_help(true)
    .subcommands(vec![
      application_list_subcommand(),
      application_show_subcommand(),
      application_status_subcommand(),
      application_tasks_subcommand(),
    ])
}

pub(crate) async fn run_application_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.subcommand() {
    Some((APPLICATION_LIST_SUBCOMMAND, sub_matches)) => run_application_list_subcommand(sub_matches, dsh_api_client).await,
    Some((APPLICATION_SHOW_SUBCOMMAND, sub_matches)) => run_application_show_subcommand(sub_matches, dsh_api_client).await,
    Some((APPLICATION_STATUS_SUBCOMMAND, sub_matches)) => run_application_status_subcommand(sub_matches, dsh_api_client).await,
    Some((APPLICATION_TASKS_SUBCOMMAND, sub_matches)) => run_application_tasks_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

fn application_list_subcommand() -> Command {
  Command::new(APPLICATION_LIST_SUBCOMMAND)
    .about("List applications")
    .after_help("List applications")
    .after_long_help("List applications.")
}

fn application_show_subcommand() -> Command {
  Command::new(APPLICATION_SHOW_SUBCOMMAND)
    .about("Show application details")
    .after_help("Show application details")
    .after_long_help("Show application details.")
    .args(vec![application_argument()])
}

fn application_status_subcommand() -> Command {
  Command::new(APPLICATION_STATUS_SUBCOMMAND)
    .about("Show application status")
    .after_help("Show application status")
    .after_long_help("Show application status.")
    .args(vec![application_argument()])
}

fn application_tasks_subcommand() -> Command {
  Command::new(APPLICATION_TASKS_SUBCOMMAND)
    .about("Show application tasks")
    .after_help("Show application tasks")
    .after_long_help("Show application tasks.")
    .args(vec![application_argument()])
}

async fn run_application_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.get_one::<String>(APPLICATION_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_application(application_id).await {
      Ok(application) => println!("{}", serde_json::to_string_pretty(&application).unwrap()),
      Err(DshApiError::NotFound) => println!("application id {} does not exist", application_id),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing application id"),
  }
}

async fn run_application_list_subcommand(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match dsh_api_client.get_applications().await {
    Ok(applications) => {
      let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      application_ids.sort();

      let mut table: Vec<Vec<String>> = vec![];
      for application_id in application_ids {
        table.push(vec![application_id.clone(), applications.get(&application_id).unwrap().image.clone()]);
      }
      for line in make_tabular_with_headers(vec!["application", "image"], table) {
        println!("{}", line)
      }
    }
    Err(error) => println!("unexpected error {}", error),
  }
}

async fn run_application_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.get_one::<String>(APPLICATION_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_application_status(application_id).await {
      Ok(application_status) => println!("{}", serde_json::to_string_pretty(&application_status).unwrap()),
      Err(DshApiError::NotFound) => println!("application id {} does not exist", application_id),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing application id"),
  }
}

async fn run_application_tasks_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.get_one::<String>(APPLICATION_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_tasks(application_id).await {
      Ok(application_tasks) => {
        for task_id in application_tasks {
          println!("{}", task_id)
        }
      }
      Err(DshApiError::NotFound) => println!("application id {} does not exist", application_id),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing application id"),
  }
}

fn application_argument() -> Arg {
  Arg::new(APPLICATION_ARGUMENT)
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("APPLICATION")
    .help("Application name")
    .long_help("Application name.")
}
