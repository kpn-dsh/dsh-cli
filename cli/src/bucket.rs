use clap::{ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;

use crate::arguments::{actual_flag, status_flag, STATUS_FLAG};
use crate::formatters::bucket::{bucket_status_table_column_labels, bucket_status_to_table_row};
use crate::subcommands::{list_subcommand, show_subcommand, status_subcommand, CommandDescriptor, LIST_SUBCOMMAND, SHOW_SUBCOMMAND, STATUS_SUBCOMMAND, TARGET_ARGUMENT};
use crate::tabular::make_tabular_with_headers;
use crate::{to_command_error, to_command_error_missing_id, to_command_error_with_id, CommandResult};

pub(crate) const BUCKET_COMMAND: &str = "bucket";
pub(crate) const BUCKETS_COMMAND: &str = "buckets";

const WHAT: &str = "bucket";
const UPPER_WHAT: &str = "BUCKET";

pub(crate) fn bucket_command() -> Command {
  let command_descriptor = CommandDescriptor::new(WHAT, UPPER_WHAT);
  Command::new(BUCKET_COMMAND)
    .about("Show bucket details")
    .alias("b")
    .long_about("Show bucket details")
    .arg_required_else_help(true)
    .subcommands(vec![
      list_subcommand(&command_descriptor, vec![actual_flag(), status_flag()]),
      show_subcommand(&command_descriptor, vec![]),
      status_subcommand(&command_descriptor, vec![]),
      // usage_subcommand(&command_descriptor, vec![]),
    ])
}

pub(crate) fn buckets_command() -> Command {
  Command::new(BUCKETS_COMMAND)
    .about("List buckets")
    .alias("bs")
    .long_about("List buckets")
    .hide(true)
    .args(vec![actual_flag(), status_flag()])
}

pub(crate) async fn run_bucket_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.subcommand() {
    Some((LIST_SUBCOMMAND, sub_matches)) => run_bucket_list_subcommand(sub_matches, dsh_api_client).await,
    Some((SHOW_SUBCOMMAND, sub_matches)) => run_bucket_show_subcommand(sub_matches, dsh_api_client).await,
    Some((STATUS_SUBCOMMAND, sub_matches)) => run_bucket_status_subcommand(sub_matches, dsh_api_client).await,
    // Some((USAGE_SUBCOMMAND, sub_matches)) => run_bucket_usage_subcommand(sub_matches, dsh_api_client).await,
    _ => unreachable!(),
  }
}

pub(crate) async fn run_buckets_command(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  run_bucket_list_subcommand(matches, dsh_api_client).await
}

async fn run_bucket_list_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  if matches.get_flag(STATUS_FLAG) {
    run_bucket_list_subcommand_status(matches, dsh_api_client).await
  } else {
    run_bucket_list_subcommand_normal(matches, dsh_api_client).await
  }
}

async fn run_bucket_list_subcommand_normal(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match dsh_api_client.get_bucket_ids().await {
    Ok(mut bucket_ids) => {
      bucket_ids.sort();
      for bucket_id in bucket_ids {
        println!("{}", bucket_id)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_bucket_list_subcommand_status(_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match dsh_api_client.get_bucket_ids().await {
    Ok(mut bucket_ids) => {
      bucket_ids.sort();
      let bucket_statusses = futures::future::join_all(bucket_ids.iter().map(|id| dsh_api_client.get_bucket_status(id.as_str()))).await;
      let mut table = vec![];
      for (id, bucket_status) in bucket_ids.iter().zip(bucket_statusses) {
        let status = bucket_status.unwrap(); // TODO
        table.push(bucket_status_to_table_row(id, &status));
      }
      for line in make_tabular_with_headers(&bucket_status_table_column_labels(WHAT), table) {
        println!("{}", line)
      }
      Ok(())
    }
    Err(error) => to_command_error(error),
  }
}

async fn run_bucket_show_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(bucket_id) => match dsh_api_client.get_bucket_status(bucket_id).await {
      Ok(bucket) => {
        println!("{:?}", bucket);
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, bucket_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

async fn run_bucket_status_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
  match matches.get_one::<String>(TARGET_ARGUMENT) {
    Some(bucket_id) => match dsh_api_client.get_bucket_allocation_status(bucket_id).await {
      Ok(allocation_status) => {
        println!("{}", serde_json::to_string_pretty(&allocation_status).unwrap());
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, WHAT, bucket_id),
    },
    None => to_command_error_missing_id(WHAT),
  }
}

// TODO Is this possible?
// async fn run_bucket_usage_subcommand(matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) {}
