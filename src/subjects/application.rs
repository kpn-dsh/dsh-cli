use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::subjects::image::parse_image_string;
use async_trait::async_trait;
use chrono::DateTime;
use clap::ArgMatches;
use dsh_api::types::Application;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

use dsh_api::types::{Task, TaskStatus};

use crate::arguments::target_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR, SHOW_COMMAND, SHOW_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::subjects::DEFAULT_ALLOCATION_STATUS_LABELS;
use crate::{include_started_stopped, DshCliResult};

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

  fn subject_command_about(&self) -> String {
    "Show, manage and list applications deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("a")
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(true, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(APPLICATION_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(APPLICATION_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &APPLICATION_CAPABILITIES
  }
}

lazy_static! {
  static ref APPLICATION_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List applications")
      .set_long_about(
        "Lists all deployed DSH applications. \
        This will also include applications that are stopped \
        (deployed with 0 instances)."
      )
      .set_default_command_executor(&ApplicationListAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ApplicationListAllocationStatus {}, None),
        (FlagType::Ids, &ApplicationListIds {}, None),
        (FlagType::Tasks, &ApplicationListTasks {}, None),
      ])
      .set_run_all_executors(true)
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List all started applications.".to_string())),
        (FilterFlagType::Stopped, Some("List all stopped applications.".to_string()))
      ])
  );
  static ref APPLICATION_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND_PAIR, "Show application configuration")
      .set_long_about("Show the configuration of an application deployed on the DSH.")
      .set_default_command_executor(&ApplicationShowAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ApplicationShowAllocationStatus {}, None),
        (FlagType::Tasks, &ApplicationShowTasks {}, None),
      ])
      .add_target_argument(target_argument(APPLICATION_SUBJECT_TARGET, None))
  );
  static ref APPLICATION_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![APPLICATION_LIST_CAPABILITY.as_ref(), APPLICATION_SHOW_CAPABILITY.as_ref()];
}

struct ApplicationListAll {}

#[async_trait]
impl CommandExecutor for ApplicationListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all applications with their parameters");
    let start_instant = Instant::now();
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    context.print_execution_time(start_instant);
    let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<_>>();
    application_ids.sort();
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut formatter = ListFormatter::new(&APPLICATION_LABELS_LIST, None, context);
    for application_id in application_ids {
      if let Some(application) = applications.get(&application_id) {
        if (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped) {
          formatter.push_target_id_value(application_id.clone(), application);
        }
      };
    }
    formatter.print()?;
    Ok(())
  }
}

