use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::{notifications_to_string, OutputFormat};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{Bucket, BucketStatus};
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::arguments::bucket_id_argument;
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::{DshCliResult, COMMAND_OPTIONS_HEADING};

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

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(BUCKET_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(BUCKET_DELETE_CAPABILITY.as_ref()),
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
  static ref BUCKET_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &BucketCreate {}, "Create new bucket")
      .add_target_argument(bucket_id_argument().required(true))
      .add_extra_arguments(vec![versioned_flag(COMMAND_OPTIONS_HEADING)])
  );
  static ref BUCKET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &BucketDelete {}, "Delete bucket")
      .set_long_about("Delete a bucket.")
      .add_target_argument(bucket_id_argument().required(true))
  );
  static ref BUCKET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &BucketListAll {}, "List buckets")
      .set_long_about("Lists all available buckets.")
      .add_command_executor(FlagType::Ids, &BucketListIds {}, None)
  );
  static ref BUCKET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &BucketShowAll {}, "Show bucket configuration").add_target_argument(bucket_id_argument().required(true))
  );
  static ref BUCKET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![BUCKET_CREATE_CAPABILITY.as_ref(), BUCKET_DELETE_CAPABILITY.as_ref(), BUCKET_LIST_CAPABILITY.as_ref(), BUCKET_SHOW_CAPABILITY.as_ref()];
}

// Encrypted is mandatory
// pub(crate) const ENCRYPTED_FLAG: &str = "encrypted";
// pub(crate) fn encrypted_flag(heading: &'static str) -> Arg {
//   Arg::new(ENCRYPTED_FLAG)
//     .long("encrypted")
//     .action(ArgAction::SetTrue)
//     .help("Encrypted bucket")
//     .long_help("Create an encrypted bucket.")
//     .help_heading(heading)
// }

pub(crate) const VERSIONED_FLAG: &str = "versioned";

pub(crate) fn versioned_flag(heading: &'static str) -> Arg {
  Arg::new(VERSIONED_FLAG)
    .long("versioned")
    .action(ArgAction::SetTrue)
    .help("Versioned bucket")
    .long_help("Create a versioned bucket.")
    .help_heading(heading)
}

struct BucketCreate {}

#[async_trait]
impl CommandExecutor for BucketCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    let versioned = matches.get_flag(VERSIONED_FLAG);
    if client.get_bucket_configuration(&bucket_id).await.is_ok() {
      return Err(format!("bucket '{}' already exists", bucket_id));
    }
    context.print_explanation(format!("create new bucket '{}'", bucket_id));
    if context.dry_run() {
      context.print_warning("dry-run mode, bucket not created");
    } else {
      let bucket = Bucket { encrypted: true, versioned };
      client.put_bucket_configuration(&bucket_id, &bucket).await?;
      context.print_outcome(format!("bucket '{}' created", bucket_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct BucketDelete {}

#[async_trait]
impl CommandExecutor for BucketDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    if client.get_bucket_configuration(&bucket_id).await.is_err() {
      return Err(format!("bucket '{}' does not exists", bucket_id));
    }
    if context.confirmed(format!("delete bucket '{}'?", bucket_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, bucket not deleted");
      } else {
        client.delete_bucket_configuration(&bucket_id).await?;
        context.print_outcome(format!("bucket '{}' deleted", bucket_id));
      }
    } else {
      context.print_outcome(format!("cancelled, bucket '{}' not deleted", bucket_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct BucketListAll {}

#[async_trait]
impl CommandExecutor for BucketListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all buckets with their parameters");
    let start_instant = context.now();
    let bucket_ids = client.list_bucket_ids().await?;
    let bucket_statuses = try_join_all(bucket_ids.iter().map(|bucket_id| client.get_bucket(bucket_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&BUCKET_STATUS_LABELS, None, context);
    formatter.push_target_ids_and_values(bucket_ids.as_slice(), bucket_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct BucketListIds {}

#[async_trait]
impl CommandExecutor for BucketListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all bucket ids");
    let start_instant = context.now();
    let bucket_ids = client.list_bucket_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("bucket id", context);
    formatter.push_target_ids(&bucket_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct BucketShowAll {}

#[async_trait]
impl CommandExecutor for BucketShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for bucket '{}'", bucket_id));
    let start_instant = context.now();
    let bucket = client.get_bucket(&bucket_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(bucket_id, &BUCKET_STATUS_LABELS, None, context).print(&bucket, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
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
}

pub static BUCKET_STATUS_LABELS: [BucketLabel; 6] =
  [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned, BucketLabel::Provisioned, BucketLabel::Notifications, BucketLabel::DerivedFrom];

pub static BUCKET_LABELS: [BucketLabel; 3] = [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned];
