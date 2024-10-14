use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::print_allocation_statuses;
use crate::formatters::bucket::{BUCKET_LABELS, BUCKET_STATUS_LABELS};
use crate::formatters::formatter::{print_ids, TableBuilder};
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

pub(crate) struct BucketSubject {}

const BUCKET_SUBJECT_TARGET: &str = "bucket";

lazy_static! {
  pub static ref BUCKET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(BucketSubject {});
}

#[async_trait]
impl Subject for BucketSubject {
  fn subject(&self) -> &'static str {
    BUCKET_SUBJECT_TARGET
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

  fn subject_command_alias(&self) -> Option<&str> {
    Some("b")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, BUCKET_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, BUCKET_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref BUCKET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List buckets".to_string(),
    command_long_about: Some("Lists all available buckets.".to_string()),
    command_executors: vec![
      (FlagType::All, &BucketListAll {}, None),
      (FlagType::AllocationStatus, &BucketListAllocationStatus {}, None),
      (FlagType::Configuration, &BucketListConfiguration {}, None),
      (FlagType::Ids, &BucketListIds {}, None),
    ],
    default_command_executor: Some(&BucketListAll {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref BUCKET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show bucket configuration".to_string(),
    command_long_about: None,
    command_executors: vec![(FlagType::All, &BucketShowAll {}, None), (FlagType::AllocationStatus, &BucketShowAllocationStatus {}, None),],
    default_command_executor: Some(&BucketShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct BucketListAll {}

#[async_trait]
impl CommandExecutor for BucketListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all buckets with their parameters");
    }
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let bucket_statuses = try_join_all(bucket_ids.iter().map(|id| dsh_api_client.get_bucket(id.as_str()))).await?;
    let mut builder = TableBuilder::list(&BUCKET_STATUS_LABELS, context);
    for (bucket_id, bucket_status) in bucket_ids.iter().zip(bucket_statuses) {
      builder.value(bucket_id.to_string(), &bucket_status);
    }
    builder.print();
    Ok(false)
  }
}

struct BucketListAllocationStatus {}

#[async_trait]
impl CommandExecutor for BucketListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all buckets with their allocation status");
    }
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let allocation_statuses = try_join_all(bucket_ids.iter().map(|bucket_id| dsh_api_client.get_bucket_allocation_status(bucket_id))).await?;
    print_allocation_statuses(bucket_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct BucketListConfiguration {}

#[async_trait]
impl CommandExecutor for BucketListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all buckets with their configuration");
    }
    let bucket_ids = dsh_api_client.get_bucket_ids().await?;
    let buckets = try_join_all(bucket_ids.iter().map(|bucket_id| dsh_api_client.get_bucket_configuration(bucket_id.as_str()))).await?;
    let mut builder = TableBuilder::list(&BUCKET_LABELS, context);
    for (bucket_id, bucket) in bucket_ids.iter().zip(buckets) {
      builder.value(bucket_id.to_string(), &bucket);
    }
    builder.print();
    Ok(false)
  }
}

struct BucketListIds {}

#[async_trait]
impl CommandExecutor for BucketListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all bucket ids");
    }
    print_ids("bucket ids".to_string(), dsh_api_client.get_bucket_ids().await?, context);
    Ok(false)
  }
}

struct BucketShowAll {}

#[async_trait]
impl CommandExecutor for BucketShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for bucket '{}'", bucket_id);
    }
    let bucket = dsh_api_client.get_bucket(bucket_id.as_str()).await?;
    println!("{:?}", bucket);
    Ok(false)
  }
}

struct BucketShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for BucketShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the allocation status for bucket '{}'", bucket_id);
    }
    let allocation_status = dsh_api_client.get_bucket_allocation_status(bucket_id.as_str()).await?;
    println!("{}", serde_json::to_string_pretty(&allocation_status).unwrap());
    Ok(false)
  }
}
