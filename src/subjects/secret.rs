use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::modifier_flags::ModifierFlagType;
use crate::subject::Subject;
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, USED_BY_LABELS, USED_BY_LABELS_LIST};
use crate::{read_multi_line, read_single_line, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::Secret;
use dsh_api::{secret, UsedBy};
use futures::future::try_join_all;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::time::Instant;

pub(crate) struct SecretSubject {}

const SECRET_SUBJECT_TARGET: &str = "secret";

lazy_static! {
  pub static ref SECRET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SecretSubject {});
}

#[async_trait]
impl Subject for SecretSubject {
  fn subject(&self) -> &'static str {
    SECRET_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH secrets.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list secrets used by the applications/services and apps on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("s")
  }

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Create, SECRET_CREATE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Delete, SECRET_DELETE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, SECRET_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, SECRET_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref SECRET_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Create, "Create secret")
      .set_long_about("Create a secret.")
      .set_default_command_executor(&SecretCreate {})
      .add_target_argument(target_argument(SECRET_SUBJECT_TARGET, None))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  );
  pub static ref SECRET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Delete, "Delete secret")
      .set_long_about("Delete a secret.")
      .set_default_command_executor(&SecretDelete {})
      .add_target_argument(target_argument(SECRET_SUBJECT_TARGET, None))
  );
  pub static ref SECRET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List secrets")
      .set_long_about("Lists all secrets used by the applications/services and apps on the DSH.")
      .set_default_command_executor(&SecretListIds {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &SecretListAllocationStatus {}, None),
        (FlagType::System, &SecretListSystem {}, None),
        (FlagType::Usage, &SecretListUsage {}, None),
      ])
      .add_filter_flags(vec![
        (FilterFlagType::App, Some("List all apps that use the secret.".to_string())),
        (FilterFlagType::Application, Some("List all applications that use the secret.".to_string())),
      ])
  );
  pub static ref SECRET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show secret configuration or value")
      .set_default_command_executor(&SecretShowAllocationStatus {})
      .add_command_executors(vec![(FlagType::Usage, &SecretShowUsage {}, None), (FlagType::Value, &SecretShowValue {}, None),])
      .add_target_argument(target_argument(SECRET_SUBJECT_TARGET, None))
  );
}

struct SecretCreate {}

#[async_trait]
impl CommandExecutor for SecretCreate {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    if matches.get_flag(ModifierFlagType::MultiLine.id()) {
      let secret_id = target.unwrap_or_else(|| unreachable!());
      context.print_explanation(format!("create new multi-line secret '{}'", secret_id));
      if context.dsh_api_client.as_ref().unwrap().get_secret(&secret_id).await.is_ok() {
        return Err(format!("secret '{}' already exists", secret_id));
      }
      context.print_prompt("enter multi-line secret (terminate input with ctrl-d after last line)");
      let value = read_multi_line()?;
      let secret = Secret { name: secret_id.clone(), value };
      if context.dry_run {
        context.print_warning("dry-run mode, secret not created");
      } else {
        context.dsh_api_client.as_ref().unwrap().create_secret(&secret).await?;
        context.print_outcome(format!("secret {} created", secret_id));
      }
      Ok(())
    } else {
      let secret_id = target.unwrap_or_else(|| unreachable!());
      context.print_explanation(format!("create new single line secret '{}'", secret_id));
      if context.dsh_api_client.as_ref().unwrap().get_secret(&secret_id).await.is_ok() {
        return Err(format!("secret '{}' already exists", secret_id));
      }
      let value = read_single_line("enter secret: ")?;
      let secret = Secret { name: secret_id.clone(), value };
      if context.dry_run {
        context.print_warning("dry-run mode, secret not created");
      } else {
        context.dsh_api_client.as_ref().unwrap().create_secret(&secret).await?;
        context.print_outcome(format!("secret {} created", secret_id));
      }
      Ok(())
    }
  }
}

struct SecretDelete {}

