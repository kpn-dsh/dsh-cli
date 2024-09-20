use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{Application, Volume};

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::formatter::{print_ids, TableBuilder};
use crate::formatters::usage::{Usage, UsageLabel, USAGE_LABELS_LIST, USAGE_LABELS_SHOW};
use crate::formatters::volume::{VOLUME_LABELS, VOLUME_STATUS_LABELS};
use crate::subject::Subject;
use crate::{confirmed, read_single_line, DcliContext, DcliResult};

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

  fn subject_first_upper(&self) -> &'static str {
    "Volume"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH volumes.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list volumes deployed on the DSH.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
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
  pub static ref VOLUME_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Create,
    command_about: "Create volume".to_string(),
    command_long_about: Some("Create a volume.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![],
    default_command_executor: Some(&VolumeCreate {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref VOLUME_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Delete,
    command_about: "Delete volume".to_string(),
    command_long_about: Some("Delete a volume.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![],
    default_command_executor: Some(&VolumeDelete {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref VOLUME_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List volumes".to_string(),
    command_long_about: Some("Lists all available volumes.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &VolumeListAll {}, None),
      (FlagType::AllocationStatus, &VolumeListAllocationStatus {}, None),
      (FlagType::Configuration, &VolumeListConfiguration {}, None),
      (FlagType::Ids, &VolumeListIds {}, None),
      (FlagType::Usage, &VolumeListUsage {}, None),
    ],
    default_command_executor: Some(&VolumeListIds {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref VOLUME_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show volume configuration".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &VolumeShowAll {}, None),
      (FlagType::AllocationStatus, &VolumeShowAllocationStatus {}, None),
      (FlagType::Configuration, &VolumeShowConfiguration {}, None),
      (FlagType::Usage, &VolumeShowUsage {}, None),
    ],
    default_command_executor: Some(&VolumeShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct VolumeCreate {}

#[async_trait]
impl CommandExecutor for VolumeCreate {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("create new volume '{}'", volume_id);
    }
    if dsh_api_client.get_volume(&volume_id).await.is_ok() {
      return Err(format!("volume '{}' already exists", volume_id));
    }
    println!("enter size in gigabytes, followed by Enter");
    let line = read_single_line()?;
    let size_gi_b = line.parse::<i64>().map_err(|_| format!("could not parse '{}' as a valid integer", line))?;
    let volume = Volume { size_gi_b };
    dsh_api_client.create_volume(&volume_id, &volume).await?;
    println!("ok");
    Ok(true)
  }
}

struct VolumeDelete {}

#[async_trait]
impl CommandExecutor for VolumeDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("delete volume '{}'", volume_id);
    }
    if dsh_api_client.get_volume(&volume_id).await.is_err() {
      return Err(format!("volume '{}' does not exists", volume_id));
    }
    println!("type 'yes' and Enter to delete volume '{}'", volume_id);
    if confirmed()? {
      dsh_api_client.delete_volume(&volume_id).await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with their parameters");
    }
    let volume_ids = dsh_api_client.get_volume_ids().await?;
    let volumes = try_join_all(volume_ids.iter().map(|volume_id| dsh_api_client.get_volume(volume_id.as_str()))).await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with their allocation status");
    }
    let volume_ids = dsh_api_client.get_volume_ids().await?;
    let allocation_statuses = try_join_all(volume_ids.iter().map(|volume_id| dsh_api_client.get_volume_allocation_status(volume_id.as_str()))).await?;
    print_allocation_statuses(volume_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct VolumeListConfiguration {}

#[async_trait]
impl CommandExecutor for VolumeListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with their configurations");
    }
    let volume_ids = dsh_api_client.get_volume_ids().await?;
    let configurations = try_join_all(volume_ids.iter().map(|volume_id| dsh_api_client.get_volume_configuration(volume_id.as_str()))).await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list volume ids");
    }
    print_ids("volume ids".to_string(), dsh_api_client.get_volume_ids().await?, context);
    Ok(false)
  }
}

struct VolumeListUsage {}

#[async_trait]
impl CommandExecutor for VolumeListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all volumes with the applications that use them");
    }
    let (volume_ids, applications) = try_join!(dsh_api_client.get_volume_ids(), dsh_api_client.get_application_configurations())?;
    let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_LABELS_LIST, context);
    for volume_id in &volume_ids {
      let usages: Vec<(String, String)> = applications_that_use_volume(volume_id, &applications);
      if usages.is_empty() {
        builder.row(&Usage::empty(volume_id.to_string()));
      } else {
        let mut first = true;
        for (application_id, path) in usages {
          if first {
            builder.row(&Usage::application(volume_id.to_string(), application_id, vec![path]));
          } else {
            builder.row(&Usage::application("".to_string(), application_id, vec![path]));
          }
          first = false;
        }
      }
    }
    builder.print();
    Ok(false)
  }
}

struct VolumeShowAll {}

#[async_trait]
impl CommandExecutor for VolumeShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for volume '{}'", volume_id);
    }
    let mut builder = TableBuilder::show(&VOLUME_STATUS_LABELS, context);
    builder.value(volume_id.clone(), &dsh_api_client.get_volume(volume_id.as_str()).await?);
    builder.print();
    Ok(false)
  }
}

struct VolumeShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for VolumeShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the allocation status for volume '{}'", volume_id);
    }
    print_allocation_status(volume_id.clone(), dsh_api_client.get_volume_allocation_status(volume_id.as_str()).await?, context);
    Ok(false)
  }
}

struct VolumeShowConfiguration {}

#[async_trait]
impl CommandExecutor for VolumeShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the configuration for volume '{}'", volume_id);
    }
    let mut builder = TableBuilder::show(&VOLUME_STATUS_LABELS, context);
    builder.value(volume_id.clone(), &dsh_api_client.get_volume(volume_id.as_str()).await?);
    builder.print();
    Ok(false)
  }
}

struct VolumeShowUsage {}

#[async_trait]
impl CommandExecutor for VolumeShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let volume_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the applications that use volume '{}'", volume_id);
    }
    let applications = dsh_api_client.get_application_configurations().await?;
    let usages: Vec<(String, String)> = applications_that_use_volume(volume_id.as_str(), &applications);
    if !usages.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_LABELS_SHOW, context);
      for (application_id, path) in usages {
        builder.row(&Usage::application(application_id.clone(), application_id.to_string(), vec![path]));
      }
      builder.print();
    } else {
      println!("volume not used")
    }
    Ok(false)
  }
}

pub(crate) fn applications_that_use_volume(volume_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, String)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, String)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    for (path, volume) in application.volumes.clone() {
      if volume.name.contains(&format!("volume('{}')", volume_id)) {
        pairs.push((application_id.clone(), path))
      }
    }
  }
  pairs
}
