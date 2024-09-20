use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use lazy_static::lazy_static;

use dsh_api::application::application_diff;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::Application;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses, ALLOCATION_STATUS_LABELS};
use crate::formatters::application::{ApplicationLabel, APPLICATION_LABELS_LIST, APPLICATION_LABELS_SHOW};
use crate::formatters::formatter::{print_ids, TableBuilder};
use crate::subject::Subject;
use crate::{to_command_error_with_id, DcliContext, DcliResult};

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
    capabilities.insert(CapabilityType::Diff, APPLICATION_DIFF_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, APPLICATION_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, APPLICATION_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref APPLICATION_DIFF_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Diff,
    command_about: "Diff applications".to_string(),
    command_long_about: Some("Compare the deployed configuration of the application against the actual configuration.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::All, &ApplicationDiffAll {}, None),],
    default_command_executor: Some(&ApplicationDiffAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
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

struct ApplicationDiffAll {}

#[async_trait]
impl CommandExecutor for ApplicationDiffAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show difference between configuration and actual status for application '{}'", application_id);
    }
    let deployed = dsh_api_client.get_application_configuration(application_id.as_str()).await?;
    let actual = dsh_api_client.get_application_actual_configuration(application_id.as_str()).await?;
    let diff = application_diff(&deployed, &actual);
    if diff.is_empty() {
      println!("equal")
    } else {
      println!("{:#?}", diff)
    }
    Ok(false)
  }
}

struct ApplicationListAll {}

#[async_trait]
impl CommandExecutor for ApplicationListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all applications with their parameters");
    }
    print_applications(&dsh_api_client.get_application_actual_configurations().await?, context)
  }
}

struct ApplicationListAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all applications with their allocation status");
    }
    let application_ids = dsh_api_client.get_application_ids().await?;
    let allocation_statuses = try_join_all(
      application_ids
        .iter()
        .map(|application_id| dsh_api_client.get_application_allocation_status(application_id.as_str())),
    )
    .await?;
    print_allocation_statuses(application_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct ApplicationListConfiguration {}

#[async_trait]
impl CommandExecutor for ApplicationListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all applications with their configuration");
    }
    print_applications(&dsh_api_client.get_application_configurations().await?, context)
  }
}

struct ApplicationListIds {}

#[async_trait]
impl CommandExecutor for ApplicationListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all application ids");
    }
    print_ids("application ids".to_string(), dsh_api_client.get_application_ids().await?, context);
    Ok(false)
  }
}

struct ApplicationListTasks {}

#[async_trait]
impl CommandExecutor for ApplicationListTasks {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all applications with their tasks");
    }
    let application_ids = dsh_api_client.get_application_ids_with_derived_tasks().await?;
    let tasks: Vec<Vec<String>> = try_join_all(
      application_ids
        .iter()
        .map(|application_id| dsh_api_client.get_application_derived_task_ids(application_id.as_str())),
    )
    .await?;
    let mut builder: TableBuilder<ApplicationLabel, HashMap<&ApplicationLabel, String>> = TableBuilder::list(&[ApplicationLabel::Target, ApplicationLabel::Tasks], context);
    for (application_id, mut tasks) in application_ids.iter().zip(tasks) {
      if !tasks.is_empty() {
        tasks.sort();
        let mut map = HashMap::<&ApplicationLabel, String>::new();
        map.insert(&ApplicationLabel::Target, application_id.to_string());
        map.insert(
          &ApplicationLabel::Tasks,
          if tasks.len() <= 4 {
            tasks.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
          } else {
            format!(
              "{}, plus {} more",
              tasks.iter().take(4).map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
              tasks.len() - 4,
            )
          },
        );
        builder.value("".to_string(), &map);
      }
    }
    builder.print();
    Ok(false)
  }
}

struct ApplicationShowAll {}

#[async_trait]
impl CommandExecutor for ApplicationShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for application '{}'", application_id);
    }
    match dsh_api_client.get_application_actual_configuration(application_id.as_str()).await {
      Ok(application) => {
        let mut builder = TableBuilder::show(&APPLICATION_LABELS_SHOW, context);
        builder.value(application_id, &application);
        builder.print();
        Ok(false)
      }
      Err(error) => to_command_error_with_id(error, APPLICATION_SUBJECT_TARGET, application_id.as_str()),
    }
  }
}

struct ApplicationShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show allocation status for application '{}'", application_id);
    }
    let allocation_status = dsh_api_client.get_application_allocation_status(application_id.as_str()).await?;
    print_allocation_status(application_id, allocation_status, context);
    Ok(false)
  }
}

struct ApplicationShowConfiguration {}

#[async_trait]
impl CommandExecutor for ApplicationShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show configuration for application '{}'", application_id);
    }
    let application = dsh_api_client.get_application_configuration(application_id.as_str()).await?;
    let mut builder = TableBuilder::show(&APPLICATION_LABELS_SHOW, context);
    builder.value(application_id, &application);
    Ok(false)
  }
}

struct ApplicationShowTasks {}

#[async_trait]
impl CommandExecutor for ApplicationShowTasks {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all tasks for application '{}'", application_id);
    }
    let task_ids = dsh_api_client.get_application_derived_task_ids(application_id.as_str()).await?;
    let allocation_statuses = try_join_all(
      task_ids
        .iter()
        .map(|task_id| dsh_api_client.get_application_task_allocation_status(application_id.as_str(), task_id.as_str())),
    )
    .await?;
    let mut builder = TableBuilder::show(&ALLOCATION_STATUS_LABELS, context);
    for (task_id, allocation_status) in task_ids.iter().zip(allocation_statuses) {
      builder.value(task_id.to_string(), &allocation_status);
    }
    builder.print();
    Ok(false)
  }
}

fn print_applications(applications: &HashMap<String, Application>, context: &DcliContext) -> DcliResult {
  let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
  application_ids.sort();
  let mut builder = TableBuilder::list(&APPLICATION_LABELS_LIST, context);
  for application_id in application_ids {
    builder.value(application_id.clone(), applications.get(&application_id).unwrap());
  }
  builder.print();
  Ok(false)
}

pub(crate) fn _applications_that_use_value(value: &str, applications: &HashMap<String, Application>) -> Vec<(String, Vec<String>)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, Vec<String>)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.env.is_empty() {
      let envs_that_contain_certificate_id: Vec<String> = application.env.clone().into_iter().filter(|(_, v)| v.contains(value)).map(|(k, _)| k).collect();
      pairs.push((application_id.clone(), envs_that_contain_certificate_id));
    }
  }
  pairs
}

// Returns vector with pairs (application_id, secret -> environment variables)
pub(crate) fn applications_with_secret_injections(secrets: &[String], applications: &HashMap<String, Application>) -> Vec<(String, HashMap<String, Vec<String>>)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, HashMap<String, Vec<String>>)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.secrets.is_empty() {
      let mut injections = HashMap::<String, Vec<String>>::new();
      for application_secret in &application.secrets {
        if secrets.contains(&application_secret.name) {
          let mut env_injections = vec![];
          for application_secret_injection in &application_secret.injections {
            if let Some(env_injection) = application_secret_injection.get("env") {
              env_injections.push(env_injection.to_string());
            }
          }
          if !env_injections.is_empty() {
            injections.insert(application_secret.name.clone(), env_injections);
          }
        }
      }
      if !injections.is_empty() {
        pairs.push((application_id.clone(), injections));
      }
    }
  }
  pairs
}
