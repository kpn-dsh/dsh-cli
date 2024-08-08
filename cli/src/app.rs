use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;
use trifonius_dsh_api::DshApiError;

use crate::tabular::make_tabular_with_headers;

pub(crate) const APP_COMMAND: &str = "app";
const APP_ARGUMENT: &str = "app-argument";

const APP_LIST_SUBCOMMAND: &str = "list";
const APP_SHOW_SUBCOMMAND: &str = "show";
const APP_STATUS_SUBCOMMAND: &str = "status";

pub(crate) fn app_command() -> Command {
  Command::new(APP_COMMAND)
    .about("Show app details")
    .long_about("Show app details")
    .arg_required_else_help(true)
    .subcommands(vec![app_list_subcommand(), app_show_subcommand(), app_status_subcommand()])
}

pub(crate) async fn run_app_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.subcommand() {
    Some((APP_LIST_SUBCOMMAND, sub_matches)) => run_app_list_subcommand(sub_matches, dsh_api_client).await,
    Some((APP_SHOW_SUBCOMMAND, sub_matches)) => run_app_show_subcommand(sub_matches, dsh_api_client).await,
    Some((APP_STATUS_SUBCOMMAND, sub_matches)) => run_app_status_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

fn app_list_subcommand() -> Command {
  Command::new(APP_LIST_SUBCOMMAND)
    .about("List apps")
    .after_help("List apps")
    .after_long_help("List apps.")
}

fn app_show_subcommand() -> Command {
  Command::new(APP_SHOW_SUBCOMMAND)
    .about("Show app details")
    .after_help("Show app details")
    .after_long_help("Show app details.")
    .args(vec![app_argument()])
}

fn app_status_subcommand() -> Command {
  Command::new(APP_STATUS_SUBCOMMAND)
    .about("Show app status")
    .after_help("Show app status")
    .after_long_help("Show app status.")
    .args(vec![app_argument()])
}

async fn run_app_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.get_one::<String>(APP_ARGUMENT) {
    Some(app_id) => match dsh_api_client.get_app(app_id).await {
      Ok(app) => println!("{}", serde_json::to_string_pretty(&app).unwrap()),
      Err(DshApiError::NotFound) => println!("app id {} does not exist", app_id),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing app id"),
  }
}

async fn run_app_list_subcommand(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match dsh_api_client.get_apps().await {
    Ok(apps) => {
      let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      app_ids.sort();

      let mut table: Vec<Vec<String>> = vec![];
      for app_id in app_ids {
        let app = apps.get(&app_id).unwrap();
        table.push(vec![app_id.clone(), app.manifest_urn.clone()]);
      }
      for line in make_tabular_with_headers(vec!["app", "manifest urn"], table) {
        println!("{}", line)
      }
    }
    Err(error) => println!("unexpected error {}", error),
  }
}

async fn run_app_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {
  match matches.get_one::<String>(APP_ARGUMENT) {
    Some(app_id) => match dsh_api_client.get_app_catalog_app_status(app_id).await {
      Ok(app_status) => println!("{}", serde_json::to_string_pretty(&app_status).unwrap()),
      Err(DshApiError::NotFound) => println!("app id {} does not exist", app_id),
      Err(error) => println!("unexpected error {}", error),
    },
    None => println!("missing app id"),
  }
}

fn app_argument() -> Arg {
  Arg::new(APP_ARGUMENT)
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("APP")
    .help("App name")
    .long_help("App name.")
}
