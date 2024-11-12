use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::Secret;

use crate::{confirmed, DcliContext, DcliResult, include_app_application, read_multi_line, read_single_line};
use crate::app::apps_with_secret_injections;
use crate::application::applications_with_secret_injections;
use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::list_table::ListTable;
use crate::formatters::usage::{Usage, USAGE_IN_APPLICATIONS_LABELS_LIST, USAGE_IN_APPLICATIONS_LABELS_SHOW, USAGE_IN_APPS_LABELS_LIST, USAGE_IN_APPS_LABELS_SHOW, UsageLabel};
use crate::modifier_flags::ModifierFlagType;
use crate::subject::Subject;

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

  fn subject_first_upper(&self) -> &'static str {
    "Secret"
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
  pub static ref SECRET_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Create,
    command_about: "Create secret".to_string(),
    command_long_about: Some("Create a secret.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SecretCreate {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![(ModifierFlagType::MultiLine, None)],
  });
  pub static ref SECRET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Delete,
    command_about: "Delete secret".to_string(),
    command_long_about: Some("Delete a secret.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&SecretDelete {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref SECRET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List secrets".to_string(),
    command_long_about: Some("Lists all secrets used by the applications/services and apps on the DSH.".to_string()),
    command_executors: vec![
      (FlagType::All, &SecretListIds {}, None),
      (FlagType::AllocationStatus, &SecretListAllocationStatus {}, None),
      (FlagType::Ids, &SecretListIds {}, None),
      (FlagType::Usage, &SecretListUsage {}, None),
    ],
    default_command_executor: Some(&SecretListIds {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![(FilterFlagType::App, Some("List all apps that use the secret.")), (FilterFlagType::Application, Some("List all applications that use the secret."))],
    modifier_flags: vec![],
  });
  pub static ref SECRET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show secret configuration or value".to_string(),
    command_long_about: None,
    command_executors: vec![
      (FlagType::AllocationStatus, &SecretShowAllocationStatus {}, None),
      (FlagType::Usage, &SecretShowUsage {}, None),
      (FlagType::Value, &SecretShowValue {}, None),
    ],
    default_command_executor: Some(&SecretShowAllocationStatus {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
}

struct SecretCreate {}

#[async_trait]
impl CommandExecutor for SecretCreate {
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

struct SecretDelete {}

#[async_trait]
impl CommandExecutor for SecretDelete {
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

struct SecretListAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all secrets with their allocation status");
    }
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    let allocation_statusses = try_join_all(secret_ids.iter().map(|id| dsh_api_client.get_secret_allocation_status(id.as_str()))).await?;
    print_allocation_statuses(secret_ids, allocation_statusses, context);
    Ok(false)
  }
}

struct SecretListIds {}

#[async_trait]
impl CommandExecutor for SecretListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all secret ids");
    }
    print_vec("secret ids".to_string(), dsh_api_client.get_secret_ids().await?, context);
    Ok(false)
  }
}

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let (include_app, include_application) = include_app_application(matches);
    if include_app {
      if context.show_capability_explanation() {
        println!("list all secrets with their usage in apps");
      }
      let (secret_ids, apps) = try_join!(dsh_api_client.get_secret_ids(), dsh_api_client.get_app_configurations())?;
      let mut table = ListTable::new(&USAGE_IN_APPS_LABELS_LIST, context);
      for secret_id in &secret_ids {
        let app_usages: Vec<(String, u64, HashMap<String, Vec<String>>)> = apps_with_secret_injections(&[secret_id.to_string()], &apps);
        let mut first = true;
        for (app_id, instances, secret_injections) in &app_usages {
          let injections = secret_injections.values().map(|envs| envs.join(", ")).collect::<Vec<String>>();
          if first {
            table.row(&Usage::app(secret_id.to_string(), app_id.to_string(), *instances, injections));
          } else {
            table.row(&Usage::app("".to_string(), app_id.to_string(), *instances, injections));
          }
          first = false;
        }
      }
      if table.is_empty() {
        println!("no secrets found in apps");
      } else {
        table.print();
      }
    }
    if include_application {
      if context.show_capability_explanation() {
        println!("list all secrets with their usage in applications");
      }
      let (secret_ids, applications) = try_join!(dsh_api_client.get_secret_ids(), dsh_api_client.get_applications())?;
      let mut table = ListTable::new(&USAGE_IN_APPLICATIONS_LABELS_LIST, context);
      for secret_id in &secret_ids {
        let mut first = true;
        let application_usages: Vec<(String, u64, HashMap<String, Vec<String>>)> = applications_with_secret_injections(&[secret_id.to_string()], &applications);
        for (application_id, instances, secret_injections) in &application_usages {
          let injections = secret_injections.values().map(|envs| envs.join(", ")).collect::<Vec<String>>();
          if first {
            table.row(&Usage::application(secret_id.to_string(), application_id.to_string(), *instances, injections));
          } else {
            table.row(&Usage::application("".to_string(), application_id.to_string(), *instances, injections));
          }
          first = false;
        }
      }
      if table.is_empty() {
        println!("no secrets found in applications");
      } else {
        table.print();
      }
    }
    Ok(false)
  }
}

struct SecretShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show allocation status for secret '{}'", secret_id);
    }
    let allocation_status = dsh_api_client.get_secret_allocation_status(secret_id.as_str()).await?;
    print_allocation_status(secret_id, allocation_status, context);
    Ok(false)
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show applications that use secret '{}'", secret_id);
    }
    let applications = dsh_api_client.get_applications().await?;
    let application_injections: Vec<(String, u64, HashMap<String, Vec<String>>)> = applications_with_secret_injections(&[secret_id.clone()], &applications);
    if !application_injections.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_IN_APPLICATIONS_LABELS_SHOW, context);
      for (application_id, instances, secret_injections) in application_injections {
        let injections = secret_injections.values().map(|envs| envs.join("\n")).collect();
        builder.row(&Usage::application(secret_id.clone(), application_id, instances, injections));
      }
      builder.print();
    } else {
      println!("secret not used in applications")
    }
    let apps = dsh_api_client.get_app_configurations().await?;
    let app_injections: Vec<(String, u64, HashMap<String, Vec<String>>)> = apps_with_secret_injections(&[secret_id.clone()], &apps);
    if !app_injections.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_IN_APPS_LABELS_SHOW, context);
      for (app_id, instances, secret_injections) in app_injections {
        let injections = secret_injections.iter().map(|(secret, envs)| format!("{}:{}", secret, envs.join(", "))).collect();
        builder.row(&Usage::app(secret_id.clone(), app_id, instances, injections));
      }
      builder.print();
    } else {
      println!("secret not used in apps")
    }

    Ok(false)
  }
}

struct SecretShowValue {}

#[async_trait]
impl CommandExecutor for SecretShowValue {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the value of secret '{}'", secret_id);
    }
    let secret = dsh_api_client.get_secret(secret_id.as_str()).await?;
    println!("{}", secret);
    Ok(false)
  }
}
