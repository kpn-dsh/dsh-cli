use crate::arguments::volume_id_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{ColumnAlignment, Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS_APPS, DEPENDANT_LABELS_LIST, DEPENDANT_LABELS_SERVICES};
use crate::{cli_error, err, DshCliResult};
use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{Volume, VolumeStatus};
use dsh_api::volume::VolumeInjection;
use dsh_api::{Dependant, DependantApp, DependantApplication};
use futures::future::try_join_all;
use futures::join;
use itertools::{Either, Itertools};
use lazy_static::lazy_static;
use serde::Serialize;

struct VolumeSubject {}

const VOLUME_SUBJECT_TARGET: &str = "volume";

lazy_static! {
  pub(crate) static ref VOLUME_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(VolumeSubject {});
}

#[async_trait]
impl Subject for VolumeSubject {
  fn subject(&self) -> &'static str {
    VOLUME_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH volumes.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list volumes deployed on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(VOLUME_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(VOLUME_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(VOLUME_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(VOLUME_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &VOLUME_CAPABILITIES
  }
}

lazy_static! {
  static ref VOLUME_CREATE_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &VolumeCreate {}, "Create new volume")
      .set_long_about("Create a new volume.")
      .add_target_argument(volume_id_argument().required(true))
      .add_extra_argument(size_option())
  );
  static ref VOLUME_DELETE_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &VolumeDelete {}, "Delete volume")
      .set_long_about("Delete a volume.")
      .add_target_argument(volume_id_argument().required(true))
  );
  static ref VOLUME_LIST_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &VolumeList {}, "List volumes")
      .set_long_about("Lists all available volumes.")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &VolumeListAllocationStatus {}, None),
        (FlagType::Configuration, &VolumeListConfiguration {}, None),
        (FlagType::Ids, &VolumeListIds {}, None),
        (FlagType::Usage, &VolumeListUsage {}, None)
      ])
  );
  static ref VOLUME_SHOW_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &VolumeShow {}, "Show secret configuration")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &VolumeShowAllocationStatus {}, None),
        (FlagType::Usage, &VolumeShowUsage {}, None),
      ])
      .add_target_argument(volume_id_argument().required(true))
  );
  static ref VOLUME_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![VOLUME_CREATE_CAPABILITY.as_ref(), VOLUME_DELETE_CAPABILITY.as_ref(), VOLUME_LIST_CAPABILITY.as_ref(), VOLUME_SHOW_CAPABILITY.as_ref()];
}

const SIZE_OPTION: &str = "size";

fn size_option() -> Arg {
  Arg::new(SIZE_OPTION)
    .long("size")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..))
    .value_name("GIGABYTES")
    .help("Size in gigabytes")
    .long_help("Size in gigabytes for the created volume.")
}

struct VolumeCreate {}

