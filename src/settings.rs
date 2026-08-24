use crate::authentication::AuthenticationMethod;
use crate::bundle::CertificateAuthorityId;
use crate::context::BrowserMethod;
use crate::directory::{get_settings, write_settings};
use crate::formatters::OutputFormat;
use crate::log_level::LogLevel;
use crate::style::{DshColor, DshStyle};
use crate::verbosity::Verbosity;
use crate::{cli_error, err, DshCliResult};
use dsh_api::platform::VhostZone;
use itertools::Itertools;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct Settings {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) authentication: Option<AuthenticationMethod>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) browser: Option<BrowserMethod>,
  #[serde(rename = "certificate-authority", skip_serializing_if = "Option::is_none")]
  pub(crate) certificate_authority: Option<CertificateAuthorityId>,
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) expiration: Option<u64>,
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
  #[serde(rename = "output-directory", skip_serializing_if = "Option::is_none")]
  pub(crate) output_directory: Option<PathBuf>,
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
  #[serde(rename = "vhost-zone", skip_serializing_if = "Option::is_none")]
  pub(crate) vhost_zone: Option<VhostZone>,
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
      .ok_or(cli_error!(""))?
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
    Err(error) => err!("unable to update settings ({})", error),
  }
}

impl Debug for Settings {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("Settings");
    if let Some(authentication) = &self.authentication {
      builder.field("authentication", authentication);
    }
    if let Some(browser) = &self.browser {
      builder.field("browser", browser);
    }
    if let Some(certificate_authority) = &self.certificate_authority {
      builder.field("certificate_authority", certificate_authority);
    }
    if let Some(csv_quote) = &self.csv_quote {
      builder.field("csv_quote", csv_quote);
    }
    if let Some(csv_separator) = &self.csv_separator {
      builder.field("csv_separator", csv_separator);
    }
    if let Some(default_platform) = &self.default_platform {
      builder.field("default_platform", default_platform);
    }
    if let Some(default_tenant) = &self.default_tenant {
      builder.field("default_tenant", default_tenant);
    }
    if let Some(dry_run) = &self.dry_run {
      builder.field("dry_run", dry_run);
    }
    if let Some(error_color) = &self.error_color {
      builder.field("error_color", error_color);
    }
    if let Some(error_style) = &self.error_style {
      builder.field("error_style", error_style);
    }
    if let Some(expiration) = &self.expiration {
      builder.field("expiration", expiration);
    }
    if let Some(label_color) = &self.label_color {
      builder.field("label_color", label_color);
    }
    if let Some(label_style) = &self.label_style {
      builder.field("label_style", label_style);
    }
    if let Some(log_color) = &self.log_color {
      builder.field("log_color", log_color);
    }
    if let Some(log_level) = &self.log_level {
      builder.field("log_level", log_level);
    }
    if let Some(log_level_api) = &self.log_level_api {
      builder.field("log_level_api", log_level_api);
    }
    if let Some(log_style) = &self.log_style {
      builder.field("log_style", log_style);
    }
    if let Some(matching_color) = &self.matching_color {
      builder.field("matching_color", matching_color);
    }
    if let Some(matching_style) = &self.matching_style {
      builder.field("matching_style", matching_style);
    }
    if let Some(no_csv_headers) = &self.no_csv_headers {
      builder.field("no_csv_headers", no_csv_headers);
    }
    if let Some(no_escape) = &self.no_escape {
      builder.field("no_escape", no_escape);
    }
    if let Some(output_directory) = &self.output_directory {
      builder.field("output_directory", output_directory);
    }
    if let Some(output_format) = &self.output_format {
      builder.field("output_format", output_format);
    }
    if let Some(quiet) = &self.quiet {
      builder.field("quiet", quiet);
    }
    if let Some(show_execution_time) = &self.show_execution_time {
      builder.field("show_execution_time", show_execution_time);
    }
    if let Some(stderr_color) = &self.stderr_color {
      builder.field("stderr_color", stderr_color);
    }
    if let Some(stderr_style) = &self.stderr_style {
      builder.field("stderr_style", stderr_style);
    }
    if let Some(stdout_color) = &self.stdout_color {
      builder.field("stdout_color", stdout_color);
    }
    if let Some(stdout_style) = &self.stdout_style {
      builder.field("stdout_style", stdout_style);
    }
    if let Some(suppress_exit_status) = &self.suppress_exit_status {
      builder.field("suppress_exit_status", suppress_exit_status);
    }
    if let Some(target_color) = &self.target_color {
      builder.field("target_color", target_color);
    }
    if let Some(target_style) = &self.target_style {
      builder.field("target_style", target_style);
    }
    if let Some(terminal_width) = &self.terminal_width {
      builder.field("terminal_width", terminal_width);
    }
    if let Some(verbosity) = &self.verbosity {
      builder.field("verbosity", verbosity);
    }
    if let Some(vhost_zone) = &self.vhost_zone {
      builder.field("vhost_zone", vhost_zone);
    }
    if let Some(file_name) = &self.file_name {
      builder.field("file_name", file_name);
    }
    if let Some(warning_color) = &self.warning_color {
      builder.field("warning_color", warning_color);
    }
    if let Some(warning_style) = &self.warning_style {
      builder.field("warning_style", warning_style);
    }
    builder.finish()
  }
}
