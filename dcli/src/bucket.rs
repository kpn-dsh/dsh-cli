use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use lazy_static::lazy_static;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::flags::FlagType;
use crate::formatters::bucket::BUCKET_STATUS_LABELS;
use crate::formatters::formatter::{print_vec, TableBuilder};
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

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH buckets.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list buckets deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("b")
  }

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, BUCKET_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, BUCKET_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref BUCKET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List buckets")
      .set_long_about("Lists all available buckets.")
      .set_default_command_executor(&BucketListAll {})
      .add_command_executor(FlagType::Ids, &BucketListIds {}, None)
      .set_run_all_executors(true)
  );
  pub static ref BUCKET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show bucket configuration")
      .set_default_command_executor(&BucketShowAll {})
      .add_target_argument(target_argument(BUCKET_SUBJECT_TARGET, None))
  );
}

struct BucketListAll {}

#[async_trait]
impl CommandExecutor for BucketListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all buckets with their parameters");
    }
    let bucket_ids = context.dsh_api_client.as_ref().unwrap().list_bucket_ids().await?;
    let bucket_statuses = try_join_all(bucket_ids.iter().map(|id| context.dsh_api_client.as_ref().unwrap().get_bucket(id.as_str()))).await?;
    let mut builder = TableBuilder::list(&BUCKET_STATUS_LABELS, context);
    for (bucket_id, bucket_status) in bucket_ids.iter().zip(bucket_statuses) {
      builder.value(bucket_id.to_string(), &bucket_status);
    }
    builder.print();
    Ok(false)
  }
}

struct BucketListIds {}

#[async_trait]
impl CommandExecutor for BucketListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all bucket ids");
    }
    print_vec("bucket ids".to_string(), context.dsh_api_client.as_ref().unwrap().list_bucket_ids().await?, context);
    Ok(false)
  }
}

struct BucketShowAll {}

#[async_trait]
impl CommandExecutor for BucketShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for bucket '{}'", bucket_id);
    }
    let bucket = context.dsh_api_client.as_ref().unwrap().get_bucket(bucket_id.as_str()).await?;
    let mut builder = TableBuilder::show(&BUCKET_STATUS_LABELS, context);
    builder.value(bucket_id, &bucket);
    builder.print();
    Ok(false)
  }
}
