use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::notifications_to_string;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::{Bucket, BucketStatus};
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

use crate::arguments::target_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR, SHOW_COMMAND, SHOW_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;

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

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(true, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(BUCKET_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(BUCKET_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &BUCKET_CAPABILITIES
  }
}

lazy_static! {
  static ref BUCKET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List buckets")
      .set_long_about("Lists all available buckets.")
      .set_default_command_executor(&BucketListAll {})
      .add_command_executor(FlagType::Ids, &BucketListIds {}, None)
      .set_run_all_executors(true)
  );
  static ref BUCKET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND_PAIR, "Show bucket configuration")
      .set_default_command_executor(&BucketShowAll {})
      .add_target_argument(target_argument(BUCKET_SUBJECT_TARGET, None))
  );
  static ref BUCKET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![BUCKET_LIST_CAPABILITY.as_ref(), BUCKET_SHOW_CAPABILITY.as_ref()];
}

struct BucketListAll {}

#[async_trait]
impl CommandExecutor for BucketListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all buckets with their parameters");
    let start_instant = Instant::now();
    let bucket_ids = context.dsh_api_client.as_ref().unwrap().list_bucket_ids().await?;
    let bucket_statuses = try_join_all(bucket_ids.iter().map(|id| context.dsh_api_client.as_ref().unwrap().get_bucket(id.as_str()))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&BUCKET_STATUS_LABELS, None, context);
    formatter.push_target_ids_and_values(bucket_ids.as_slice(), bucket_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct BucketListIds {}

#[async_trait]
impl CommandExecutor for BucketListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all bucket ids");
    let start_instant = Instant::now();
    let bucket_ids = context.dsh_api_client.as_ref().unwrap().list_bucket_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("bucket id", context);
    formatter.push_target_ids(&bucket_ids);
    formatter.print()?;
    Ok(())
  }
}

struct BucketShowAll {}

#[async_trait]
impl CommandExecutor for BucketShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for bucket '{}'", bucket_id));
    let start_instant = Instant::now();
    let bucket = context.dsh_api_client.as_ref().unwrap().get_bucket(bucket_id.as_str()).await?;
    context.print_execution_time(start_instant);
    let formatter = UnitFormatter::new(bucket_id, &BUCKET_STATUS_LABELS, None, &bucket, context);
    formatter.print()?;
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum BucketLabel {
  DerivedFrom,
  Encrypted,
  Notifications,
  Provisioned,
  Target,
  Versioned,
}

impl Label for BucketLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Encrypted => "encrypted",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => "bucket id",
      Self::Versioned => "versioned",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<BucketLabel> for BucketStatus {
  fn value(&self, label: &BucketLabel, target_id: &str) -> String {
    match label {
      BucketLabel::DerivedFrom => self.status.derived_from.clone().unwrap_or_default(),
      BucketLabel::Encrypted => self.configuration.as_ref().map(|bs| bs.encrypted.to_string()).unwrap_or_default(),
      BucketLabel::Notifications => {
        if self.status.notifications.is_empty() {
          "none".to_string()
        } else {
          notifications_to_string(&self.status.notifications)
        }
      }
      BucketLabel::Provisioned => self.status.provisioned.to_string(),
      BucketLabel::Target => target_id.to_string(),
      BucketLabel::Versioned => self.configuration.as_ref().map(|bs| bs.versioned.to_string()).unwrap_or_default(),
    }
  }

  fn target_label(&self) -> Option<BucketLabel> {
    Some(BucketLabel::Target)
  }
}

impl SubjectFormatter<BucketLabel> for Bucket {
  fn value(&self, label: &BucketLabel, target_id: &str) -> String {
    match label {
      BucketLabel::Encrypted => self.encrypted.to_string(),
      BucketLabel::Target => target_id.to_string(),
      BucketLabel::Versioned => self.versioned.to_string(),
      _ => "".to_string(),
    }
  }

  fn target_label(&self) -> Option<BucketLabel> {
    Some(BucketLabel::Target)
  }
}

pub static BUCKET_STATUS_LABELS: [BucketLabel; 6] =
  [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned, BucketLabel::Provisioned, BucketLabel::Notifications, BucketLabel::DerivedFrom];

pub static BUCKET_LABELS: [BucketLabel; 3] = [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned];
