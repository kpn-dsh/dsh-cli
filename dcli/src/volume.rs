use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use dsh_api::types::Volume;

use crate::app::apps_that_use_volume;
use crate::application::applications_that_use_volume;
use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::list_table::ListTable;
use crate::formatters::usage::{Usage, UsageLabel, USAGE_IN_APPLICATIONS_LABELS_LIST, USAGE_IN_APPS_LABELS_LIST, USAGE_LABELS_SHOW};
use crate::formatters::volume::{VOLUME_LABELS, VOLUME_STATUS_LABELS};
use crate::subject::Subject;
use crate::{confirmed, include_app_application, read_single_line, DcliContext, DcliResult};

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
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("create new volume '{}'", volume_id);
    }
    if context.dsh_api_client.as_ref().unwrap().get_volume(&volume_id).await.is_ok() {
      return Err(format!("volume '{}' already exists", volume_id));
    }
    let line = read_single_line("enter size in gigabytes: ")?;
    let size_gi_b = line.parse::<i64>().map_err(|_| format!("could not parse '{}' as a valid integer", line))?;
    let volume = Volume { size_gi_b };
    context.dsh_api_client.as_ref().unwrap().create_volume(&volume_id, &volume).await?;
    println!("ok");
    Ok(true)
  }
}

struct VolumeDelete {}

#[async_trait]
impl CommandExecutor for VolumeDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("delete volume '{}'", volume_id);
    }
    if context.dsh_api_client.as_ref().unwrap().get_volume(&volume_id).await.is_err() {
      return Err(format!("volume '{}' does not exists", volume_id));
    }
    if confirmed(format!("type 'yes' to delete volume '{}': ", volume_id).as_str())? {
      context.dsh_api_client.as_ref().unwrap().delete_volume(&volume_id).await?;
      println!("ok");
    } else {
      println!("cancelled");
    }
    Ok(false)
  }
}

struct VolumeListAll {}

#[async_trait]
impl CommandExecutor for VolumeListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with their parameters");
    }
    let volume_ids = context.dsh_api_client.as_ref().unwrap().get_volume_ids().await?;
    let volumes = try_join_all(
      volume_ids
        .iter()
        .map(|volume_id| context.dsh_api_client.as_ref().unwrap().get_volume(volume_id.as_str())),
    )
    .await?;
    let mut builder = TableBuilder::list(&VOLUME_STATUS_LABELS, context);
    for (volume_id, volume_status) in volume_ids.iter().zip(volumes) {
      builder.value(volume_id.to_string(), &volume_status);
    }
    builder.print();
    Ok(false)
  }
}

struct VolumeListAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with their allocation status");
    }
    let volume_ids = context.dsh_api_client.as_ref().unwrap().get_volume_ids().await?;
    let allocation_statuses = try_join_all(
      volume_ids
        .iter()
        .map(|volume_id| context.dsh_api_client.as_ref().unwrap().get_volume_allocation_status(volume_id.as_str())),
    )
    .await?;
    print_allocation_statuses(volume_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct VolumeListConfiguration {}

#[async_trait]
impl CommandExecutor for VolumeListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with their configurations");
    }
    let volume_ids = context.dsh_api_client.as_ref().unwrap().get_volume_ids().await?;
    let configurations = try_join_all(
      volume_ids
        .iter()
        .map(|volume_id| context.dsh_api_client.as_ref().unwrap().get_volume_configuration(volume_id.as_str())),
    )
    .await?;
    let mut builder = TableBuilder::list(&VOLUME_LABELS, context);
    for (volume_id, configuration) in volume_ids.iter().zip(configurations) {
      builder.value(volume_id.to_string(), &configuration);
    }
    builder.print();
    Ok(false)
  }
}

struct VolumeListIds {}

#[async_trait]
impl CommandExecutor for VolumeListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list volume ids");
    }
    print_vec("volume ids".to_string(), context.dsh_api_client.as_ref().unwrap().get_volume_ids().await?, context);
    Ok(false)
  }
}

struct VolumeListUsage {}

#[async_trait]
impl CommandExecutor for VolumeListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    let (include_app, include_application) = include_app_application(matches);
    if include_app {
      if context.show_capability_explanation() {
        println!("list all volumes that are used in apps");
      }
      let (volume_ids, apps) = try_join!(
        context.dsh_api_client.as_ref().unwrap().get_volume_ids(),
        context.dsh_api_client.as_ref().unwrap().get_app_configurations()
      )?;
      let mut table = ListTable::new(&USAGE_IN_APPS_LABELS_LIST, context);
      for volume_id in &volume_ids {
        let app_usages: Vec<(String, u64, String)> = apps_that_use_volume(volume_id.as_str(), &apps);
        let mut first = true;
        for (app_id, instances, path) in &app_usages {
          if first {
            table.row(&Usage::app(volume_id.to_string(), app_id.to_string(), *instances, vec![path.clone()]));
          } else {
            table.row(&Usage::app("".to_string(), app_id.to_string(), *instances, vec![path.clone()]));
          }
          first = false;
        }
      }
      if table.is_empty() {
        println!("no volumes found in apps");
      } else {
        table.print();
      }
    }
    if include_application {
      if context.show_capability_explanation() {
        println!("list all volumes that are used in applications");
      }
      let (volume_ids, applications) = try_join!(
        context.dsh_api_client.as_ref().unwrap().get_volume_ids(),
        context.dsh_api_client.as_ref().unwrap().get_applications()
      )?;
      let mut table = ListTable::new(&USAGE_IN_APPLICATIONS_LABELS_LIST, context);
      for volume_id in &volume_ids {
        let application_usages: Vec<(String, u64, String)> = applications_that_use_volume(volume_id, &applications);
        let mut first = true;
        for (application_id, instances, path) in application_usages {
          if first {
            table.row(&Usage::application(volume_id.to_string(), application_id, instances, vec![path]));
          } else {
            table.row(&Usage::application("".to_string(), application_id, instances, vec![path]));
          }
          first = false;
        }
      }
      if table.is_empty() {
        println!("no volumes found in applications");
      } else {
        table.print();
      }
    }
    Ok(false)
  }
}

struct VolumeShowAll {}

#[async_trait]
impl CommandExecutor for VolumeShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for volume '{}'", volume_id);
    }
    let mut builder = TableBuilder::show(&VOLUME_STATUS_LABELS, context);
    builder.value(volume_id.clone(), &context.dsh_api_client.as_ref().unwrap().get_volume(volume_id.as_str()).await?);
    builder.print();
    Ok(false)
  }
}

struct VolumeShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the allocation status for volume '{}'", volume_id);
    }
    print_allocation_status(
      volume_id.clone(),
      context.dsh_api_client.as_ref().unwrap().get_volume_allocation_status(volume_id.as_str()).await?,
      context,
    );
    Ok(false)
  }
}

struct VolumeShowUsage {}

#[async_trait]
impl CommandExecutor for VolumeShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the applications that use volume '{}'", volume_id);
    }
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    let usages: Vec<(String, u64, String)> = applications_that_use_volume(volume_id.as_str(), &applications);
    if !usages.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_LABELS_SHOW, context);
      for (application_id, instances, path) in usages {
        builder.row(&Usage::application(application_id.clone(), application_id.to_string(), instances, vec![path]));
      }
      builder.print();
    } else {
      println!("volume not used")
    }
    Ok(false)
  }
}
