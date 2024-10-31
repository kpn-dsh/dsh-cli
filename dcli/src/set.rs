use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::Secret;

use crate::{confirmed, DcliContext, DcliResult, read_multi_line, read_single_line};
use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::formatters::allocation_status::print_allocation_status;
use crate::formatters::list_table::ListTable;
use crate::formatters::settings::TARGET_LABELS;
use crate::modifier_flags::ModifierFlagType;
use crate::settings::all_targets;
use crate::subject::Subject;

pub(crate) struct SetSubject {}

const SET_SUBJECT_TARGET: &str = "set";

lazy_static! {
  pub static ref SET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SetSubject {});
}

#[async_trait]
impl Subject for SetSubject {
  fn subject(&self) -> &'static str {
    SET_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Set"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list dcli settings.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list dcli settings.".to_string()
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Create, SET_CREATE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Delete, SET_DELETE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, SET_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, SET_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref SET_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Create,
    command_about: "Create setting".to_string(),
    command_long_about: Some("Create a setting.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SetCreate {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Delete,
    command_about: "Delete setting".to_string(),
    command_long_about: Some("Delete a setting.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SetDelete {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List settings".to_string(),
    command_long_about: Some("Lists all dcli settings.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SetList {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show setting".to_string(),
    command_long_about: None,
    command_executors: vec![],
    default_command_executor: Some(&SetShow {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
}

struct SetCreate {}

#[async_trait]
impl CommandExecutor for SetCreate {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if matches.get_flag(ModifierFlagType::MultiLine.id()) {
      let secret_id = target.unwrap_or_else(|| unreachable!());
      if context.show_capability_explanation() {
        println!("create new multi-line secret '{}'", secret_id);
      }
      if dsh_api_client.get_secret(&secret_id).await.is_ok() {
        return Err(format!("secret '{}' already exists", secret_id));
      }
      println!("enter multi-line secret (terminate input with ctrl-d after last line)");
      let value = read_multi_line()?;
      let secret = Secret { name: secret_id, value };
      dsh_api_client.create_secret(&secret).await?;
      println!("ok");
      Ok(true)
    } else {
      let secret_id = target.unwrap_or_else(|| unreachable!());
      if context.show_capability_explanation() {
        println!("create new single line secret '{}'", secret_id);
      }
      if dsh_api_client.get_secret(&secret_id).await.is_ok() {
        return Err(format!("secret '{}' already exists", secret_id));
      }
      println!("enter secret followed by newline");
      let value = read_single_line()?;
      let secret = Secret { name: secret_id, value };
      dsh_api_client.create_secret(&secret).await?;
      println!("ok");
      Ok(true)
    }
  }
}

struct SetDelete {}

#[async_trait]
impl CommandExecutor for SetDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("delete secret '{}'", secret_id);
    }
    if dsh_api_client.get_secret_configuration(&secret_id).await.is_err() {
      return Err(format!("secret '{}' does not exists", secret_id));
    }
    println!("type 'yes' and Enter to delete secret '{}'", secret_id);
    if confirmed()? {
      dsh_api_client.delete_secret(&secret_id).await?;
      println!("ok");
    } else {
      println!("cancelled");
    }
    Ok(false)
  }
}

struct SetList {}

#[async_trait]
impl CommandExecutor for SetList {
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

struct SetShow {}

#[async_trait]
impl CommandExecutor for SetShow {
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
