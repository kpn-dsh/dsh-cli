use crate::formatters::formatter::{Label, SubjectFormatter};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::{Volume, VolumeStatus};
use dsh_api::UsedBy;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::Subject;
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, USED_BY_LABELS, USED_BY_LABELS_LIST};
use crate::{read_single_line, DshCliResult};

pub(crate) struct VolumeSubject {}

const VOLUME_SUBJECT_TARGET: &str = "volume";

lazy_static! {
  pub static ref VOLUME_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(VolumeSubject {});
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

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Create, VOLUME_CREATE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Delete, VOLUME_DELETE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, VOLUME_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, VOLUME_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref VOLUME_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Create, "Create volume")
      .set_long_about("Create a volume.")
      .set_default_command_executor(&VolumeCreate {})
      .add_target_argument(target_argument(VOLUME_SUBJECT_TARGET, None))
  );
  pub static ref VOLUME_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Delete, "Delete volume")
      .set_long_about("Delete a volume.")
      .set_default_command_executor(&VolumeDelete {})
      .add_target_argument(target_argument(VOLUME_SUBJECT_TARGET, None))
  );
  pub static ref VOLUME_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List volumes")
      .set_long_about("Lists all available volumes.")
      .set_default_command_executor(&VolumeListAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &VolumeListAllocationStatus {}, None),
        (FlagType::Configuration, &VolumeListConfiguration {}, None),
        (FlagType::Ids, &VolumeListIds {}, None),
        (FlagType::Usage, &VolumeListUsage {}, None),
      ])
      .set_run_all_executors(true)
      .add_filter_flags(vec![
        (FilterFlagType::App, Some("List all apps that use the volume.".to_string())),
        (FilterFlagType::Application, Some("List all applications that use the volume.".to_string()))
      ])
  );
  pub static ref VOLUME_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show secret configuration")
      .set_default_command_executor(&VolumeShowAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &VolumeShowAllocationStatus {}, None),
        (FlagType::Usage, &VolumeShowUsage {}, None),
      ])
      .add_target_argument(target_argument(VOLUME_SUBJECT_TARGET, None))
  );
}

struct VolumeCreate {}

#[async_trait]
impl CommandExecutor for VolumeCreate {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("create new volume '{}'", volume_id));
    if context.dsh_api_client.as_ref().unwrap().get_volume(&volume_id).await.is_ok() {
      return Err(format!("volume '{}' already exists", volume_id));
    }
    let line = read_single_line("enter size in gigabytes: ")?;
    let size_gi_b = line.parse::<i64>().map_err(|_| format!("could not parse '{}' as a valid integer", line))?;
    let volume = Volume { size_gi_b };
    if context.dry_run {
      context.print_warning("dry-run mode, volume not created");
    } else {
      context.dsh_api_client.as_ref().unwrap().create_volume(&volume_id, &volume).await?;
      context.print_outcome("volume created");
    }
    Ok(())
  }
}

struct VolumeDelete {}

#[async_trait]
impl CommandExecutor for VolumeDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete volume '{}'", volume_id));
    if context.dsh_api_client.as_ref().unwrap().get_volume(&volume_id).await.is_err() {
      return Err(format!("volume '{}' does not exists", volume_id));
    }
    if context.confirmed(format!("type 'yes' to delete volume '{}': ", volume_id))? {
      if context.dry_run {
        context.print_warning("dry-run mode, volume not deleted");
      } else {
        context.dsh_api_client.as_ref().unwrap().delete_volume(&volume_id).await?;
        context.print_outcome("volume deleted");
      }
    } else {
      context.print_outcome("cancelled, volume not deleted");
    }
    Ok(())
  }
}

struct VolumeListAll {}

