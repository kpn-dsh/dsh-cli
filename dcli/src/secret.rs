use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::Injection;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::list_table::ListTable;
use crate::formatters::usage::{Usage, UsageLabel, USAGE_IN_APPLICATIONS_LABELS_LIST, USAGE_IN_APPLICATIONS_LABELS_SHOW, USAGE_IN_APPS_LABELS_LIST, USAGE_IN_APPS_LABELS_SHOW};
use crate::modifier_flags::ModifierFlagType;
use crate::subject::Subject;
use crate::{confirmed, include_app_application, read_multi_line, read_single_line, DcliContext, DcliResult};
use dsh_api::types::{AppCatalogApp, Application, Secret};

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
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    if matches.get_flag(ModifierFlagType::MultiLine.id()) {
      let secret_id = target.unwrap_or_else(|| unreachable!());
      if context.show_capability_explanation() {
        println!("create new multi-line secret '{}'", secret_id);
      }
      if context.dsh_api_client.as_ref().unwrap().get_secret(&secret_id).await.is_ok() {
        return Err(format!("secret '{}' already exists", secret_id));
      }
      println!("enter multi-line secret (terminate input with ctrl-d after last line)");
      let value = read_multi_line()?;
      let secret = Secret { name: secret_id, value };
      context.dsh_api_client.as_ref().unwrap().create_secret(&secret).await?;
      println!("ok");
      Ok(true)
    } else {
      let secret_id = target.unwrap_or_else(|| unreachable!());
      if context.show_capability_explanation() {
        println!("create new single line secret '{}'", secret_id);
      }
      if context.dsh_api_client.as_ref().unwrap().get_secret(&secret_id).await.is_ok() {
        return Err(format!("secret '{}' already exists", secret_id));
      }
      let value = read_single_line("enter secret: ")?;
      let secret = Secret { name: secret_id, value };
      context.dsh_api_client.as_ref().unwrap().create_secret(&secret).await?;
      println!("ok");
      Ok(true)
    }
  }
}

struct SecretDelete {}

#[async_trait]
impl CommandExecutor for SecretDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("delete secret '{}'", secret_id);
    }
    if context.dsh_api_client.as_ref().unwrap().get_secret_configuration(&secret_id).await.is_err() {
      return Err(format!("secret '{}' does not exists", secret_id));
    }
    if confirmed(format!("type 'yes' to delete secret '{}': ", secret_id).as_str())? {
      context.dsh_api_client.as_ref().unwrap().delete_secret(&secret_id).await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all secrets with their allocation status");
    }
    let non_system_secret_ids = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_secret_ids()
      .await?
      .into_iter()
      .filter(|id| !is_system_secret(id))
      .collect::<Vec<_>>();
    let allocation_statusses = try_join_all(
      non_system_secret_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_secret_allocation_status(id.as_str())),
    )
    .await?;
    print_allocation_statuses(non_system_secret_ids, allocation_statusses, context);
    Ok(false)
  }
}

struct SecretListSystem {}

#[async_trait]
impl CommandExecutor for SecretListSystem {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all system secret ids");
    }
    let system_secret_ids = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_secret_ids()
      .await?
      .into_iter()
      .filter(|id| is_system_secret(id))
      .collect::<Vec<_>>();
    let allocation_statusses = try_join_all(
      system_secret_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_secret_allocation_status(id.as_str())),
    )
    .await?;
    print_allocation_statuses(system_secret_ids, allocation_statusses, context);
    Ok(false)
  }
}

struct SecretListIds {}

