use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::DshApiClient;

use crate::arguments::Flag;
use crate::command::{SubjectCommand, TARGET_ARGUMENT};
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::bucket::{bucket_status_table_column_labels, bucket_status_to_table_row, bucket_table_column_labels, bucket_to_table_row};
use crate::tabular::make_tabular_with_headers;
use crate::{to_command_error_missing_id, to_command_error_with_id, CommandResult};

pub(crate) struct BucketCommand {}

lazy_static! {
  pub static ref BUCKET_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(BucketCommand {});
}

#[async_trait]
impl SubjectCommand for BucketCommand {
  fn subject(&self) -> &'static str {
    "bucket"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Bucket"
  }

  fn about(&self) -> String {
    "Show bucket details".to_string()
  }

  fn long_about(&self) -> String {
    "Show bucket details".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("b")
  }

  fn list_flags(&self) -> &'static [Flag] {
    &[Flag::All, Flag::AllocationStatus, Flag::Configuration, Flag::Ids, Flag::Tasks, Flag::Usage, Flag::Value]
  }

  fn show_flags(&self) -> &'static [Flag] {
    &[Flag::All, Flag::AllocationStatus]
  }

  async fn list_all(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let bucket_statuses = futures::future::join_all(bucket_ids.iter().map(|id| dsh_api_client.get_bucket(id.as_str()))).await;
    let mut table = vec![];
    for (bucket_id, bucket_status) in bucket_ids.iter().zip(bucket_statuses) {
      table.push(bucket_status_to_table_row(bucket_id, bucket_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&bucket_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_allocation_status(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let allocation_statuses = futures::future::join_all(bucket_ids.iter().map(|bucket_id| dsh_api_client.get_bucket_allocation_status(bucket_id))).await;
    let mut table: Vec<Vec<String>> = vec![];
    for (bucket_id, allocation_status) in bucket_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(bucket_id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_configuration(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let buckets = futures::future::join_all(bucket_ids.iter().map(|id| dsh_api_client.get_bucket_configuration(id.as_str()))).await;
    let mut table = vec![];
    for (bucket_id, bucket) in bucket_ids.iter().zip(buckets) {
      table.push(bucket_to_table_row(bucket_id, bucket.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&bucket_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_ids(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    for bucket_id in bucket_ids {
      println!("{}", bucket_id)
    }
    Ok(())
  }

  async fn show_all(&self, _target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    match matches.get_one::<String>(TARGET_ARGUMENT) {
      Some(bucket_id) => match dsh_api_client.get_bucket(bucket_id).await {
        Ok(bucket) => {
          println!("{:?}", bucket);
          Ok(())
        }
        Err(error) => to_command_error_with_id(error, self, bucket_id),
      },
      None => to_command_error_missing_id(self),
    }
  }

  async fn show_allocation_status(&self, _target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    match matches.get_one::<String>(TARGET_ARGUMENT) {
      Some(bucket_id) => match dsh_api_client.get_bucket_allocation_status(bucket_id).await {
        Ok(allocation_status) => {
          println!("{}", serde_json::to_string_pretty(&allocation_status).unwrap());
          Ok(())
        }
        Err(error) => to_command_error_with_id(error, self, bucket_id),
      },
      None => to_command_error_missing_id(self),
    }
  }
}
