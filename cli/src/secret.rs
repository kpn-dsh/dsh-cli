use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;
use trifonius_dsh_api::DshApiError;

use crate::tabular::make_tabular_with_headers;

pub(crate) const SECRET_COMMAND: &str = "secret";
const SECRET_ARGUMENT: &str = "secret-argument";

const SECRET_LIST_SUBCOMMAND: &str = "list";
const SECRET_SHOW_SUBCOMMAND: &str = "show";
const SECRET_STATUS_SUBCOMMAND: &str = "status";
const SECRET_USAGE_SUBCOMMAND: &str = "usage";

pub(crate) fn secret_command() -> Command {
  Command::new(SECRET_COMMAND)
    .about("Show secrets details")
    .long_about("Show secrets details")
    .arg_required_else_help(true)
    .subcommands(vec![
      secret_list_subcommand(),
      secret_show_subcommand(),
      secret_status_subcommand(),
      secret_usage_subcommand(),
    ])
}

pub(crate) async fn run_secret_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> () {
  match matches.subcommand() {
    Some((SECRET_LIST_SUBCOMMAND, sub_matches)) => run_secret_list_subcommand(sub_matches, dsh_api_client).await,
    Some((SECRET_SHOW_SUBCOMMAND, sub_matches)) => run_secret_show_subcommand(sub_matches, dsh_api_client).await,
    Some((SECRET_STATUS_SUBCOMMAND, sub_matches)) => run_secret_status_subcommand(sub_matches, dsh_api_client).await,
    Some((SECRET_USAGE_SUBCOMMAND, sub_matches)) => run_secret_usage_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

fn secret_list_subcommand() -> Command {
  Command::new(SECRET_LIST_SUBCOMMAND)
    .about("Show application details")
    .after_help("Show application details")
    .after_long_help("Show application details.")
}

fn secret_show_subcommand() -> Command {
  Command::new(SECRET_SHOW_SUBCOMMAND)
    .about("Show application details")
    .after_help("Show application details")
    .after_long_help("Show application details.")
    .args(vec![secret_argument()])
}

fn secret_status_subcommand() -> Command {
  Command::new(SECRET_STATUS_SUBCOMMAND)
    .about("Show application status")
    .after_help("Show application status")
    .after_long_help("Show application status.")
    .args(vec![secret_argument()])
}

fn secret_usage_subcommand() -> Command {
  Command::new(SECRET_USAGE_SUBCOMMAND)
    .about("Show secret usage")
    .after_help("Show secret usage")
    .after_long_help("Show secret usage.")
    .args(vec![secret_argument()])
}

async fn run_secret_list_subcommand(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> () {
  match dsh_api_client.get_secrets().await {
    Ok(mut secrets) => {
      secrets.sort();
      for secret in secrets {
        println!("{}", secret)
      }
    }
    Err(error) => println!("unexpected error {}", error),
  }
}

async fn run_secret_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> () {
  match matches.get_one::<String>(SECRET_ARGUMENT) {
    Some(secret_id) => match dsh_api_client.get_secret(secret_id).await {
      Ok(_secret_bytestream) => println!("{}", 42),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing application id"),
  }
}

async fn run_secret_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> () {
  match matches.get_one::<String>(SECRET_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_tasks(application_id).await {
      Ok(application_tasks) => println!("{}", serde_json::to_string_pretty(&application_tasks).unwrap()),
      Err(DshApiError::NotFound) => println!("application id {} does not exist", application_id),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing application id"),
  }
}

// TODO Secrets are also used for buckets, databases and apps
async fn run_secret_usage_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> () {
  match matches.get_one::<String>(SECRET_ARGUMENT) {
    Some(secret_argument) => match dsh_api_client.get_applications().await {
      Ok(applications) => {
        let mut table: Vec<Vec<String>> = vec![];
        for (application_id, application) in applications {
          for application_secret in application.secrets {
            if &application_secret.name == secret_argument {
              let envs = application_secret.injections.iter();
              let envs: Vec<Option<String>> = envs.map(|inj| inj.get("env").cloned()).collect();
              let envs = envs.into_iter().flatten().collect::<Vec<String>>().join(", ");
              table.push(vec![application_id.clone(), envs]);
            }
          }
        }
        for line in make_tabular_with_headers(vec!["application", "environment variable"], table) {
          println!("{}", line)
        }
      }
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing application id"),
  }
}

fn secret_argument() -> Arg {
  Arg::new(SECRET_ARGUMENT)
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("SECRET")
    .help("Secret id")
    .long_help("Secret id.")
}
