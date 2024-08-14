use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;

pub(crate) enum ApplicationLabel {
  Application,
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
  Topics,
  User,
  Volumes,
  WritableStreams,
}

lazy_static! {
  static ref DEFAULT_APPLICATION_TABLE_COLUMNS: Vec<ApplicationLabel> = vec![
    ApplicationLabel::Application,
    ApplicationLabel::NeedsToken,
    ApplicationLabel::Instances,
    ApplicationLabel::Cpus,
    ApplicationLabel::Mem,
    ApplicationLabel::ExposedPorts,
    ApplicationLabel::Metrics,
    ApplicationLabel::Image
  ];
  static ref DEFAULT_APPLICATION_TABLE_ROWS: Vec<ApplicationLabel> = vec![
    ApplicationLabel::Application,
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
}

pub(crate) fn application_table(application_id: &str, application: &Application, rows: &[ApplicationLabel]) -> Vec<Vec<String>> {
  rows
    .iter()
    .map(|column| vec![column.row_label().to_string(), application_value(application, column, application_id)])
    .collect()
}

pub(crate) fn default_application_table(application_id: &str, application: &Application) -> Vec<Vec<String>> {
  application_table(application_id, application, &DEFAULT_APPLICATION_TABLE_ROWS)
}

pub(crate) fn application_to_vector(application_id: &str, application: &Application, columns: &[ApplicationLabel]) -> Vec<String> {
  columns.iter().map(|column| application_value(application, column, application_id)).collect()
}

pub(crate) fn application_to_default_vector(application_id: &str, application: &Application) -> Vec<String> {
  application_to_vector(application_id, application, &DEFAULT_APPLICATION_TABLE_COLUMNS)
}

pub(crate) fn application_column_labels(columns: &[ApplicationLabel]) -> Vec<String> {
  columns.iter().map(|column| column.column_label().to_string()).collect()
}

pub(crate) fn default_application_column_labels() -> Vec<String> {
  application_column_labels(&DEFAULT_APPLICATION_TABLE_COLUMNS)
}

impl ApplicationLabel {
  fn row_label(&self) -> &str {
    match self {
      Self::Application => "application",
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
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn column_label(&self) -> &str {
    match self {
      Self::Application => "application",
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
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }
}

fn application_value(application: &Application, column: &ApplicationLabel, application_id: &str) -> String {
  match column {
    ApplicationLabel::Application => application_id.to_string(),
    ApplicationLabel::Cpus => application.cpus.to_string(),
    ApplicationLabel::Env => "ENV".to_string(), // TODO
    ApplicationLabel::ExposedPorts => application.exposed_ports.keys().map(|port| port.to_string()).collect::<Vec<String>>().join(","),
    ApplicationLabel::HealthCheck => match application.health_check {
      Some(ref health_check) => match health_check.protocol {
        Some(protocol) => format!("{}:{}/{}", protocol.to_string(), health_check.port, health_check.path),
        None => format!("{}/{}", health_check.port, health_check.path),
      },
      None => "".to_string(),
    },
    ApplicationLabel::Image => application.image.clone(),
    ApplicationLabel::Instances => application.instances.to_string(),
    ApplicationLabel::Mem => application.mem.to_string(),
    ApplicationLabel::Metrics => application
      .metrics
      .clone()
      .map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))
      .unwrap_or_default(),
    ApplicationLabel::NeedsToken => application.needs_token.to_string(),
    ApplicationLabel::ReadableStreams => application
      .readable_streams
      .clone()
      .into_iter()
      .map(|readable_stream| readable_stream.to_string())
      .collect::<Vec<String>>()
      .join(", "),
    ApplicationLabel::Secrets => application
      .secrets
      .clone()
      .into_iter()
      .map(|secret| secret.name)
      .collect::<Vec<String>>()
      .join(", "),
    ApplicationLabel::SingleInstance => application.single_instance.to_string(),
    ApplicationLabel::SpreadGroup => application.spread_group.clone().unwrap_or_default(),
    ApplicationLabel::Topics => application
      .topics
      .clone()
      .into_iter()
      .map(|topic| topic.to_string())
      .collect::<Vec<String>>()
      .join(", "),
    ApplicationLabel::User => application.user.clone(),
    ApplicationLabel::Volumes => application.volumes.keys().map(|k| k.to_string()).collect::<Vec<String>>().join(","),
    ApplicationLabel::WritableStreams => application
      .writable_streams
      .clone()
      .into_iter()
      .map(|writable_stream| writable_stream.to_string())
      .collect::<Vec<String>>()
      .join(", "),
  }
}