#[async_trait]
impl CommandExecutor for SecretListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all secret ids");
    }
    let non_system_secrets = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_secret_ids()
      .await?
      .into_iter()
      .filter(|id| !is_system_secret(id))
      .collect::<Vec<_>>();
    print_vec("secret ids".to_string(), non_system_secrets, context);
    Ok(false)
  }
}

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    let (include_app, include_application) = include_app_application(matches);
    if include_app {
      if context.show_capability_explanation() {
        println!("list all secrets with their usage in apps");
      }
      let (secret_ids, apps) = try_join!(
        context.dsh_api_client.as_ref().unwrap().list_secret_ids(),
        context.dsh_api_client.as_ref().unwrap().get_app_configurations()
      )?;
      let mut table = ListTable::new(&USAGE_IN_APPS_LABELS_LIST, context);
      for secret_id in &secret_ids {
        let app_usages: Vec<(String, &AppCatalogApp, String, &Application, HashMap<String, Vec<Injection>>)> =
          DshApiClient::apps_with_secrets_injections(&[secret_id.to_string()], &apps);
        let mut first = true;
        for (app_id, _, _, application, secret_injections) in app_usages {
          let injections = secret_injections
            .values()
            .map(|envs| envs.iter().map(|env| env.to_string()).collect::<Vec<_>>().join(", "))
            .collect::<Vec<_>>();
          if first {
            table.row(&Usage::app(secret_id.to_string(), app_id.to_string(), application.instances, injections));
          } else {
            table.row(&Usage::app("".to_string(), app_id.to_string(), application.instances, injections));
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
      let (secret_ids, applications) = try_join!(
        context.dsh_api_client.as_ref().unwrap().list_secret_ids(),
        context.dsh_api_client.as_ref().unwrap().get_applications()
      )?;
      let mut table = ListTable::new(&USAGE_IN_APPLICATIONS_LABELS_LIST, context);
      for secret_id in &secret_ids {
        let mut first = true;
        let application_usages: Vec<(String, &Application, HashMap<String, Vec<Injection>>)> =
          DshApiClient::applications_with_secrets_injections(&[secret_id.to_string()], &applications);
        for (application_id, application, secret_injections) in &application_usages {
          let injections = secret_injections
            .values()
            .map(|envs| envs.iter().map(|env| env.to_string()).collect::<Vec<_>>().join(", "))
            .collect::<Vec<_>>();
          if first {
            table.row(&Usage::application(
              secret_id.to_string(),
              application_id.to_string(),
              application.instances,
              injections,
            ));
          } else {
            table.row(&Usage::application("".to_string(), application_id.to_string(), application.instances, injections));
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
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show allocation status for secret '{}'", secret_id);
    }
    let allocation_status = context.dsh_api_client.as_ref().unwrap().get_secret_allocation_status(secret_id.as_str()).await?;
    print_allocation_status(secret_id, allocation_status, context);
    Ok(false)
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show applications that use secret '{}'", secret_id);
    }
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    let application_injections: Vec<(String, &Application, HashMap<String, Vec<Injection>>)> =
      DshApiClient::applications_with_secrets_injections(&[secret_id.clone()], &applications);
    if !application_injections.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_IN_APPLICATIONS_LABELS_SHOW, context);
      for (application_id, application, secret_injections) in application_injections {
        let injections = secret_injections
          .values()
          .map(|envs| envs.iter().map(|env| env.to_string()).collect::<Vec<_>>().join("\n"))
          .collect();
        builder.row(&Usage::application(secret_id.clone(), application_id, application.instances, injections));
      }
      builder.print();
    } else {
      println!("secret not used in applications")
    }
    let apps = context.dsh_api_client.as_ref().unwrap().get_app_configurations().await?;
    let app_injections: Vec<(String, &AppCatalogApp, String, &Application, HashMap<String, Vec<Injection>>)> =
      DshApiClient::apps_with_secrets_injections(&[secret_id.clone()], &apps);
    if !app_injections.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_IN_APPS_LABELS_SHOW, context);
      for (app_id, _, _, application, secret_injections) in app_injections {
        let injections = secret_injections
          .iter()
          .map(|(secret, envs)| format!("{}:{}", secret, envs.iter().map(|env| env.to_string()).collect::<Vec<_>>().join(", ")))
          .collect();
        builder.row(&Usage::app(secret_id.clone(), app_id, application.instances, injections));
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
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the value of secret '{}'", secret_id);
    }
    let secret = context.dsh_api_client.as_ref().unwrap().get_secret(secret_id.as_str()).await?;
    println!("{}", secret);
    Ok(false)
  }
}

fn is_system_secret(secret_id: &str) -> bool {
  secret_id.contains('!')
}
