use crate::arguments::secret_id_argument;
use crate::capability::{Capability, CommandExecutor, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, NEW_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS, UPDATE_COMMAND};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, USED_BY_LABELS, USED_BY_LABELS_LIST};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::Secret;
use dsh_api::{secret, UsedBy};
use futures::future::try_join_all;
use lazy_static::lazy_static;
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
    "Show, manage and list secrets used by the services and apps on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(SECRET_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(SECRET_LIST_CAPABILITY.as_ref()),
      NEW_COMMAND => Some(SECRET_NEW_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(SECRET_SHOW_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(SECRET_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &SECRET_CAPABILITIES
  }
}

lazy_static! {
  static ref SECRET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, "Delete secret")
      .set_long_about("Delete a secret.")
      .set_default_command_executor(&SecretDelete {})
      .add_target_argument(secret_id_argument().required(true))
  );
  static ref SECRET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List secrets")
      .set_long_about("Lists all secrets used by the services and apps on the DSH.")
      .set_default_command_executor(&SecretListIds {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &SecretListAllocationStatus {}, None),
        (FlagType::System, &SecretListSystem {}, None),
        (FlagType::Usage, &SecretListUsage {}, None),
      ])
      .add_filter_flags(vec![
        (FilterFlagType::App, Some("List all apps that use the secret.".to_string())),
        (FilterFlagType::Service, Some("List all services that use the secret.".to_string())),
      ])
  );
  static ref SECRET_NEW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(NEW_COMMAND, None, "Create new secret")
      .set_long_about("Create a new secret.")
      .set_default_command_executor(&SecretNew {})
      .add_target_argument(secret_id_argument().required(true))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  );
  static ref SECRET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), "Show secret configuration or value")
      .set_default_command_executor(&SecretShowAllocationStatus {})
      .add_command_executors(vec![(FlagType::Usage, &SecretShowUsage {}, None), (FlagType::Value, &SecretShowValue {}, None),])
      .add_target_argument(secret_id_argument().required(true))
  );
  static ref SECRET_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, "Update secret")
      .set_long_about("Update a secret.")
      .set_default_command_executor(&SecretUpdate {})
      .add_target_argument(secret_id_argument().required(true))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  );
  static ref SECRET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![SECRET_DELETE_CAPABILITY.as_ref(), SECRET_LIST_CAPABILITY.as_ref(), SECRET_NEW_CAPABILITY.as_ref(), SECRET_SHOW_CAPABILITY.as_ref(), SECRET_UPDATE_CAPABILITY.as_ref()];
}

struct SecretDelete {}