#[async_trait]
impl CommandExecutor for SecretDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete secret '{}'", secret_id));
    if context.dsh_api_client.as_ref().unwrap().get_secret_configuration(&secret_id).await.is_err() {
      return Err(format!("secret '{}' does not exists", secret_id));
    }
    if context.confirmed(format!("type 'yes' to delete secret '{}': ", secret_id).as_str())? {
      if context.dry_run {
        context.print_warning("dry-run mode, secret not deleted");
      } else {
        context.dsh_api_client.as_ref().unwrap().delete_secret(&secret_id).await?;
        context.print_outcome(format!("secret {} deleted", secret_id));
      }
    } else {
      context.print_outcome(format!("cancelled, secret {} not deleted", secret_id));
    }
    Ok(())
  }
}

struct SecretListAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all secrets with their allocation status");
    let start_instant = Instant::now();
    let non_system_secret_ids = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_secret_ids()
      .await?
      .into_iter()
      .filter(|id| !secret::is_system_secret(id))
      .collect::<Vec<_>>();
    let allocation_statuses = try_join_all(
      non_system_secret_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_secret_allocation_status(id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("secret id"), context);
    formatter.push_target_ids_and_values(non_system_secret_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct SecretListSystem {}

#[async_trait]
impl CommandExecutor for SecretListSystem {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all system secret ids");
    let start_instant = Instant::now();
    let system_secret_ids = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_secret_ids()
      .await?
      .into_iter()
      .filter(|id| secret::is_system_secret(id))
      .collect::<Vec<_>>();
    let allocation_statuses = try_join_all(
      system_secret_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_secret_allocation_status(id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("system secret id"), context);
    formatter.push_target_ids_and_values(system_secret_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct SecretListIds {}

#[async_trait]
impl CommandExecutor for SecretListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all secret ids");
    let start_instant = Instant::now();
    let non_system_secrets = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_secret_ids()
      .await?
      .into_iter()
      .filter(|id| !secret::is_system_secret(id))
      .collect::<Vec<_>>();
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("secret id", context);
    formatter.push_target_ids(non_system_secrets.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all secrets that are used in apps or applications");
    let start_instant = Instant::now();
    let secrets_with_usage: Vec<(String, Vec<UsedBy>)> = context.dsh_api_client.as_ref().unwrap().list_secrets_with_usage().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("secret id"), context);
    for (secret_id, used_bys) in &secrets_with_usage {
      let mut first = true;
      for used_by in used_bys {
        if first {
          formatter.push_target_id_value(secret_id.clone(), used_by);
        } else {
          formatter.push_target_id_value("".to_string(), used_by);
        }
        first = false;
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no secrets found in apps or applications");
    } else {
      formatter.print()?;
    }
    Ok(())
  }
}

struct SecretShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for secret '{}'", secret_id));
    let start_instant = Instant::now();
    let allocation_status = context.dsh_api_client.as_ref().unwrap().get_secret_allocation_status(secret_id.as_str()).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(secret_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("secret id"), &allocation_status, context).print()?;
    Ok(())
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the apps and applications that use secret '{}'", secret_id));
    let start_instant = Instant::now();
    let usages = context.dsh_api_client.as_ref().unwrap().get_secret_with_usage(secret_id.as_str()).await?;
    context.print_execution_time(start_instant);
    if usages.is_empty() {
      context.print_outcome("secret not used")
    } else {
      let mut formatter = ListFormatter::new(&USED_BY_LABELS, Some("secret id"), context);
      formatter.push_values(&usages);
      formatter.print()?;
    }
    Ok(())
  }
}

struct SecretShowValue {}

#[async_trait]
impl CommandExecutor for SecretShowValue {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the value of secret '{}'", secret_id));
    let start_instant = Instant::now();
    let secret = context.dsh_api_client.as_ref().unwrap().get_secret(secret_id.as_str()).await?;
    context.print_execution_time(start_instant);
    context.print(secret);
    Ok(())
  }
}