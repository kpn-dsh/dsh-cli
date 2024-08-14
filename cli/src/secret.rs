use std::collections::HashMap;

use clap::{ArgMatches, Command};

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::arguments::{actual_flag, status_flag, usage_flag, STATUS_FLAG};
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::subcommands::{
  list_subcommand, show_subcommand, status_subcommand, usage_subcommand, CommandDescriptor, LIST_SUBCOMMAND, SHOW_SUBCOMMAND, STATUS_SUBCOMMAND, TARGET_ARGUMENT, USAGE_SUBCOMMAND,
};
use crate::tabular::make_tabular_with_headers;
use crate::{to_command_error, to_command_error_missing_id, to_command_error_with_id, CommandResult};

pub(crate) const SECRET_COMMAND: &str = "secret";
pub(crate) const SECRETS_COMMAND: &str = "secrets";

const WHAT: &str = "secret";
const UPPER_WHAT: &str = "SECRET";

pub(crate) fn secret_command() -> Command {
  let command_descriptor = CommandDescriptor::new(WHAT, UPPER_WHAT);
  Command::new(SECRET_COMMAND)
    .about("Show secret details")
    .alias("s")
    .long_about("Show secret details")
    .arg_required_else_help(true)
    .subcommands(vec![
      list_subcommand(&command_descriptor, vec![actual_flag(), status_flag(), usage_flag(WHAT)]),
      show_subcommand(&command_descriptor, vec![]),
      status_subcommand(&command_descriptor, vec![]),
      usage_subcommand(&command_descriptor, vec![]),
    ])
}

pub(crate) fn secrets_command() -> Command {
  Command::new(SECRETS_COMMAND)
    .about("List secrets")
    .alias("ss")
    .long_about("List secrets")
    .hide(true)
    .args(vec![actual_flag(), status_flag()])
}

pub(crate) async fn run_secret_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.subcommand() {
    Some((LIST_SUBCOMMAND, sub_matches)) => run_secret_list_subcommand(sub_matches, dsh_api_client).await,
    Some((SHOW_SUBCOMMAND, sub_matches)) => run_secret_show_subcommand(sub_matches, dsh_api_client).await,
    Some((STATUS_SUBCOMMAND, sub_matches)) => run_secret_status_subcommand(sub_matches, dsh_api_client).await,
    Some((USAGE_SUBCOMMAND, sub_matches)) => run_secret_usage_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

pub(crate) async fn run_secrets_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  run_secret_list_subcommand(matches, dsh_api_client).await
}

async fn run_secret_list_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  if matches.get_flag(STATUS_FLAG) {
    run_secret_list_subcommand_status(matches, dsh_api_client).await
  } else {
    run_secret_list_subcommand_normal(matches, dsh_api_client).await
  }
}

async fn run_secret_list_subcommand_normal(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match dsh_api_client.get_secret_ids().await {
    Ok(mut secrets) => {
      secrets.sort();
      for secret in secrets {
        println!("{}", secret)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_secret_list_subcommand_status(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match dsh_api_client.get_secret_ids().await {
    Ok(mut secret_ids) => {
      secret_ids.sort();
      let allocation_statusses = futures::future::join_all(secret_ids.iter().map(|id| dsh_api_client.get_secret_allocation_status(id.as_str()))).await;
      let mut table = vec![];
      for (id, secret_status) in secret_ids.iter().zip(allocation_statusses) {
        let status = secret_status.unwrap(); // TODO
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

async fn run_secret_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(secret_id) => match dsh_api_client.get_secret(secret_id).await {
      Ok(secret) => {
        println!("{}", secret);
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, secret_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

async fn run_secret_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(secret_id) => match dsh_api_client.get_secret_allocation_status(secret_id).await {
      Ok(allocation_status) => {
        println!("{:?}", allocation_status);
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, secret_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

// TODO Secrets are also used for buckets, databases and apps
async fn run_secret_usage_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(secret_id) => match dsh_api_client.get_applications().await {
      Ok(applications) => {
        let usage = applications_that_use_secret(secret_id, &applications);
        if !usage.is_empty() {
          let table: Vec<Vec<String>> = usage.iter().map(|(application_id, usage)| vec![application_id.clone(), usage.clone()]).collect();
          for line in make_tabular_with_headers(&["application", "usage"], table) {
            println!("{}", line)
          }
        } else {
          println!("secret not used")
        }
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, secret_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

fn applications_that_use_secret(secret_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, String)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, String)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.secrets.is_empty() {
      for application_secret in &application.secrets {
        if application_secret.name == secret_id {
          let mut envs: Vec<String> = application_secret
            .clone()
            .injections
            .into_iter()
            .filter_map(|injection| injection.get("env").cloned())
            .collect();
          if envs.len() == 1 {
            pairs.push((application_id.clone(), format!("env:{}", envs.first().unwrap())));
          }
          if envs.len() > 1 {
            envs.sort();
            let joined_envs: String = envs.join(",");
            pairs.push((application_id.clone(), format!("envs:{}", joined_envs)));
          }
        }
      }
    }
  }
  pairs
}
