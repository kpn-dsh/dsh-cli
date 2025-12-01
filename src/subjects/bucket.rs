use crate::arguments::bucket_id_argument;
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{notifications_to_string, OutputFormat};
use crate::formatters::{Label, SubjectFormatter};
use crate::subject::{Requirements, Subject};
use crate::subjects::DEPENDANT_LABELS;
use crate::{error, DshCliResult, COMMAND_OPTIONS_HEADING};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};
use dsh_api::bucket::BucketInjection;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{Bucket, BucketStatus};
use dsh_api::Dependant;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use tokio::try_join;

struct BucketSubject {}

const BUCKET_SUBJECT_TARGET: &str = "bucket";

lazy_static! {
  pub(crate) static ref BUCKET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(BucketSubject {});
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
      .add_filter_flag(FilterFlagType::Complete, Some("Show all bucket parameters instead of a selection.".to_string()))
      .add_command_executor(FlagType::Ids, &BucketListIds {}, None)
  );
  static ref BUCKET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &BucketShowAll {}, "Show bucket configuration")
      .add_command_executor(FlagType::Usage, &BucketShowUsage {}, None)
      .add_target_argument(bucket_id_argument().required(true))
  );
  static ref BUCKET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![BUCKET_CREATE_CAPABILITY.as_ref(), BUCKET_DELETE_CAPABILITY.as_ref(), BUCKET_LIST_CAPABILITY.as_ref(), BUCKET_SHOW_CAPABILITY.as_ref()];
}

const VERSIONED_FLAG: &str = "versioned";

fn versioned_flag(heading: &'static str) -> Arg {
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
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    let versioned = matches.get_flag(VERSIONED_FLAG);
    if client.get_bucket_configuration(&bucket_id).await.is_ok() {
      return Err(error!("bucket '{}' already exists", bucket_id));
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
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    if client.get_bucket_configuration(&bucket_id).await.is_err() {
      return Err(error!("bucket '{}' does not exists", bucket_id));
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let complete = matches.get_flag(FilterFlagType::Complete.id());
    context.print_explanation("list all buckets with their parameters");
    let start_instant = context.now();
    let (buckets_with_dependants, (access_key_id, _)) = try_join!(client.buckets_with_dependants(), client.bucket_secrets())?;
    context.print_execution_time(start_instant);
    let buckets: Vec<(String, BucketStatus, String, Vec<Dependant<BucketInjection>>)> = buckets_with_dependants
      .into_iter()
      .map(|(bucket_id, bucket_status, dependants)| {
        let bucket_name = client
          .platform()
          .bucket_name(client.tenant().name(), bucket_id.clone(), Some(&access_key_id))
          .unwrap_or_default();
        (bucket_id, bucket_status, bucket_name, dependants)
      })
      .collect_vec();
    let mut formatter = if complete { ListFormatter::new(&BUCKET_STATUS_LABELS_ALL, None, context) } else { ListFormatter::new(&BUCKET_STATUS_LABELS, None, context) };
    formatter.push_values(&buckets);
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all bucket ids");
    let start_instant = context.now();
    let bucket_ids = client.get_bucket_ids().await?;
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
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for bucket '{}'", bucket_id));
    let start_instant = context.now();
    let (bucket, (access_key_id, _)) = try_join!(client.get_bucket(&bucket_id), client.bucket_secrets())?;
    context.print_execution_time(start_instant);
    let bucket_name = client
      .platform()
      .bucket_name(client.tenant().name(), &bucket_id, Some(access_key_id))
      .unwrap_or_default();
    UnitFormatter::new(bucket_id, &BUCKET_STATUS_LABELS, None, context).print(&(bucket, bucket_name), None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct BucketShowUsage {}

#[async_trait]
impl CommandExecutor for BucketShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let bucket_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show services that use bucket '{}'", bucket_id));
    let start_instant = context.now();
    let dependant_applications = client.applications_dependant_on_bucket(&bucket_id).await?;
    context.print_execution_time(start_instant);
    if dependant_applications.is_empty() {
      context.print_outcome("bucket not used")
    } else {
      let mut formatter = ListFormatter::new(&DEPENDANT_LABELS, Some("bucket id"), context);
      formatter.push_values(&dependant_applications);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum BucketLabel {
  Dependants,
  DerivedFrom,
  Encrypted,
  Name,
  Notifications,
  Provisioned,
  Target,
  Versioned,
}

impl Label for BucketLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Dependants => "dependants",
      Self::DerivedFrom => "derived from",
      Self::Encrypted => "encrypted",
      Self::Name => "name",
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

impl SubjectFormatter<BucketLabel> for (BucketStatus, String) {
  fn value(&self, label: &BucketLabel, target_id: &str) -> String {
    let (bucket_status, bucket_name) = self;
    match label {
      BucketLabel::Dependants => "".to_string(),
      BucketLabel::DerivedFrom => bucket_status.status.derived_from.clone().unwrap_or_default(),
      BucketLabel::Encrypted => bucket_status.configuration.as_ref().map(|bs| bs.encrypted.to_string()).unwrap_or_default(),
      BucketLabel::Name => bucket_name.to_string(),
      BucketLabel::Notifications => {
        if self.0.status.notifications.is_empty() {
          "none".to_string()
        } else {
          notifications_to_string(&bucket_status.status.notifications)
        }
      }
      BucketLabel::Provisioned => bucket_status.status.provisioned.to_string(),
      BucketLabel::Target => target_id.to_string(),
      BucketLabel::Versioned => bucket_status.configuration.as_ref().map(|bucket| bucket.versioned.to_string()).unwrap_or_default(),
    }
  }
}

impl SubjectFormatter<BucketLabel> for (String, BucketStatus, String, Vec<Dependant<BucketInjection>>) {
  fn value(&self, label: &BucketLabel, target_id: &str) -> String {
    let (bucket_id, bucket_status, bucket_name, dependants) = self;
    match label {
      BucketLabel::Dependants => dependants.iter().map(|dependant| dependant.to_string()).join("\n"),
      BucketLabel::Target => bucket_id.to_string(),
      _ => (bucket_status.clone(), bucket_name.clone()).value(label, target_id),
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

static BUCKET_STATUS_LABELS: [BucketLabel; 5] = [BucketLabel::Target, BucketLabel::Versioned, BucketLabel::Provisioned, BucketLabel::Name, BucketLabel::Dependants];

static BUCKET_STATUS_LABELS_ALL: [BucketLabel; 8] = [
  BucketLabel::Target,
  BucketLabel::Encrypted,
  BucketLabel::Versioned,
  BucketLabel::Provisioned,
  BucketLabel::Notifications,
  BucketLabel::Name,
  BucketLabel::Dependants,
  BucketLabel::DerivedFrom,
];

pub(crate) static BUCKET_LABELS: [BucketLabel; 3] = [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned];
