use crate::context::Context;
use crate::environment_variables::{ENV_VAR_DSH_CLI_LOG_COLOR, ENV_VAR_DSH_CLI_LOG_LEVEL, ENV_VAR_DSH_CLI_LOG_LEVEL_API, ENV_VAR_DSH_CLI_LOG_STYLE};
use crate::error::DshCliError;
use crate::log_arguments::{LOG_LEVEL_API_ARGUMENT, LOG_LEVEL_ARGUMENT};
use crate::settings::Settings;
use crate::style::{style_from, DshColor, DshStyle};
use crate::{environment_variable, error, DshCliResult};
use clap::ArgMatches;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::io::{stdout, IsTerminal};

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub(crate) enum LogLevel {
  /// No logging will be printed
  #[serde(rename = "off")]
  Off,
  /// Only errors will be logged
  #[serde(rename = "error")]
  Error,
  /// Warnings and errors will be logged
  #[serde(rename = "warn")]
  Warn,
  /// High level info, warnings and errors will be logged
  #[serde(rename = "info")]
  Info,
  /// Debug info, high level info, warnings and errors will be logged
  #[serde(rename = "debug")]
  Debug,
  /// Tracing info, debug info, high level info, warnings and errors will be logged
  #[serde(rename = "trace")]
  Trace,
}

pub(crate) fn initialize_logger(matches: &ArgMatches, settings: &Settings) -> DshCliResult<()> {
  let log_style = style_from(
    &Context::get_style(ENV_VAR_DSH_CLI_LOG_STYLE, matches, &settings.log_style, DshStyle::Dim)?,
    &Context::get_color(ENV_VAR_DSH_CLI_LOG_COLOR, matches, &settings.log_color, DshColor::Cyan)?,
  );
  let log_level_dsh = match matches.get_one::<LogLevel>(LOG_LEVEL_ARGUMENT) {
    Some(log_level_from_argument) => log_level_from_argument.clone(),
    None => match environment_variable(ENV_VAR_DSH_CLI_LOG_LEVEL, matches)? {
      Some(log_level_from_env_var) => LogLevel::try_from(log_level_from_env_var.as_str())?,
      None => settings.log_level.clone().unwrap_or_default(),
    },
  };
  let log_level_dsh_api = match matches.get_one::<LogLevel>(LOG_LEVEL_API_ARGUMENT) {
    Some(log_level_api_from_argument) => log_level_api_from_argument.clone(),
    None => match environment_variable(ENV_VAR_DSH_CLI_LOG_LEVEL_API, matches)? {
      Some(log_level_api_from_env_var) => LogLevel::try_from(log_level_api_from_env_var.as_str())?,
      None => settings.log_level_api.clone().unwrap_or_default(),
    },
  };
  if stdout().is_terminal() {
    env_logger::builder()
      .filter_module("dsh", LevelFilter::from(log_level_dsh))
      .filter_module("dsh_api", LevelFilter::from(log_level_dsh_api))
      .format_target(false)
      .format_timestamp(None)
      .format(move |buf, record| {
        writeln!(
          buf,
          "{log_style}[{}{}] {}{log_style:#}",
          record.level(),
          record
            .module_path()
            .map(|mp| if mp.starts_with("dsh_api") { ":API" } else { ":DSH" })
            .unwrap_or_default(),
          record.args()
        )
      })
      .init();
  } else {
    env_logger::builder()
      .filter_module("dsh", LevelFilter::from(log_level_dsh))
      .filter_module("dsh_api", LevelFilter::from(log_level_dsh_api))
      .format_file(true)
      .format_module_path(true)
      .format_source_path(false)
      .format_target(false)
      .format_timestamp_secs()
      .init();
  }
  Ok(())
}

impl TryFrom<&str> for LogLevel {
  type Error = DshCliError;

  fn try_from(value: &str) -> DshCliResult<Self> {
    match value {
      "off" => Ok(Self::Off),
      "error" => Ok(Self::Error),
      "warn" => Ok(Self::Warn),
      "info" => Ok(Self::Info),
      "debug" => Ok(Self::Debug),
      "trace" => Ok(Self::Trace),
      _ => Err(error!("invalid log level value '{}'", value)),
    }
  }
}

impl From<LogLevel> for LevelFilter {
  fn from(value: LogLevel) -> Self {
    match value {
      LogLevel::Off => LevelFilter::Off,
      LogLevel::Error => LevelFilter::Error,
      LogLevel::Warn => LevelFilter::Warn,
      LogLevel::Info => LevelFilter::Info,
      LogLevel::Debug => LevelFilter::Debug,
      LogLevel::Trace => LevelFilter::Trace,
    }
  }
}

impl Default for LogLevel {
  fn default() -> Self {
    Self::Error
  }
}

impl Display for LogLevel {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Off => write!(f, "off"),
      Self::Error => write!(f, "error"),
      Self::Warn => write!(f, "warn"),
      Self::Info => write!(f, "info"),
      Self::Debug => write!(f, "debug"),
      Self::Trace => write!(f, "trace"),
    }
  }
}