#[async_trait]
impl CommandExecutor for VolumeListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all volumes with their parameters");
    let start_instant = Instant::now();
    let volume_ids = context.dsh_api_client.as_ref().unwrap().list_volume_ids().await?;
    let volumes = try_join_all(
      volume_ids
        .iter()
        .map(|volume_id| context.dsh_api_client.as_ref().unwrap().get_volume(volume_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&VOLUME_LABELS, Some("volume id"), context);
    formatter.push_target_ids_and_values(volume_ids.as_slice(), volumes.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct VolumeListAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all volumes with their allocation status");
    let start_instant = Instant::now();
    let volume_ids = context.dsh_api_client.as_ref().unwrap().list_volume_ids().await?;
    let allocation_statuses = try_join_all(
      volume_ids
        .iter()
        .map(|volume_id| context.dsh_api_client.as_ref().unwrap().get_volume_allocation_status(volume_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("volume id"), context);
    formatter.push_target_ids_and_values(volume_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct VolumeListConfiguration {}

#[async_trait]
impl CommandExecutor for VolumeListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all volumes with their configurations");
    let start_instant = Instant::now();
    let volume_ids = context.dsh_api_client.as_ref().unwrap().list_volume_ids().await?;
    let configurations = try_join_all(
      volume_ids
        .iter()
        .map(|volume_id| context.dsh_api_client.as_ref().unwrap().get_volume_configuration(volume_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&VOLUME_LABELS, Some("volume id"), context);
    formatter.push_target_ids_and_values(volume_ids.as_slice(), configurations.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct VolumeListIds {}

#[async_trait]
impl CommandExecutor for VolumeListIds {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list volume ids");
    let start_instant = Instant::now();
    let volume_ids = context.dsh_api_client.as_ref().unwrap().list_volume_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("volume id", context);
    formatter.push_target_ids(volume_ids.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct VolumeListUsage {}

#[async_trait]
impl CommandExecutor for VolumeListUsage {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all volumes that are used in apps or applications");
    let start_instant = Instant::now();
    let volumes_with_usage: Vec<(String, Vec<UsedBy>)> = context.dsh_api_client.as_ref().unwrap().list_volumes_with_usage().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("volume id"), context);
    for (volume_id, used_bys) in &volumes_with_usage {
      let mut first = true;
      for used_by in used_bys {
        if first {
          formatter.push_target_id_value(volume_id.clone(), used_by);
        } else {
          formatter.push_target_id_value("".to_string(), used_by);
        }
        first = false;
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no volumes found in apps or applications");
    } else {
      formatter.print()?;
    }
    Ok(())
  }
}

struct VolumeShowAll {}

#[async_trait]
impl CommandExecutor for VolumeShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for volume '{}'", volume_id));
    let start_instant = Instant::now();
    let volume = context.dsh_api_client.as_ref().unwrap().get_volume(volume_id.as_str()).await?;
    context.print_execution_time(start_instant);
    let formatter = UnitFormatter::new(volume_id, &VOLUME_STATUS_LABELS, Some("volume id"), &volume, context);
    formatter.print()?;
    Ok(())
  }
}

struct VolumeShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the allocation status for volume '{}'", volume_id));
    let start_instant = Instant::now();
    let allocation_status = context.dsh_api_client.as_ref().unwrap().get_volume_allocation_status(volume_id.as_str()).await?;
    context.print_execution_time(start_instant);
    let formatter = UnitFormatter::new(volume_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("volume id"), &allocation_status, context);
    formatter.print()?;
    Ok(())
  }
}

struct VolumeShowUsage {}

#[async_trait]
impl CommandExecutor for VolumeShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the apps and applications that use volume '{}'", volume_id));
    let start_instant = Instant::now();
    let (_, usages) = context.dsh_api_client.as_ref().unwrap().get_volume_with_usage(volume_id.as_str()).await?;
    context.print_execution_time(start_instant);
    if usages.is_empty() {
      context.print_outcome("volume not used")
    } else {
      let mut formatter = ListFormatter::new(&USED_BY_LABELS, Some("volume id"), context);
      formatter.push_values(&usages);
      formatter.print()?;
    }
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum VolumeLabel {
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
}

impl SubjectFormatter<VolumeLabel> for Volume {
  fn value(&self, label: &VolumeLabel, target_id: &str) -> String {
    match label {
      VolumeLabel::Target => target_id.to_string(),
      VolumeLabel::Size => self.size_gi_b.to_string(),
      _ => "".to_string(),
    }
  }

  fn target_label(&self) -> Option<VolumeLabel> {
    Some(VolumeLabel::Target)
  }
}

impl SubjectFormatter<VolumeLabel> for VolumeStatus {
  fn value(&self, label: &VolumeLabel, target_id: &str) -> String {
    match label {
      VolumeLabel::ActualSize => self.actual.clone().map(|a| a.size_gi_b.to_string()).unwrap_or("NA".to_string()),
      VolumeLabel::ConfigurationSize => self.configuration.clone().map(|a| a.size_gi_b.to_string()).unwrap_or("NA".to_string()),
      VolumeLabel::Size => self.actual.clone().map(|a| a.size_gi_b.to_string()).unwrap_or("NA".to_string()),
      VolumeLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<VolumeLabel> {
    Some(VolumeLabel::Target)
  }
}

pub static VOLUME_LABELS: [VolumeLabel; 2] = [VolumeLabel::Target, VolumeLabel::Size];

pub static VOLUME_STATUS_LABELS: [VolumeLabel; 4] = [VolumeLabel::Target, VolumeLabel::Size, VolumeLabel::ConfigurationSize, VolumeLabel::ActualSize];
