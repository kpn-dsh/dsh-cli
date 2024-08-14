use clap::{ArgMatches, Command};

use trifonius_dsh_api::types::AppCatalogAppResourcesValue;
use trifonius_dsh_api::DshApiClient;

use crate::arguments::{actual_flag, status_flag, ACTUAL_FLAG, STATUS_FLAG};
use crate::formatters::app::get_application_from_app;
use crate::formatters::application::{application_to_default_vector, default_application_column_labels, default_application_table};
use crate::subcommands::{list_subcommand, show_subcommand, status_subcommand, CommandDescriptor, LIST_SUBCOMMAND, SHOW_SUBCOMMAND, STATUS_SUBCOMMAND, TARGET_ARGUMENT};
use crate::tabular::{make_tabular_with_headers, print_table};
use crate::{to_command_error, to_command_error_missing_id, to_command_error_with_id, CommandResult};

pub(crate) const APP_COMMAND: &str = "app";
pub(crate) const APPS_COMMAND: &str = "apps";

const WHAT: &str = "app";
const UPPER_WHAT: &str = "APP";

pub(crate) fn app_command() -> Command {
  let command_descriptor = CommandDescriptor::new(WHAT, UPPER_WHAT);
  Command::new(APP_COMMAND)
    .about("Show app details")
    .long_about("Show app details.")
    .arg_required_else_help(true)
    .subcommands(vec![
      list_subcommand(&command_descriptor, vec![actual_flag(), status_flag()]),
      show_subcommand(&command_descriptor, vec![]),
      status_subcommand(&command_descriptor, vec![]),
    ])
}

pub(crate) fn apps_command() -> Command {
  Command::new(APPS_COMMAND)
    .about("List app details")
    .long_about("List app details")
    .hide(true)
    .args(vec![actual_flag(), status_flag()])
}

pub(crate) async fn run_app_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.subcommand() {
    Some((LIST_SUBCOMMAND, sub_matches)) => run_app_list_subcommand(sub_matches, dsh_api_client).await,
    Some((SHOW_SUBCOMMAND, sub_matches)) => run_app_show_subcommand(sub_matches, dsh_api_client).await,
    Some((STATUS_SUBCOMMAND, sub_matches)) => run_app_status_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

pub(crate) async fn run_apps_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  run_app_list_subcommand(matches, dsh_api_client).await
}

async fn run_app_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(app_id) => match dsh_api_client.get_app(app_id).await {
      Ok(app) => {
        println!("name:                 {}", app.name);
        println!("manifest urn:         {}", app.manifest_urn);
        println!("configuration:        {}", app.configuration.clone().unwrap_or("none".to_string()));
        for (resource_name, resource) in &app.resources {
          match resource {
            AppCatalogAppResourcesValue::Application(application) => {
              println!("resource/application: {}", resource_name);
              print_table(default_application_table(app.name.as_str(), application), "  ", "  ", "");
            }
            AppCatalogAppResourcesValue::Bucket(bucket) => {
              println!("resource/bucket:      {}", resource_name);
              println!("  {:?}", bucket)
            }
            AppCatalogAppResourcesValue::Certificate(certificate) => {
              println!("resource/certificate: {}", resource_name);
              println!("  {:?}", certificate)
            }
            AppCatalogAppResourcesValue::Secret(secret) => {
              println!("resource/secret:      {}", resource_name);
              println!("  {:?}", secret)
            }
            AppCatalogAppResourcesValue::Topic(topic) => {
              println!("resource/topic:       {}", resource_name);
              println!("  {:?}", topic)
            }
            AppCatalogAppResourcesValue::Vhost(vhost) => {
              println!("resource/vhost:       {}", resource_name);
              println!("  {:?}", vhost)
            }
            AppCatalogAppResourcesValue::Volume(volume) => {
              println!("resource/volume:      {}", resource_name);
              println!("  {:?}", volume)
            }
          }
        }
        Ok(())
      }
      Err(dsh_api_error) => to_command_error_with_id(dsh_api_error, WHAT, app_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

async fn run_app_list_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  if matches.get_flag(STATUS_FLAG) {
    run_app_list_subcommand_status(matches, dsh_api_client).await
  } else {
    run_app_list_subcommand_normal(matches, dsh_api_client).await
  }
}

async fn run_app_list_subcommand_normal(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  let apps = if matches.get_flag(ACTUAL_FLAG) { dsh_api_client.get_apps_actual().await } else { dsh_api_client.get_apps().await };
  match apps {
    Ok(apps) => {
      let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      app_ids.sort();
      let mut table: Vec<Vec<String>> = vec![];
      for app_id in app_ids {
        let app = apps.get(&app_id).unwrap();
        let (_, application) = get_application_from_app(app).unwrap();
        table.push(application_to_default_vector(app_id.as_str(), application));
      }
      for line in make_tabular_with_headers(&default_application_column_labels(), table) {
        println!("{}", line)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_app_list_subcommand_status(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  let apps = if matches.get_flag(ACTUAL_FLAG) { dsh_api_client.get_apps_actual().await } else { dsh_api_client.get_apps().await };
  match apps {
    Ok(apps) => {
      let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      app_ids.sort();
      let mut table: Vec<Vec<String>> = vec![];
      for app_id in app_ids {
        let app = apps.get(&app_id).unwrap();
        let application = app
          .resources
          .iter()
          .find_map(|(_key, resource)| match resource {
            AppCatalogAppResourcesValue::Application(application) => Some(application),
            _ => None,
          })
          .unwrap();
        table.push(application_to_default_vector(app_id.as_str(), application));
      }
      for line in make_tabular_with_headers(&[WHAT, "manifest urn"], table) {
        println!("{}", line)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_app_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(app_id) => match dsh_api_client.get_app_catalog_app_status(app_id).await {
      Ok(app_status) => {
        println!("{}", serde_json::to_string_pretty(&app_status).unwrap());
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, app_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}
