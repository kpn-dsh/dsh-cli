use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::dsh_api_client::DshApiClient;
use trifonius_dsh_api::types::Application;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table, allocation_status_to_table_row};
use crate::subject::Subject;
use crate::tabular::{make_tabular_with_headers, print_table};
use crate::CommandResult;

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

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("s")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, SECRET_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, SECRET_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref SECRET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List secrets".to_string(),
    command_long_about: Some("Lists all secrets used by the applications/services and apps on the DSH.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &SecretListIds {}, None),
      (FlagType::AllocationStatus, &SecretListAllocationStatus {}, None),
      (FlagType::Ids, &SecretListIds {}, None),
      (FlagType::Usage, &SecretListUsage {}, None),
    ],
    default_command_executor: Some(&SecretListIds {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref SECRET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show secret configuration or value".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::AllocationStatus, &SecretShowAllocationStatus {}, None),
      (FlagType::Usage, &SecretShowUsage {}, None),
      (FlagType::Value, &SecretShowValue {}, None),
    ],
    default_command_executor: None,
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct SecretListAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    let allocation_statusses = futures::future::join_all(secret_ids.iter().map(|id| dsh_api_client.get_secret_allocation_status(id.as_str()))).await;
    let mut table = vec![];
    for (secret_id, secret_status) in secret_ids.iter().zip(allocation_statusses) {
      table.push(allocation_status_to_table_row(secret_id, secret_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(SECRET_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct SecretListIds {}

#[async_trait]
impl CommandExecutor for SecretListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    for secret_id in secret_ids {
      println!("{}", secret_id)
    }
    Ok(())
  }
}

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = dsh_api_client.get_application_configurations().await?;
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for secret_id in &secret_ids {
      let mut first = true;
      let usages: Vec<(String, String)> = applications_that_use_secret(secret_id, &applications);
      for (application_id, envs) in usages {
        if first {
          table.push(vec![secret_id.clone(), application_id, envs])
        } else {
          table.push(vec!["".to_string(), application_id, envs])
        }
        first = false;
      }
    }
    for line in make_tabular_with_headers(&["secret", "application", "usage"], table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct SecretShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    let allocation_status = dsh_api_client.get_secret_allocation_status(secret_id.as_str()).await?;
    let table = allocation_status_to_table(SECRET_SUBJECT_TARGET, secret_id.as_str(), &allocation_status);
    print_table(table, "", "  ", "");
    Ok(())
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    let applications = dsh_api_client.get_application_configurations().await?;
    let usage = applications_that_use_secret(secret_id.as_str(), &applications);
    if !usage.is_empty() {
      let table: Vec<Vec<String>> = usage.iter().map(|(application_id, usage)| vec![application_id.clone(), usage.clone()]).collect();
      for line in make_tabular_with_headers(&["application", "usage"], table) {
        println!("{}", line)
      }
    } else {
      println!("secret not used")
    }
    Ok(())
  }
}

struct SecretShowValue {}

#[async_trait]
impl CommandExecutor for SecretShowValue {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    let secret = dsh_api_client.get_secret(secret_id.as_str()).await?;
    println!("{}", secret);
    Ok(())
  }
}

// Returns vector with pairs (application_id, environment variables)
fn applications_that_use_secret(secret_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, String)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, String)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.secrets.is_empty() {
      for application_secret in &application.secrets {
        if application_secret.name == secret_id {
          let mut envs: Vec<String> = application_secret
            .clone()
            .injections
            .into_iter()
            .filter_map(|injection| injection.get("env").cloned())
            .collect();
          if envs.len() == 1 {
            pairs.push((application_id.clone(), format!("env:{}", envs.first().unwrap())));
          }
          if envs.len() > 1 {
            envs.sort();
            let joined_envs: String = envs.join(",");
            pairs.push((application_id.clone(), format!("envs:{}", joined_envs)));
          }
        }
      }
    }
  }
  pairs
}