#[async_trait]
impl CommandExecutor for VolumeCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("create new volume '{}'", volume_id));
    if client.get_volume(&volume_id).await.is_ok() {
      return err!("volume '{}' already exists", volume_id);
    }
    let size_gi_b: i64 = match matches.get_one::<i64>(SIZE_OPTION) {
      Some(size) => *size,
      None => {
        let line = context.read_single_line("enter size in gigabytes")?;
        line.parse::<i64>().map_err(|_| cli_error!("could not parse '{}' as a valid integer", line))?
      }
    };
    let volume = Volume { size_gi_b };
    if context.dry_run() {
      context.print_warning("dry-run mode, volume not created");
    } else {
      client.put_volume_configuration(&volume_id, &volume).await?;
      context.print_outcome("volume created");
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeDelete {}

#[async_trait]
impl CommandExecutor for VolumeDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if client.get_volume(&volume_id).await.is_err() {
      return err!("volume '{}' does not exists", volume_id);
    }
    let (_, volume_dependants) = client.volume_with_dependants(&volume_id).await?;
    if context.dependencies_warning("volume", volume_dependants, &volume_id) && !context.confirmed("do you want to continue?")? {
      context.print_outcome(format!("cancelled, volume '{}' not deleted", volume_id));
      return Ok(());
    }
    if context.confirmed(format!("delete volume '{}'?", volume_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, volume not deleted");
      } else {
        client.delete_volume_configuration(&volume_id).await?;
        context.print_outcome("volume deleted");
      }
    } else {
      context.print_outcome("cancelled, volume not deleted");
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) static VOLUME_LABELS: [VolumeLabel; 2] = [VolumeLabel::Target, VolumeLabel::Size];

struct VolumeList {}

#[async_trait]
impl CommandExecutor for VolumeList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all volumes with their parameters");
    let start_instant = context.now();
    let volume_ids = client.get_volume_ids().await?;
    let volumes = try_join_all(volume_ids.iter().map(|volume_id| client.get_volume_configuration(volume_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&VOLUME_LABELS, context);
    formatter.push_target_ids_and_values(volume_ids.as_slice(), volumes.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeListAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeListAllocationStatus {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all volumes with their allocation status");
    let start_instant = context.now();
    let volume_ids = client.get_volume_ids().await?;
    let allocation_statuses = try_join_all(volume_ids.iter().map(|volume_id| client.get_volume_status(volume_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
    formatter.push_target_ids_and_values(volume_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeListConfiguration {}

#[async_trait]
impl CommandExecutor for VolumeListConfiguration {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all volumes with their configurations");
    let start_instant = context.now();
    let volume_ids = client.get_volume_ids().await?;
    let configurations = try_join_all(volume_ids.iter().map(|volume_id| client.get_volume_configuration(volume_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&VOLUME_LABELS, context);
    formatter.push_target_ids_and_values(volume_ids.as_slice(), configurations.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeListIds {}

#[async_trait]
impl CommandExecutor for VolumeListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list volume ids");
    let start_instant = context.now();
    let volume_ids = client.get_volume_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("volume id", context);
    formatter.push_target_ids(volume_ids.as_slice());
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeListUsage {}

#[async_trait]
impl CommandExecutor for VolumeListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all volumes that are used in apps or services");
    let start_instant = context.now();
    let volumes_with_usage: Vec<(String, Vec<Dependant<VolumeInjection>>)> = client.volumes_with_dependants().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, context);
    for (volume_id, used_bys) in &volumes_with_usage {
      for used_by in used_bys {
        formatter.push_target_id_value(volume_id.clone(), used_by);
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no volumes found in apps or services");
    } else {
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static VOLUME_STATUS_LABELS: [VolumeLabel; 4] = [VolumeLabel::Target, VolumeLabel::Size, VolumeLabel::ConfigurationSize, VolumeLabel::ActualSize];

struct VolumeShow {}

#[async_trait]
impl CommandExecutor for VolumeShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for volume '{}'", volume_id));
    let start_instant = context.now();
    let (volume, allocation_status) = join!(client.get_volume(&volume_id), client.get_volume_status(&volume_id));
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, VOLUME_SUBJECT_TARGET);
    UnitFormatter::new(volume_id, &VOLUME_STATUS_LABELS, context).print(&volume?, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeShowAllocationStatus {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the allocation status for volume '{}'", volume_id));
    let start_instant = context.now();
    let allocation_status = client.get_volume_status(&volume_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(volume_id, &DEFAULT_ALLOCATION_STATUS_LABELS, context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct VolumeShowUsage {}

#[async_trait]
impl CommandExecutor for VolumeShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the apps and services that use volume '{}'", volume_id));
    let start_instant = context.now();
    let (_, usages) = client.volume_with_dependants(&volume_id).await?;
    context.print_execution_time(start_instant);
    let (dependant_apps, dependant_services): (Vec<DependantApp>, Vec<DependantApplication<VolumeInjection>>) = usages.into_iter().partition_map(|dependant| match dependant {
      Dependant::App { app } => Either::Left(app),
      Dependant::Application { application } => Either::Right(application),
      _ => unreachable!(),
    });
    if dependant_services.is_empty() {
      context.print_outcome("volume not used in services")
    } else {
      context.print_outcome("volume used in services");
      let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_SERVICES, context);
      formatter.push_values(&dependant_services);
      formatter.print(None)?;
    }
    if dependant_apps.is_empty() {
      context.print_outcome("volume not used in apps")
    } else {
      context.print_outcome("volume used in apps");
      let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_APPS, context);
      formatter.push_values(&dependant_apps);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum VolumeLabel {
  ActualSize,
  ConfigurationSize,
  Size,
  Target,
}

impl Label for VolumeLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::ActualSize => "actual size",
      Self::ConfigurationSize => "configured size",
      Self::Size => "size",
      Self::Target => "volume id",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::ActualSize => "actual",
      Self::ConfigurationSize => "conf",
      Self::Size => "size",
      Self::Target => "volume id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }

  fn column_alignment(&self) -> ColumnAlignment {
    match self {
      Self::ActualSize | Self::ConfigurationSize | Self::Size => ColumnAlignment::Right,
      _ => ColumnAlignment::default(),
    }
  }
}

impl SubjectFormatter<VolumeLabel> for Volume {
  fn value(&self, label: &VolumeLabel, target_id: &str) -> Value {
    match label {
      VolumeLabel::Target => Value::target(target_id),
      VolumeLabel::Size => Value::plain(self.size_gi_b),
      _ => Value::not_applicable(),
    }
  }
}

impl SubjectFormatter<VolumeLabel> for VolumeStatus {
  fn value(&self, label: &VolumeLabel, target_id: &str) -> Value {
    match label {
      VolumeLabel::ActualSize => Value::some_or(self.actual.clone().map(|volume| volume.size_gi_b), "NA"),
      VolumeLabel::ConfigurationSize => Value::some_or(self.configuration.clone().map(|volume| volume.size_gi_b), "NA"),
      VolumeLabel::Size => Value::some_or(self.actual.clone().map(|volume| volume.size_gi_b), "NA"),
      VolumeLabel::Target => Value::target(target_id),
    }
  }
}
