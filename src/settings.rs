use crate::authentication::AuthenticationMethod;
use crate::context::BrowserMethod;
use crate::directory::{get_settings, write_settings};
use crate::formatters::OutputFormat;
use crate::log_level::LogLevel;
use crate::style::{DshColor, DshStyle};
use crate::verbosity::Verbosity;
use crate::{error, DshCliResult};
use itertools::Itertools;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Settings {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) authentication: Option<AuthenticationMethod>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) browser: Option<BrowserMethod>,
  #[serde(rename = "csv-quote", skip_serializing_if = "Option::is_none")]
  pub(crate) csv_quote: Option<char>,
  #[serde(rename = "csv-separator", skip_serializing_if = "Option::is_none")]
  pub(crate) csv_separator: Option<String>,
  #[serde(rename = "default-platform", skip_serializing_if = "Option::is_none")]
  pub(crate) default_platform: Option<String>,
  #[serde(rename = "default-tenant", skip_serializing_if = "Option::is_none")]
  pub(crate) default_tenant: Option<String>,
  #[serde(rename = "dry-run", skip_serializing_if = "Option::is_none")]
  pub(crate) dry_run: Option<bool>,
  #[serde(rename = "error-color", skip_serializing_if = "Option::is_none")]
  pub(crate) error_color: Option<DshColor>,
  #[serde(rename = "error-style", skip_serializing_if = "Option::is_none")]
  pub(crate) error_style: Option<DshStyle>,
  #[serde(rename = "label-color", skip_serializing_if = "Option::is_none")]
  pub(crate) label_color: Option<DshColor>,
  #[serde(rename = "label-style", skip_serializing_if = "Option::is_none")]
  pub(crate) label_style: Option<DshStyle>,
  #[serde(rename = "log-color", skip_serializing_if = "Option::is_none")]
  pub(crate) log_color: Option<DshColor>,
  #[serde(rename = "log-level", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level: Option<LogLevel>,
  #[serde(rename = "log-level-api", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level_api: Option<LogLevel>,
  #[serde(rename = "log-style", skip_serializing_if = "Option::is_none")]
  pub(crate) log_style: Option<DshStyle>,
  #[serde(rename = "matching-color", skip_serializing_if = "Option::is_none")]
  pub(crate) matching_color: Option<DshColor>,
  #[serde(rename = "matching-style", skip_serializing_if = "Option::is_none")]
  pub(crate) matching_style: Option<DshStyle>,
  #[serde(rename = "no-csv-headers", skip_serializing_if = "Option::is_none")]
  pub(crate) no_csv_headers: Option<bool>,
  #[serde(rename = "no-escape", skip_serializing_if = "Option::is_none")]
  pub(crate) no_escape: Option<bool>,
  #[serde(rename = "output-format", skip_serializing_if = "Option::is_none")]
  pub(crate) output_format: Option<OutputFormat>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) quiet: Option<bool>,
  #[serde(rename = "show-execution-time", skip_serializing_if = "Option::is_none")]
  pub(crate) show_execution_time: Option<bool>,
  #[serde(rename = "stderr-color", skip_serializing_if = "Option::is_none")]
  pub(crate) stderr_color: Option<DshColor>,
  #[serde(rename = "stderr-style", skip_serializing_if = "Option::is_none")]
  pub(crate) stderr_style: Option<DshStyle>,
  #[serde(rename = "stdout-color", skip_serializing_if = "Option::is_none")]
  pub(crate) stdout_color: Option<DshColor>,
  #[serde(rename = "stdout-style", skip_serializing_if = "Option::is_none")]
  pub(crate) stdout_style: Option<DshStyle>,
  #[serde(rename = "suppress-exit-status", skip_serializing_if = "Option::is_none")]
  pub(crate) suppress_exit_status: Option<bool>,
  #[serde(rename = "target-color", skip_serializing_if = "Option::is_none")]
  pub(crate) target_color: Option<DshColor>,
  #[serde(rename = "target-style", skip_serializing_if = "Option::is_none")]
  pub(crate) target_style: Option<DshStyle>,
  #[serde(rename = "terminal-width", skip_serializing_if = "Option::is_none")]
  pub(crate) terminal_width: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) verbosity: Option<Verbosity>,
  #[serde(skip_serializing)]
  pub(crate) file_name: Option<String>,
  #[serde(rename = "warning-color", skip_serializing_if = "Option::is_none")]
  pub(crate) warning_color: Option<DshColor>,
  #[serde(rename = "warning-style", skip_serializing_if = "Option::is_none")]
  pub(crate) warning_style: Option<DshStyle>,
}

impl Settings {
  pub(crate) fn non_empty_attributes(&self) -> DshCliResult<Vec<(String, String)>> {
    let file_name = &self.file_name;
    let mut valued_attributes: Vec<(String, String)> = serde_json::from_str::<Value>(serde_json::to_string(self)?.as_str())?
      .as_object()
      .ok_or(error!(""))?
      .iter()
      .map(|(attribute, value)| {
        (
          attribute.to_string(),
          match value {
            Value::String(string) => string.to_string(),
            other => other.to_string(),
          }
          .to_string(),
        )
      })
      .collect_vec();
    if let Some(name) = file_name {
      valued_attributes.push(("file-name".to_string(), name.to_string()));
    }
    valued_attributes.sort_by(|(attribute_a, _), (attribute_b, _)| attribute_a.cmp(attribute_b));
    Ok(valued_attributes)
  }
}

pub(crate) fn upsert_settings<F>(upsert: F) -> DshCliResult<()>
where
  F: FnOnce(Settings) -> Result<Settings, String>,
{
  match upsert(get_settings()?.0) {
    Ok(upserted_settings) => {
      debug!("updated settings");
      write_settings(upserted_settings)
    }
    Err(error) => Err(error!("unable to update settings ({})", error)),
  }
}
