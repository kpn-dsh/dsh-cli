use lazy_static::lazy_static;

use trifonius_dsh_api::types::AppCatalogApp;

use crate::app::get_application_from_app;
use crate::formatters::application::{application_value, ApplicationLabel};

pub(crate) enum AppLabel {
  App,
  Configuration,
  ManifestUrl,
}

lazy_static! {
  static ref DEFAULT_APP_TABLE_COLUMNS: Vec<AppLabel> = vec![AppLabel::App, AppLabel::ManifestUrl];
  static ref DEFAULT_APP_TABLE_ROWS: Vec<AppLabel> = vec![AppLabel::App, AppLabel::Configuration, AppLabel::ManifestUrl];
  static ref DEFAULT_APP_APPLICATION_RESOURCE_TABLE_COLUMNS: Vec<ApplicationLabel> =
    vec![ApplicationLabel::Application, ApplicationLabel::NeedsToken, ApplicationLabel::Instances, ApplicationLabel::Cpus, ApplicationLabel::Mem,];
  static ref DEFAULT_APP_APPLICATION_RESOURCE_TABLE_ROWS: Vec<ApplicationLabel> = vec![
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

pub(crate) fn _app_table(app_id: &str, app: &AppCatalogApp, app_rows: &[AppLabel], application_rows: &[ApplicationLabel]) -> Vec<Vec<String>> {
  let mut table: Vec<Vec<String>> = vec![];
  for app_row in app_rows {
    table.push(vec![app_row.row_label().to_string(), app_value(app, app_row, app_id)])
  }
  match get_application_from_app(app) {
    Some((application_resource, application)) => {
      for application_row in application_rows {
        table.push(vec![
          application_row.row_label().to_string(),
          application_value(application, application_row, application_resource),
        ])
      }
      table
    }
    None => {
      table.push(vec!["application".to_string(), "not available".to_string()]);
      table
    }
  }
}

pub(crate) fn _default_app_table(app_id: &str, app: &AppCatalogApp) -> Vec<Vec<String>> {
  _app_table(app_id, app, &DEFAULT_APP_TABLE_ROWS, &DEFAULT_APP_APPLICATION_RESOURCE_TABLE_ROWS)
}

pub(crate) fn app_to_vector(app_id: &str, app: &AppCatalogApp, app_columns: &[AppLabel], application_columns: &[ApplicationLabel]) -> Vec<String> {
  let mut vector: Vec<String> = vec![];
  for app_column in app_columns {
    vector.push(app_value(app, app_column, app_id))
  }
  match get_application_from_app(app) {
    Some((application_resource, application)) => {
      for application_column in application_columns {
        vector.push(application_value(application, application_column, application_resource))
      }
      vector
    }
    None => {
      vector.push("not available".to_string());
      vector
    }
  }
}

pub(crate) fn app_to_default_vector(app_id: &str, app: &AppCatalogApp) -> Vec<String> {
  app_to_vector(app_id, app, &DEFAULT_APP_TABLE_COLUMNS, &DEFAULT_APP_APPLICATION_RESOURCE_TABLE_COLUMNS)
}

pub(crate) fn app_column_labels(app_columns: &[AppLabel], application_columns: &[ApplicationLabel]) -> Vec<String> {
  let mut labels: Vec<String> = vec![];
  for app_column in app_columns {
    labels.push(app_column.column_label().to_string())
  }
  for application_column in application_columns {
    labels.push(application_column.column_label().to_string())
  }
  labels
}

pub(crate) fn default_app_column_labels() -> Vec<String> {
  app_column_labels(&DEFAULT_APP_TABLE_COLUMNS, &DEFAULT_APP_APPLICATION_RESOURCE_TABLE_COLUMNS)
}

fn app_value(app: &AppCatalogApp, label: &AppLabel, app_id: &str) -> String {
  match label {
    AppLabel::App => app_id.to_string(),
    AppLabel::Configuration => app.configuration.clone().unwrap_or_default().to_string(),
    AppLabel::ManifestUrl => app.manifest_urn.clone(),
  }
}

impl AppLabel {
  fn row_label(&self) -> &str {
    match self {
      AppLabel::App => "app",
      AppLabel::Configuration => "app configuration",
      AppLabel::ManifestUrl => "manifest url",
    }
  }

  fn column_label(&self) -> &str {
    match self {
      AppLabel::App => "app",
      AppLabel::Configuration => "configuration",
      AppLabel::ManifestUrl => "manifest",
    }
  }
}
