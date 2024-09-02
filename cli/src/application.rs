use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::dsh_api_client::DshApiClient;
use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiResult;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table, allocation_status_to_table_row};
use crate::formatters::application::{application_to_default_vector, default_application_column_labels, default_application_table};
use crate::subject::Subject;
use crate::tabular::{make_tabular, make_tabular_with_headers, print_table, print_tabular};
use crate::{to_command_error_with_id, CommandResult};

pub(crate) struct ApplicationSubject {}

const APPLICATION_SUBJECT_TARGET: &str = "application";

lazy_static! {
  pub static ref APPLICATION_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ApplicationSubject {});
}

#[async_trait]
impl Subject for ApplicationSubject {
  fn subject(&self) -> &'static str {
    APPLICATION_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Application"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH applications/services.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list applications/services deployed on the DSH.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("a")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, APPLICATION_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, APPLICATION_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref APPLICATION_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List applications".to_string(),
    command_long_about: Some("Lists all available DSH applications/services.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &ApplicationListAll {}, None),
      (FlagType::AllocationStatus, &ApplicationListAllocationStatus {}, None),
      (FlagType::Configuration, &ApplicationListConfiguration {}, None),
      (FlagType::Ids, &ApplicationListIds {}, None),
      (FlagType::Tasks, &ApplicationListTasks {}, None),
    ],
    default_command_executor: Some(&ApplicationListIds {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref APPLICATION_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show application configuration".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &ApplicationShowAll {}, None),
      (FlagType::AllocationStatus, &ApplicationShowAllocationStatus {}, None),
      (FlagType::Configuration, &ApplicationShowConfiguration {}, None),
      (FlagType::Tasks, &ApplicationShowTasks {}, None),
    ],
    default_command_executor: Some(&ApplicationShowConfiguration {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct ApplicationListAll {}

#[async_trait]
impl CommandExecutor for ApplicationListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    print_applications(&dsh_api_client.get_application_actual_configurations().await?)
  }
}

struct ApplicationListAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_ids = dsh_api_client.get_application_ids().await?;
    let allocation_statuses = futures::future::join_all(
      application_ids
        .iter()
        .map(|application_id| dsh_api_client.get_application_allocation_status(application_id.as_str())),
    )
    .await;
    let mut table = vec![];
    for (id, allocation_status) in application_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(APPLICATION_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct ApplicationListConfiguration {}

#[async_trait]
impl CommandExecutor for ApplicationListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    print_applications(&dsh_api_client.get_application_configurations().await?)
  }
}

struct ApplicationListIds {}

#[async_trait]
impl CommandExecutor for ApplicationListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_ids = dsh_api_client.get_application_ids().await?;
    for application_id in application_ids {
      println!("{}", application_id)
    }
    Ok(())
  }
}

struct ApplicationListTasks {}

#[async_trait]
impl CommandExecutor for ApplicationListTasks {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_ids = dsh_api_client.get_application_ids_with_derived_tasks().await?;
    let tasks: Vec<DshApiResult<Vec<String>>> = futures::future::join_all(
      application_ids
        .iter()
        .map(|application_id| dsh_api_client.get_application_derived_task_ids(application_id.as_str())),
    )
    .await;
    let mut table = vec![];
    for (application_id, tasks) in application_ids.iter().zip(tasks) {
      if let Ok(mut ts) = tasks {
        if !ts.is_empty() {
          ts.sort();
          let vector = vec![
            application_id.to_string(),
            if ts.len() <= 4 {
              ts.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
            } else {
              format!(
                "{}, plus {} more",
                ts.iter().take(4).map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
                ts.len() - 4,
              )
            },
          ];

          table.push(vector);
        }
      }
    }
    for line in make_tabular_with_headers(&["application", "tasks"], table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct ApplicationShowAll {}

#[async_trait]
impl CommandExecutor for ApplicationShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    match dsh_api_client.get_application_actual_configuration(application_id.as_str()).await {
      Ok(application) => {
        let table = default_application_table(application_id.as_str(), &application);
        let tabular = make_tabular(table, "", "  ", "");
        print_tabular("", &tabular);
        Ok(())
      }
      Err(error) => to_command_error_with_id(error, APPLICATION_SUBJECT_TARGET, application_id.as_str()),
    }
  }
}

struct ApplicationShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    let allocation_status = dsh_api_client.get_application_allocation_status(application_id.as_str()).await?;
    let table = allocation_status_to_table(APPLICATION_SUBJECT_TARGET, application_id.as_str(), &allocation_status);
    print_table(table, "", "  ", "");
    Ok(())
  }
}

struct ApplicationShowConfiguration {}

#[async_trait]
impl CommandExecutor for ApplicationShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    let application = dsh_api_client.get_application_configuration(application_id.as_str()).await?;
    let table = default_application_table(application_id.as_str(), &application);
    let tabular = make_tabular(table, "", "  ", "");
    print_tabular("", &tabular);
    Ok(())
  }
}

struct ApplicationShowTasks {}

#[async_trait]
impl CommandExecutor for ApplicationShowTasks {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    let task_ids = dsh_api_client.get_application_derived_task_ids(application_id.as_str()).await?;
    let allocation_statuses = futures::future::join_all(
      task_ids
        .iter()
        .map(|task_id| dsh_api_client.get_application_task_allocation_status(application_id.as_str(), task_id.as_str())),
    )
    .await;
    let mut table = vec![];
    for (task_id, allocation_status) in task_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(task_id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(APPLICATION_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

fn print_applications(applications: &HashMap<String, Application>) -> CommandResult {
  let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
  application_ids.sort();
  let mut table: Vec<Vec<String>> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    table.push(application_to_default_vector(application_id.as_str(), application));
  }
  for line in make_tabular_with_headers(&default_application_column_labels(), table) {
    println!("{}", line)
  }
  Ok(())
}