struct ApplicationListAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all applications with their allocation status");
    let start_instant = Instant::now();
    let application_ids = context.dsh_api_client.as_ref().unwrap().list_application_ids().await?;
    let allocation_statuses = try_join_all(
      application_ids
        .iter()
        .map(|application_id| context.dsh_api_client.as_ref().unwrap().get_application_allocation_status(application_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("application id"), context);
    formatter.push_target_ids_and_values(application_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct ApplicationListIds {}

#[async_trait]
impl CommandExecutor for ApplicationListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all application ids");
    let start_instant = Instant::now();
    let ids = context.dsh_api_client.as_ref().unwrap().list_application_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("application id", context);
    formatter.push_target_ids(ids.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct ApplicationListTasks {}

#[async_trait]
impl CommandExecutor for ApplicationListTasks {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    fn tasks_to_string(tasks: Vec<String>) -> String {
      if tasks.len() <= 4 {
        tasks.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
      } else {
        format!(
          "{}, plus {} more",
          tasks.iter().take(4).map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
          tasks.len() - 4,
        )
      }
    }

    context.print_explanation("list all applications with their tasks");
    let start_instant = Instant::now();
    let application_ids = context.dsh_api_client.as_ref().unwrap().list_application_ids_with_derived_tasks().await?;
    let tasks: Vec<Vec<String>> = try_join_all(
      application_ids
        .iter()
        .map(|application_id| context.dsh_api_client.as_ref().unwrap().list_application_derived_task_ids(application_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let application_id_tasks_pairs: Vec<(String, String)> = application_ids
      .iter()
      .zip(tasks)
      .map(|(id, tasks)| (id.to_string(), tasks_to_string(tasks)))
      .collect::<Vec<_>>();
    let mut formatter: ListFormatter<ApplicationLabel, (String, String)> = ListFormatter::new(&[ApplicationLabel::Target, ApplicationLabel::Tasks], None, context);
    formatter.push_values(&application_id_tasks_pairs);
    formatter.print()?;
    Ok(())
  }
}

struct ApplicationShowAll {}

#[async_trait]
impl CommandExecutor for ApplicationShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for application '{}'", application_id));
    let start_instant = Instant::now();
    let application = context.dsh_api_client.as_ref().unwrap().get_application(application_id.as_str()).await?;
    context.print_execution_time(start_instant);
    // let table = ShowTable::new(application_id.as_str(), &application, &APPLICATION_LABELS_SHOW, context);
    // table.print();
    let formatter = UnitFormatter::new(application_id, &APPLICATION_LABELS_SHOW, Some("application id"), &application, context);
    formatter.print()?;
    Ok(())
  }
}

struct ApplicationShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for application '{}'", application_id));
    let start_instant = Instant::now();
    let allocation_status = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .get_application_allocation_status(application_id.as_str())
      .await?;
    context.print_execution_time(start_instant);
    let formatter = UnitFormatter::new(
      application_id,
      &DEFAULT_ALLOCATION_STATUS_LABELS,
      Some("application id"),
      &allocation_status,
      context,
    );
    formatter.print()?;
    Ok(())
  }
}

struct ApplicationShowTasks {}

#[async_trait]
impl CommandExecutor for ApplicationShowTasks {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all tasks for application '{}'", application_id));
    let start_instant = Instant::now();
    let task_ids = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .list_application_derived_task_ids(application_id.as_str())
      .await?;
    let task_statuses = try_join_all(task_ids.iter().map(|task_id| {
      context
        .dsh_api_client
        .as_ref()
        .unwrap()
        .get_application_task(application_id.as_str(), task_id.as_str())
    }))
    .await?;
    context.print_execution_time(start_instant);
    let mut tasks: Vec<(String, TaskStatus)> = task_ids.into_iter().zip(task_statuses).collect();
    tasks.sort_by(|first, second| second.1.actual.clone().unwrap().staged_at.cmp(&first.1.actual.clone().unwrap().staged_at));
    let mut formatter = ListFormatter::new(&TASK_LABELS_LIST, Some("application id"), context);
    formatter.push_target_id_value_pairs(tasks.as_slice());
    formatter.print()?;
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ApplicationLabel {
  Cpus,
  Env,
  ExposedPorts,
  HealthCheck,
  Image,
  Instances,
  Mem,
  Metrics,
  NeedsToken,
  ReadableStreams,
  Secrets,
  SingleInstance,
  SpreadGroup,
  Target,
  Tasks,
  Topics,
  User,
  Volumes,
  WritableStreams,
}

impl Label for ApplicationLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Target => "application id",
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "exposed ports",
      Self::HealthCheck => "health check",
      Self::Image => "image",
      Self::Instances => "instances",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "needs token",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single instance",
      Self::SpreadGroup => "spread group",
      Self::Tasks => "tasks",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "ports",
      Self::HealthCheck => "health",
      Self::Image => "image",
      Self::Instances => "#",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "token",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single",
      Self::SpreadGroup => "spread group",
      Self::Target => "application id",
      Self::Tasks => "tasks",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<ApplicationLabel> for Application {
  fn value(&self, label: &ApplicationLabel, application_id: &str) -> String {
    match label {
      ApplicationLabel::Cpus => self.cpus.to_string(),
      ApplicationLabel::Env => {
        let mut key = self.env.keys().map(|k| k.to_string()).collect::<Vec<_>>();
        key.sort();
        key
          .iter()
          .map(|key| format!("{} -> {}", key, self.env.get(key).unwrap()))
          .collect::<Vec<_>>()
          .join("\n")
      }
      ApplicationLabel::ExposedPorts => self.exposed_ports.keys().map(|port| port.to_string()).collect::<Vec<_>>().join(","),
      ApplicationLabel::HealthCheck => match self.health_check {
        Some(ref health_check) => match health_check.protocol {
          Some(protocol) => format!("{}:{}/{}", protocol.to_string(), health_check.port, health_check.path),
          None => format!("{}/{}", health_check.port, health_check.path),
        },
        None => "".to_string(),
      },
      ApplicationLabel::Image => match parse_image_string(self.image.as_str()) {
        Ok((kind, image)) => format!("{}:{}", kind, image),
        Err(_) => self.image.clone(),
      },
      ApplicationLabel::Instances => self.instances.to_string(),
      ApplicationLabel::Mem => self.mem.to_string(),
      ApplicationLabel::Metrics => self
        .metrics
        .clone()
        .map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))
        .unwrap_or_default(),
      ApplicationLabel::NeedsToken => self.needs_token.to_string(),
      ApplicationLabel::ReadableStreams => self
        .readable_streams
        .clone()
        .into_iter()
        .map(|readable_stream| readable_stream.to_string())
        .collect::<Vec<_>>()
        .join(", "),
      ApplicationLabel::Secrets => self.secrets.clone().into_iter().map(|secret| secret.name).collect::<Vec<_>>().join(", "),
      ApplicationLabel::SingleInstance => self.single_instance.to_string(),
      ApplicationLabel::SpreadGroup => self.spread_group.clone().unwrap_or_default(),
      ApplicationLabel::Target => application_id.to_string(),
      ApplicationLabel::Tasks => "".to_string(),
      ApplicationLabel::Topics => self.topics.clone().into_iter().map(|topic| topic.to_string()).collect::<Vec<_>>().join(", "),
      ApplicationLabel::User => self.user.clone(),
      ApplicationLabel::Volumes => self.volumes.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(","),
      ApplicationLabel::WritableStreams => self
        .writable_streams
        .clone()
        .into_iter()
        .map(|writable_stream| writable_stream.to_string())
        .collect::<Vec<_>>()
        .join(", "),
    }
  }

  fn target_label(&self) -> Option<ApplicationLabel> {
    Some(ApplicationLabel::Target)
  }
}

pub static APPLICATION_LABELS_LIST: [ApplicationLabel; 8] = [
  ApplicationLabel::Target,
  ApplicationLabel::NeedsToken,
  ApplicationLabel::Instances,
  ApplicationLabel::Cpus,
  ApplicationLabel::Mem,
  ApplicationLabel::ExposedPorts,
  ApplicationLabel::Metrics,
  ApplicationLabel::Image,
];

pub static APPLICATION_LABELS_SHOW: [ApplicationLabel; 18] = [
  ApplicationLabel::Target,
  ApplicationLabel::NeedsToken,
  ApplicationLabel::Instances,
  ApplicationLabel::Cpus,
  ApplicationLabel::Mem,
  ApplicationLabel::ExposedPorts,
  ApplicationLabel::Volumes,
  ApplicationLabel::Metrics,
  ApplicationLabel::Image,
  ApplicationLabel::HealthCheck,
  ApplicationLabel::ReadableStreams,
  ApplicationLabel::WritableStreams,
  ApplicationLabel::Secrets,
  ApplicationLabel::SingleInstance,
  ApplicationLabel::SpreadGroup,
  ApplicationLabel::Topics,
  ApplicationLabel::User,
  ApplicationLabel::Env,
];

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum TaskLabel {
  Healthy,
  HostIpAddress,
  _LastestLog,
  LastUpdateAt,
  StagedAt,
  StartedAt,
  State,
  StoppedAt,
  Target,
}

impl Label for TaskLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Healthy => "healthy",
      Self::HostIpAddress => "host ip address",
      Self::_LastestLog => "latest log",
      Self::LastUpdateAt => "last update",
      Self::StagedAt => "staged",
      Self::StartedAt => "started",
      Self::State => "state",
      Self::StoppedAt => "stopped",
      Self::Target => "task id",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::Healthy => "healthy",
      Self::HostIpAddress => "host",
      Self::_LastestLog => "log",
      Self::LastUpdateAt => "update",
      Self::StagedAt => "staged",
      Self::StartedAt => "started",
      Self::State => "state",
      Self::StoppedAt => "stopped",
      Self::Target => "task id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<TaskLabel> for TaskStatus {
  fn value(&self, label: &TaskLabel, task_id: &str) -> String {
    let task: Option<Task> = match self.actual.clone() {
      Some(actual) => Some(actual),
      None => self.configuration.clone(),
    };
    match task {
      Some(task) => match label {
        TaskLabel::Healthy => task.healthy.map(|healthy| healthy.to_string()).unwrap_or_default(),
        TaskLabel::HostIpAddress => task.host.to_string(),
        TaskLabel::_LastestLog => task.logs.map(|log| log.to_string()).unwrap_or_default(),
        TaskLabel::LastUpdateAt => task
          .last_update
          .and_then(|update| DateTime::from_timestamp_millis(update).map(|ts| ts.to_string()))
          .unwrap_or_default(),
        TaskLabel::StagedAt => task.staged_at.to_string(),
        TaskLabel::StartedAt => task.started_at.to_string(),
        TaskLabel::State => task.state.to_string(),
        TaskLabel::StoppedAt => task.stopped_at.map(|update| update.to_string()).unwrap_or_default(),
        TaskLabel::Target => task_id.to_string(),
      },
      None => "".to_string(),
    }
  }

  fn target_label(&self) -> Option<TaskLabel> {
    Some(TaskLabel::Target)
  }
}

pub static TASK_LABELS_LIST: [TaskLabel; 8] =
  [TaskLabel::StartedAt, TaskLabel::State, TaskLabel::Healthy, TaskLabel::Target, TaskLabel::HostIpAddress, TaskLabel::LastUpdateAt, TaskLabel::StagedAt, TaskLabel::StoppedAt];
