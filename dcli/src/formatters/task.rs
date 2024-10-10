use chrono::DateTime;

use dsh_api::types::{Task, TaskStatus};

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
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
  fn label_show(&self) -> &str {
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

  fn label_list(&self) -> &str {
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
