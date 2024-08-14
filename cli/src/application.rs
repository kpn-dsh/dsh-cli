use clap::{Arg, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;

use crate::arguments::{actual_flag, status_flag, ACTUAL_FLAG, STATUS_FLAG};
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::application::{application_to_default_vector, default_application_column_labels, default_application_table};
use crate::subcommands::{
  list_subcommand, show_subcommand, status_subcommand, target_argument, CommandDescriptor, LIST_SUBCOMMAND, SHOW_SUBCOMMAND, STATUS_SUBCOMMAND, TARGET_ARGUMENT,
};
use crate::tabular::{make_tabular, make_tabular_with_headers, print_tabular};
use crate::{to_command_error, to_command_error_missing_id, to_command_error_with_id, CommandResult};

pub(crate) const APPLICATION_COMMAND: &str = "application";
pub(crate) const APPLICATIONS_COMMAND: &str = "applications";

const WHAT: &str = "application";
const UPPER_WHAT: &str = "APPLICATION";

const TASKS_SUBCOMMAND: &str = "tasks";

pub(crate) fn application_command() -> Command {
  let command_descriptor = CommandDescriptor::new(WHAT, UPPER_WHAT);
  Command::new(APPLICATION_COMMAND)
    .alias("a")
    .about("Show application details")
    .long_about("Show application details")
    .arg_required_else_help(true)
    .subcommands(vec![
      list_subcommand(&command_descriptor, vec![actual_flag(), status_flag()]),
      show_subcommand(&command_descriptor, vec![]),
      status_subcommand(&command_descriptor, vec![]),
      application_tasks_subcommand(&command_descriptor, vec![]),
    ])
}

pub(crate) fn applications_command() -> Command {
  Command::new(APPLICATIONS_COMMAND)
    .about("List application details")
    .alias("as")
    .long_about("List application details")
    .hide(true)
    .args(vec![actual_flag(), status_flag()])
}

pub(crate) async fn run_application_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.subcommand() {
    Some((LIST_SUBCOMMAND, sub_matches)) => run_application_list_subcommand(sub_matches, dsh_api_client).await,
    Some((SHOW_SUBCOMMAND, sub_matches)) => run_application_show_subcommand(sub_matches, dsh_api_client).await,
    Some((STATUS_SUBCOMMAND, sub_matches)) => run_application_status_subcommand(sub_matches, dsh_api_client).await,
    Some((TASKS_SUBCOMMAND, sub_matches)) => run_application_tasks_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

fn application_tasks_subcommand(command_descriptor: &CommandDescriptor, _arguments: Vec<Arg>) -> Command {
  Command::new(TASKS_SUBCOMMAND)
    .about("Show application tasks")
    .after_help("Show application tasks")
    .after_long_help("Show application tasks.")
    .args(vec![target_argument(command_descriptor)])
}

pub(crate) async fn run_applications_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  run_application_list_subcommand(matches, dsh_api_client).await
}

async fn run_application_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_application(application_id).await {
      Ok(application) => {
        let table = default_application_table(application_id, &application);
        let tabular = make_tabular(table, "", "  ", "");
        print_tabular("", &tabular);
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, application_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

async fn run_application_list_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  if matches.get_flag(STATUS_FLAG) {
    run_application_list_subcommand_status(matches, dsh_api_client).await
  } else {
    run_application_list_subcommand_normal(matches, dsh_api_client).await
  }
}

async fn run_application_list_subcommand_normal(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  let applications = if matches.get_flag(ACTUAL_FLAG) { dsh_api_client.get_applications_actual().await } else { dsh_api_client.get_applications().await };
  match applications {
    Ok(applications) => {
      let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      application_ids.sort();
      let mut table: Vec<Vec<String>> = vec![];
      for application_id in application_ids {
        let application = applications.get(&application_id).unwrap();
        table.push(application_to_default_vector(application_id.as_str(), application));
      }
      for line in make_tabular_with_headers(&default_application_column_labels(), table) {
        println!("{}", line)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_application_list_subcommand_status(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  let applications = if matches.get_flag(ACTUAL_FLAG) { dsh_api_client.get_applications_actual().await } else { dsh_api_client.get_applications().await };
  match applications {
    Ok(applications) => {
      let mut application_ids: Vec<String> = applications.keys().map(|k| k.to_string()).collect();
      application_ids.sort();
      let allocation_statusses = futures::future::join_all(application_ids.iter().map(|id| dsh_api_client.get_application_allocation_status(id.as_str()))).await;
      let mut table = vec![];
      for (id, allocation_status) in application_ids.iter().zip(allocation_statusses) {
        let status = allocation_status.unwrap(); // TODO
        table.push(allocation_status_to_table_row(id, &status));
      }
      for line in make_tabular_with_headers(&allocation_status_table_column_labels(WHAT), table) {
        println!("{}", line)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_application_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_application_allocation_status(application_id).await {
      Ok(application_status) => {
        println!("{}", serde_json::to_string_pretty(&application_status).unwrap());
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, application_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

async fn run_application_tasks_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(application_id) => match dsh_api_client.get_application_task_ids(application_id).await {
      Ok(application_tasks) => {
        for task_id in application_tasks {
          println!("{}", task_id)
        }
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, application_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}
