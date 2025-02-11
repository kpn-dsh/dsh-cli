use crate::arguments::{LogLevel, Verbosity};
use crate::context::MatchingStyle;
use crate::formatters::OutputFormat;
use crate::{dsh_directory, read_and_deserialize_from_toml_file, serialize_and_write_to_toml_file, DEFAULT_DSH_CLI_SETTINGS_FILENAME};
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
  #[serde(rename = "log-level", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level: Option<LogLevel>,
  #[serde(rename = "log-level-api", skip_serializing_if = "Option::is_none")]
  pub(crate) log_level_api: Option<LogLevel>,
  #[serde(rename = "matching-style", skip_serializing_if = "Option::is_none")]
  pub(crate) matching_style: Option<MatchingStyle>,
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
  #[serde(rename = "terminal-width", skip_serializing_if = "Option::is_none")]
  pub(crate) terminal_width: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) verbosity: Option<Verbosity>,
  #[serde(skip_serializing)]
  pub(crate) file_name: Option<String>,
}

pub(crate) fn read_settings(explicit_settings_filename: Option<&str>) -> Result<Option<Settings>, String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("read settings from explicit file '{}'", explicit_name);
      let settings = read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(explicit_name))?;
      Ok(settings.map(|settings| Settings { file_name: Some(explicit_name.to_string()), ..settings }))
    }
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      log::debug!("read settings from default file '{}'", default_settings_file.to_string_lossy());
      let settings = read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(default_settings_file.clone()))?;
      Ok(settings.map(|settings| Settings { file_name: Some(default_settings_file.to_string_lossy().to_string()), ..settings }))
    }
  }
}

pub(crate) fn write_settings(explicit_settings_filename: Option<&str>, settings: Settings) -> Result<(), String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("write settings to explicit file '{}'", explicit_name);
      serialize_and_write_to_toml_file::<Settings>(PathBuf::new().join(explicit_name), &settings)
    }
    None => {
      let default_settings_file = dsh_directory()?.join(DEFAULT_DSH_CLI_SETTINGS_FILENAME);
      log::debug!("write settings to default file '{}'", default_settings_file.to_string_lossy());
      serialize_and_write_to_toml_file(default_settings_file, &settings)
    }
  }
}

pub(crate) fn upsert_settings<F>(explicit_settings_filename: Option<&str>, mut upsert: F) -> Result<(), String>
where
  F: FnMut(Settings) -> Result<Settings, String>,
{
  match read_settings(explicit_settings_filename)? {
    Some(existing_settings) => match upsert(existing_settings) {
      Ok(upserted_settings) => write_settings(explicit_settings_filename, upserted_settings),
      Err(_) => Err("unable to update settings".to_string()),
    },
    None => {
      let new_settings = Settings::default();
      match upsert(new_settings) {
        Ok(upserted_settings) => write_settings(explicit_settings_filename, upserted_settings),
        Err(_) => Err("unable to update settings".to_string()),
      }
    }
  }
}
