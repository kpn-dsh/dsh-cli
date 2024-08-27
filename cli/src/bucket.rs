use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::bucket::{bucket_status_table_column_labels, bucket_status_to_table_row, bucket_table_column_labels, bucket_to_table_row};
use crate::subject::Subject;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

pub(crate) struct BucketSubject {}

const SUBJECT_TARGET: &str = "bucket";

lazy_static! {
  pub static ref BUCKET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(BucketSubject {});
}

#[async_trait]
impl Subject for BucketSubject {
  fn subject(&self) -> &'static str {
    SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Bucket"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH buckets.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list buckets deployed on the DSH.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("b")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> {
    let mut capabilities: HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> = HashMap::new();
    capabilities.insert(CapabilityType::List, &BUCKET_LIST_CAPABILITY);
    capabilities.insert(CapabilityType::Show, &BUCKET_SHOW_CAPABILITY);
    capabilities
  }
}

lazy_static! {
  pub static ref BUCKET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List buckets".to_string(),
    command_long_about: Some("Lists all available buckets.".to_string()),
    command_after_help: Some("".to_string()),
    command_after_long_help: Some("".to_string()),
    command_executors: vec![
      (FlagType::All, Box::new(&ListAll {}), None),
      (FlagType::AllocationStatus, Box::new(&ListAllocationStatus {}), None),
      (FlagType::Configuration, Box::new(&ListConfiguration {}), None),
      (FlagType::Ids, Box::new(&ListIds {}), None),
    ],
    default_command_executor: Some(Box::new(&ListIds {})),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref BUCKET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show bucket configuration".to_string(),
    command_long_about: Some("".to_string()),
    command_after_help: Some("".to_string()),
    command_after_long_help: Some("".to_string()),
    command_executors: vec![(FlagType::All, Box::new(&ShowAll {}), None), (FlagType::AllocationStatus, Box::new(&ShowAllocationStatus {}), None),],
    default_command_executor: Some(Box::new(&ShowAll {})),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct ListAll {}

#[async_trait]
impl CommandExecutor for ListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let bucket_statuses = futures::future::join_all(bucket_ids.iter().map(|id| dsh_api_client.get_bucket(id.as_str()))).await;
    let mut table = vec![];
    for (bucket_id, bucket_status) in bucket_ids.iter().zip(bucket_statuses) {
      table.push(bucket_status_to_table_row(bucket_id, bucket_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&bucket_status_table_column_labels(SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct ListAllocationStatus {}

#[async_trait]
impl CommandExecutor for ListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let allocation_statuses = futures::future::join_all(bucket_ids.iter().map(|bucket_id| dsh_api_client.get_bucket_allocation_status(bucket_id))).await;
    let mut table: Vec<Vec<String>> = vec![];
    for (bucket_id, allocation_status) in bucket_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(bucket_id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct ListConfiguration {}

#[async_trait]
impl CommandExecutor for ListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let buckets = futures::future::join_all(bucket_ids.iter().map(|id| dsh_api_client.get_bucket_configuration(id.as_str()))).await;
    let mut table = vec![];
    for (bucket_id, bucket) in bucket_ids.iter().zip(buckets) {
      table.push(bucket_to_table_row(bucket_id, bucket.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&bucket_table_column_labels(SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct ListIds {}

#[async_trait]
impl CommandExecutor for ListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    for bucket_id in bucket_ids {
      println!("{}", bucket_id)
    }
    Ok(())
  }
}

struct ShowAll {}

#[async_trait]
impl CommandExecutor for ShowAll {
  async fn execute(&self, argument: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let bucket = dsh_api_client.get_bucket(argument.unwrap().as_str()).await?;
    println!("{:?}", bucket);
    Ok(())
  }
}

struct ShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ShowAllocationStatus {
  async fn execute(&self, argument: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let allocation_status = dsh_api_client.get_bucket_allocation_status(argument.unwrap().as_str()).await?;
    println!("{}", serde_json::to_string_pretty(&allocation_status).unwrap());
    Ok(())
  }
}
