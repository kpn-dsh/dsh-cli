use crate::formatters::OutputFormat;
use crate::log_level::LogLevel;
use crate::style::{DshColor, DshStyle};
use crate::verbosity::Verbosity;
use crate::{dsh_directory, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, DEFAULT_DSH_CLI_SETTINGS_FILENAME};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Settings {
  #[serde(rename = "csv-quote", skip_serializing_if = "Option::is_none")]
  pub(crate) csv_quote: Option<char>,
  #[serde(rename = "csv-separator", skip_serializing_if = "Option::is_none")]
  pub(crate) csv_separator: Option<String>,
  #[serde(rename = "default-platform", skip_serializing_if = "Option::is_none")]
  pub(crate) default_platform: Option<String>,
  #[serde(rename = "dry-run", skip_serializing_if = "Option::is_none")]
  pub(crate) dry_run: Option<bool>,
  #[serde(rename = "default-tenant", skip_serializing_if = "Option::is_none")]
  pub(crate) default_tenant: Option<String>,
  #[serde(rename = "error-color", skip_serializing_if = "Option::is_none")]
  pub(crate) error_color: Option<DshColor>,
  #[serde(rename = "error-style", skip_serializing_if = "Option::is_none")]
  pub(crate) error_style: Option<DshStyle>,
  #[serde(rename = "log-level", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level: Option<LogLevel>,
  #[serde(rename = "log-level-api", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level_api: Option<LogLevel>,
  #[serde(rename = "log-level-sdk", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level_sdk: Option<LogLevel>,
  #[serde(rename = "matching-color", skip_serializing_if = "Option::is_none")]
  pub(crate) matching_color: Option<DshColor>,
  #[serde(rename = "matching-style", skip_serializing_if = "Option::is_none")]
  pub(crate) matching_style: Option<DshStyle>,
  #[serde(rename = "no-escape", skip_serializing_if = "Option::is_none")]
  pub(crate) no_escape: Option<bool>,
  #[serde(rename = "no-headers", skip_serializing_if = "Option::is_none")]
  pub(crate) no_headers: Option<bool>,
  #[serde(rename = "output-format", skip_serializing_if = "Option::is_none")]
  pub(crate) output_format: Option<OutputFormat>,
  #[serde(rename = "quiet", skip_serializing_if = "Option::is_none")]
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

pub(crate) fn get_settings(explicit_settings_filename: Option<&str>) -> Result<(Settings, String), String> {
  match explicit_settings_filename {
    Some(explicit_name) => match read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(explicit_name))? {
      Some(settings_from_explicit_file) => Ok((
        Settings { file_name: Some(explicit_name.to_string()), ..settings_from_explicit_file },
        format!("read settings (explicit file '{}')", explicit_name),
      )),
      None => Err(format!("explicit settings file '{}' does not exist", explicit_name)),
    },
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      match read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(default_settings_file.clone()))? {
        Some(settings_from_default_file) => Ok((
          Settings { file_name: Some(default_settings_file.to_string_lossy().to_string()), ..settings_from_default_file },
          format!("read settings (default file '{}')", default_settings_file.to_string_lossy()),
        )),
        None => Ok((Settings::default(), "default settings".to_string())),
      }
    }
  }
}

pub(crate) fn write_settings(explicit_settings_filename: Option<&str>, settings: Settings) -> Result<(), String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      debug!("write settings to explicit file '{}'", explicit_name);
      serialize_and_write_to_toml_file::<Settings>(PathBuf::new().join(explicit_name), &settings)
    }
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      debug!("write settings to default file '{}'", default_settings_file.to_string_lossy());
      serialize_and_write_to_toml_file(default_settings_file, &settings)
    }
  }
}

pub(crate) fn upsert_settings<F>(explicit_settings_filename: Option<&str>, mut upsert: F) -> Result<(), String>
where
  F: FnMut(Settings) -> Result<Settings, String>,
{
  match upsert(get_settings(explicit_settings_filename)?.0) {
    Ok(upserted_settings) => {
      debug!("updated settings");
      write_settings(explicit_settings_filename, upserted_settings)
    }
    Err(error) => Err(format!("unable to update settings ({})", error)),
  }
}