#[async_trait]
impl CommandExecutor for SecretDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete secret '{}'", secret_id));
    if context.client_unchecked().get_secret_configuration(&secret_id).await.is_err() {
      return Err(format!("secret '{}' does not exist", secret_id));
    }
    if context.confirmed(format!("type 'yes' to delete secret '{}': ", secret_id))? {
      if context.dry_run {
        context.print_warning("dry-run mode, secret not deleted");
      } else {
        context.client_unchecked().delete_secret_configuration(&secret_id).await?;
        context.print_outcome(format!("secret '{}' deleted", secret_id));
      }
    } else {
      context.print_outcome(format!("cancelled, secret '{}' not deleted", secret_id));
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretListAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all secrets with their allocation status");
    let start_instant = Instant::now();
    let non_system_secret_ids = context
      .client_unchecked()
      .get_secret_ids()
      .await?
      .into_iter()
      .filter(|id| !secret::is_system_secret(id))
      .collect::<Vec<_>>();
    let allocation_statuses = try_join_all(
      non_system_secret_ids
        .iter()
        .map(|secret_id| context.client_unchecked().get_secret_status(secret_id)),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("secret id"), context);
    formatter.push_target_ids_and_values(non_system_secret_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretListSystem {}

#[async_trait]
impl CommandExecutor for SecretListSystem {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all system secret ids");
    let start_instant = Instant::now();
    let system_secret_ids = context
      .client_unchecked()
      .get_secret_ids()
      .await?
      .into_iter()
      .filter(|id| secret::is_system_secret(id))
      .collect::<Vec<_>>();
    let allocation_statuses = try_join_all(system_secret_ids.iter().map(|secret_id| context.client_unchecked().get_secret_status(secret_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("system secret id"), context);
    formatter.push_target_ids_and_values(system_secret_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretListIds {}

#[async_trait]
impl CommandExecutor for SecretListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all secret ids");
    let start_instant = Instant::now();
    let non_system_secrets = context
      .client_unchecked()
      .get_secret_ids()
      .await?
      .into_iter()
      .filter(|id| !secret::is_system_secret(id))
      .collect::<Vec<_>>();
    context.print_execution_time(start_instant);
    let header = format!("secret ids ({})", non_system_secrets.len());
    let mut formatter = IdsFormatter::new(&header, context);
    formatter.push_target_ids(non_system_secrets.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(Some(OutputFormat::Plain))
  }
}

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all secrets that are used in apps or services");
    let start_instant = Instant::now();
    let secrets_with_usage: Vec<(String, Vec<UsedBy>)> = context.client_unchecked().list_secrets_with_usage().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("secret id"), context);
    for (secret_id, used_bys) in &secrets_with_usage {
      for used_by in used_bys {
        formatter.push_target_id_value(secret_id.clone(), used_by);
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no secrets found in apps or services");
    } else {
      formatter.print()?;
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretNew {}

#[async_trait]
impl CommandExecutor for SecretNew {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.client_unchecked().get_secret(&secret_id).await.is_ok() {
      return Err(format!("secret '{}' already exists", secret_id));
    }
    if context.stdin_is_terminal {
      if matches.get_flag(ModifierFlagType::MultiLine.id()) {
        context.print_explanation(format!("create new multi-line secret '{}'", secret_id));
        let secret = context.read_multi_line("enter multi-line secret (terminate input with ctrl-d after last line)")?;
        let secret = Secret { name: secret_id.clone(), value: secret };
        if context.dry_run {
          context.print_warning("dry-run mode, secret not created");
        } else {
          context.client_unchecked().post_secret(&secret).await?;
          context.print_outcome(format!("secret '{}' created", secret_id));
        }
      } else {
        context.print_explanation(format!("create new single line secret '{}'", secret_id));
        let secret = context.read_single_line_password("enter secret: ")?;
        let secret = Secret { name: secret_id.clone(), value: secret };
        if context.dry_run {
          context.print_warning("dry-run mode, secret not created");
        } else {
          context.client_unchecked().post_secret(&secret).await?;
          context.print_outcome(format!("secret '{}' created", secret_id));
        }
      }
    } else {
      let secret = context.read_multi_line("")?;
      let secret = Secret { name: secret_id.clone(), value: secret };
      if context.dry_run {
        context.print_warning("dry-run mode, secret not created");
      } else {
        context.client_unchecked().post_secret(&secret).await?;
        context.print_outcome(format!("secret '{}' created", secret_id));
      }
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for secret '{}'", secret_id));
    let start_instant = Instant::now();
    let allocation_status = context.client_unchecked().get_secret_status(&secret_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(secret_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("secret id"), context).print(&allocation_status)
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the apps and services that use secret '{}'", secret_id));
    let start_instant = Instant::now();
    let usages = context.client_unchecked().get_secret_with_usage(&secret_id).await?;
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

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct SecretShowValue {}

#[async_trait]
impl CommandExecutor for SecretShowValue {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the value of secret '{}'", secret_id));
    let start_instant = Instant::now();
    let secret = context.client_unchecked().get_secret(&secret_id).await?;
    context.print_execution_time(start_instant);
    context.print(secret);
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(Some(OutputFormat::Plain))
  }
}

struct SecretUpdate {}

#[async_trait]
impl CommandExecutor for SecretUpdate {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.client_unchecked().get_secret(&secret_id).await.is_err() {
      return Err(format!("secret '{}' does not exist", secret_id));
    }
    if context.stdin_is_terminal {
      if matches.get_flag(ModifierFlagType::MultiLine.id()) {
        context.print_explanation(format!("update multi-line secret '{}'", secret_id));
        let secret = context.read_multi_line("enter multi-line secret (terminate input with ctrl-d after last line)")?;
        if context.dry_run {
          context.print_warning("dry-run mode, secret not updated");
        } else {
          context.client_unchecked().put_secret(&secret_id, secret).await?;
          context.print_outcome(format!("secret '{}' updated", secret_id));
        }
      } else {
        context.print_explanation(format!("update single line secret '{}'", secret_id));
        let secret = context.read_single_line_password("enter secret: ")?;
        if context.dry_run {
          context.print_warning("dry-run mode, secret not updated");
        } else {
          context.client_unchecked().put_secret(&secret_id, secret).await?;
          context.print_outcome(format!("secret '{}' updated", secret_id));
        }
      }
    } else {
      let secret = context.read_multi_line("")?;
      if context.dry_run {
        context.print_warning("dry-run mode, secret not updated");
      } else {
        context.client_unchecked().put_secret(&secret_id, secret).await?;
        context.print_outcome(format!("secret '{}' updated", secret_id));
      }
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}
