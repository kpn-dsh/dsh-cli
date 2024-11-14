use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
use std::collections::HashMap;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::platform::DshPlatform;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::formatters::allocation_status::print_allocation_status;
use crate::formatters::list_table::ListTable;
use crate::formatters::settings::TARGET_LABELS;
use crate::settings::{all_targets, delete_target, read_target, upsert_target, Target};
use crate::subject::Subject;
use crate::{confirmed, read_single_line, DcliContext, DcliResult};

pub(crate) struct SettingSubject {}

const SETTING_SUBJECT_TARGET: &str = "setting";

lazy_static! {
  pub static ref SETTING_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SettingSubject {});
}

#[async_trait]
impl Subject for SettingSubject {
  fn subject(&self) -> &'static str {
    SETTING_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Setting"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list dcli settings.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list dcli settings.".to_string()
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Delete, SETTING_DELETE_TARGET_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, SETTING_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::New, SETTING_NEW_TARGET_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, SETTING_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref SETTING_DELETE_TARGET_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Delete,
    command_about: "Delete target".to_string(),
    command_long_about: Some("Delete a target.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SettingDeleteTarget {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SETTING_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List settings".to_string(),
    command_long_about: Some("Lists all dcli settings.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SettingList {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SETTING_NEW_TARGET_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::New,
    command_about: "Create new target".to_string(),
    command_long_about: Some("Create a new target.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SettingNewTarget {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SETTING_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show setting".to_string(),
    command_long_about: None,
    command_executors: vec![],
    default_command_executor: Some(&SettingShow {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
}

struct SettingDeleteTarget {}

#[async_trait]
impl CommandExecutor for SettingDeleteTarget {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, _dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("delete existing target");
    }
    let platform = read_single_line("enter platform: ")?;
    let platform = DshPlatform::try_from(platform.as_str())?;
    let tenant = read_single_line("enter tenant: ")?;
    match read_target(&platform, &tenant)? {
      Some(target) => {
        if confirmed(format!("type 'yes' to delete target '{}': ", target).as_str())? {
          delete_target(&platform, &tenant)?;
          println!("target '{}' deleted", target);
        } else {
          println!("cancelled");
        }
      }
      None => {
        return Err(format!("target {}@{} does not exist", tenant, platform));
      }
    }

    Ok(false)
  }
}

struct SettingList {}

#[async_trait]
impl CommandExecutor for SettingList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, _dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all targets");
    }
    let mut table = ListTable::new(&TARGET_LABELS, context);
    table.rows(all_targets()?.as_slice());
    table.print();
    Ok(false)
  }
}

struct SettingNewTarget {}

#[async_trait]
impl CommandExecutor for SettingNewTarget {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &DcliContext, _dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("create new target");
    }
    let platform = read_single_line("enter platform: ")?;
    let platform = DshPlatform::try_from(platform.as_str())?;
    let tenant = read_single_line("enter tenant: ")?;
    if let Some(existing_target) = read_target(&platform, &tenant)? {
      return Err(format!("target {} already exists (first delete the existing target)", existing_target));
    }
    let guid = read_single_line("enter group/user id: ")?;
    let guid = match guid.parse::<u32>() {
      Ok(guid) => {
        if guid > 0 && guid < 32768 {
          // TODO Check these bounds
          format!("{}:{}", guid, guid)
        } else {
          return Err("group/user id must be greater than 0 and smaller than 32768".to_string());
        }
      }
      Err(_) => return Err("invalid group/user id (single integer required)".to_string()),
    };
    let password = read_single_line("enter password: ")?;
    let target = Target::new(platform, tenant, guid, password)?;
    upsert_target(&target)?;
    println!("target {} created", target);
    Ok(false)
  }
}

struct SettingShow {}

#[async_trait]
impl CommandExecutor for SettingShow {
  async fn execute(&self, argument: Option<String>, _sub_argument: Option<String>, _matches: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let secret_id = argument.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show allocation status for secret '{}'", secret_id);
    }
    let allocation_status = dsh_api_client.get_secret_allocation_status(secret_id.as_str()).await?;
    print_allocation_status(secret_id, allocation_status, context);
    Ok(false)
  }
}
