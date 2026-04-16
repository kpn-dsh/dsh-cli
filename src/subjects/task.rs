use crate::arguments::{service_id_argument, task_id_argument, TASK_ID_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::Value;
use crate::formatters::{Label, SubjectFormatter};
use crate::monitoring::tenant_service_monitoring_url;
use crate::subject::{Requirements, Subject};
use crate::target_platform::get_target_platform;
use crate::target_tenant::get_target_tenant;
use crate::DshCliResult;
use async_trait::async_trait;
use chrono::DateTime;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{Task, TaskState, TaskStatus};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::sync::LazyLock;

struct TaskSubject {}

const TASK_SUBJECT_TARGET: &str = "task";

lazy_static! {
  pub(crate) static ref TASK_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TaskSubject {});
}

#[async_trait]
impl Subject for TaskSubject {
  fn subject(&self) -> &'static str {
    TASK_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "List and show DSH service tasks.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "List and show DSH service tasks.".to_string()
  }

  fn support_list_shortcut(&self) -> bool {
    false
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(TASK_LIST_CAPABILITY.as_ref()),
      OPEN_COMMAND => Some(TASK_OPEN_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(TASK_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TASK_CAPABILITIES
  }
}

static TASK_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TaskList {}, "List all tasks for a service").add_target_argument(service_id_argument().required(true)))
});

static TASK_OPEN_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(
      OPEN_COMMAND,
      Some(OPEN_COMMAND_ALIAS),
      &TaskOpen {},
      "Open the monitoring web application for the task",
    )
    .add_target_argument(service_id_argument().required(true))
    .add_target_argument(task_id_argument()),
  )
});

static TASK_SHOW_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &TaskShow {}, "Show the task details")
      .add_target_argument(service_id_argument().required(true))
      .add_target_argument(task_id_argument()),
  )
});

static TASK_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> =
  LazyLock::new(|| vec![TASK_LIST_CAPABILITY.as_ref(), TASK_OPEN_CAPABILITY.as_ref(), TASK_SHOW_CAPABILITY.as_ref()]);

struct TaskList {}

#[async_trait]
impl CommandExecutor for TaskList {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all tasks for service '{}'", service_id));
    let start_instant = context.now();
    let tasks: Vec<(String, TaskStatus)> = client.application_tasks(&service_id).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&TASK_LABELS_LIST, context);
    formatter.push_target_id_value_pairs(tasks.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TaskOpen {}

#[async_trait]
impl CommandExecutor for TaskOpen {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let service_id = target.unwrap_or_else(|| unreachable!());
    match matches.get_one::<String>(TASK_ID_ARGUMENT) {
      Some(task_id) => {
        let start_instant = context.now();
        let task_status = client.get_task(&service_id, task_id).await?;
        context.print_execution_time(start_instant);
        let url = tenant_service_monitoring_url(&platform, &tenant_name, &service_id, task_status.actual.unwrap().logs.unwrap());
        context.open_url(
          &url,
          format!("monitoring page for tenant '{}@{}' and service '{}'", tenant_name, platform, service_id),
        );
      }
      None => {
        let start_instant = context.now();
        let tasks = client.application_tasks(&service_id).await?;
        context.print_execution_time(start_instant);
        let active_tasks: Vec<(String, TaskStatus)> = tasks
          .into_iter()
          .filter(|(_, task_status)| {
            task_status
              .actual
              .as_ref()
              .is_some_and(|task| task.state == TaskState::Running || task.state == TaskState::Staging || task.state == TaskState::Starting)
          })
          .collect_vec();
        if active_tasks.is_empty() {
          context.print_outcome(format!("no active of starting tasks for service '{}'", service_id))
        } else if active_tasks.len() == 1 {
          let (_, task_status) = active_tasks.into_iter().next().unwrap_or_else(|| unreachable!());
          let url = tenant_service_monitoring_url(&platform, &tenant_name, &service_id, task_status.actual.unwrap().logs.unwrap());
          context.open_url(
            &url,
            format!("monitoring page for tenant '{}@{}' and service '{}'", tenant_name, platform, service_id),
          );
        } else {
          context.print_warning(format!(
            "more than one active or starting tasks found for service '{}', provide task id",
            service_id
          ));
          let mut formatter = ListFormatter::new(&TASK_LABELS_LIST, context);
          formatter.push_target_id_value_pairs(active_tasks.as_slice());
          formatter.print(None)?;
        }
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TaskShow {}

#[async_trait]
impl CommandExecutor for TaskShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    match matches.get_one::<String>(TASK_ID_ARGUMENT) {
      Some(task_id) => {
        context.print_explanation(format!("show task '{}' for service '{}'", task_id, service_id));
        let start_instant = context.now();
        let task_status = client.get_task(&service_id, &task_id).await?;
        context.print_execution_time(start_instant);
        let formatter = UnitFormatter::new(task_id, &TASK_LABELS_LIST, context);
        formatter.print(&task_status, None)?;
      }
      None => {
        context.print_explanation(format!("show all active or starting tasks for service '{}'", service_id));
        let start_instant = context.now();
        let tasks = client.application_tasks(&service_id).await?;
        context.print_execution_time(start_instant);
        let active_tasks: Vec<&(String, TaskStatus)> = tasks
          .iter()
          .filter(|(_, task_status)| {
            task_status
              .actual
              .as_ref()
              .is_some_and(|task| task.state == TaskState::Running || task.state == TaskState::Staging || task.state == TaskState::Starting)
          })
          .collect_vec();
        if active_tasks.is_empty() {
          context.print_outcome(format!("no active of starting tasks for service '{}'", service_id))
        } else {
          for (task_id, task_status) in active_tasks {
            let formatter = UnitFormatter::new(task_id, &TASK_LABELS_LIST, context);
            formatter.print(task_status, None)?;
          }
        }
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum TaskLabel {
  Healthy,
  HostIpAddress,
  LastestLog,
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
      Self::LastestLog => "latest log",
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
      Self::LastestLog => "log",
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
  fn value(&self, label: &TaskLabel, target_id: &str) -> Value {
    let task: Option<Task> = match self.actual.clone() {
      Some(actual) => Some(actual),
      None => self.configuration.clone(),
    };
    match task {
      Some(task) => match label {
        TaskLabel::Healthy => Value::option(task.healthy),
        TaskLabel::HostIpAddress => Value::plain(task.host),
        TaskLabel::LastestLog => Value::option(task.logs),
        TaskLabel::LastUpdateAt => Value::option(task.last_update.and_then(|update| DateTime::from_timestamp_millis(update).map(|ts| ts.to_string()))),
        TaskLabel::StagedAt => Value::plain(task.staged_at),
        TaskLabel::StartedAt => Value::plain(task.started_at),
        TaskLabel::State => Value::plain(task.state),
        TaskLabel::StoppedAt => Value::option(task.stopped_at),
        TaskLabel::Target => Value::plain(target_id),
      },
      None => Value::empty(),
    }
  }
}

static TASK_LABELS_LIST: [TaskLabel; 9] = [
  TaskLabel::Target,
  TaskLabel::StartedAt,
  TaskLabel::State,
  TaskLabel::Healthy,
  TaskLabel::HostIpAddress,
  TaskLabel::LastUpdateAt,
  TaskLabel::StagedAt,
  TaskLabel::StoppedAt,
  TaskLabel::LastestLog,
];
