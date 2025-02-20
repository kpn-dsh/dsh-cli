use crate::log_arguments::{LOG_LEVEL_API_ARGUMENT, LOG_LEVEL_ARGUMENT, LOG_LEVEL_SDK_ARGUMENT};
use crate::settings::Settings;
use crate::{ENV_VAR_LOG_LEVEL, ENV_VAR_LOG_LEVEL_API, ENV_VAR_LOG_LEVEL_SDK};
use clap::ArgMatches;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::env;
use std::fmt::{Display, Formatter};
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

pub(crate) fn initialize_logger(matches: &ArgMatches, settings: &Settings) -> Result<(), String> {
  let log_level_dsh = match matches.get_one::<LogLevel>(LOG_LEVEL_ARGUMENT) {
    Some(log_level_from_argument) => log_level_from_argument.clone(),
    None => match env::var(ENV_VAR_LOG_LEVEL) {
      Ok(log_level_from_env_var) => LogLevel::try_from(log_level_from_env_var.as_str())?,
      Err(_) => settings.log_level.clone().unwrap_or_default(),
    },
  };
  let log_level_dsh_api = match matches.get_one::<LogLevel>(LOG_LEVEL_API_ARGUMENT) {
    Some(log_level_api_from_argument) => log_level_api_from_argument.clone(),
    None => match env::var(ENV_VAR_LOG_LEVEL_API) {
      Ok(log_level_api_from_env_var) => LogLevel::try_from(log_level_api_from_env_var.as_str())?,
      Err(_) => settings.log_level_api.clone().unwrap_or_default(),
    },
  };
  let log_level_dsh_sdk = match matches.get_one::<LogLevel>(LOG_LEVEL_SDK_ARGUMENT) {
    Some(log_level_sdk_from_argument) => log_level_sdk_from_argument.clone(),
    None => match env::var(ENV_VAR_LOG_LEVEL_SDK) {
      Ok(log_level_sdk_from_env_var) => LogLevel::try_from(log_level_sdk_from_env_var.as_str())?,
      Err(_) => settings.log_level_sdk.clone().unwrap_or_default(),
    },
  };
  if stdout().is_terminal() {
    env_logger::builder()
      .filter_module("dsh", LevelFilter::from(log_level_dsh))
      .filter_module("dsh_api", LevelFilter::from(log_level_dsh_api))
      .filter_module("dsh_sdk", LevelFilter::from(log_level_dsh_sdk))
      .format_target(false)
      .format_timestamp(None)
      .init();
  } else {
    env_logger::builder()
      .filter_module("dsh", LevelFilter::from(log_level_dsh))
      .filter_module("dsh_api", LevelFilter::from(log_level_dsh_api))
      .filter_module("dsh_sdk", LevelFilter::from(log_level_dsh_sdk))
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
  type Error = String;

  fn try_from(value: &str) -> Result<Self, String> {
    match value {
      "off" => Ok(Self::Off),
      "error" => Ok(Self::Error),
      "warn" => Ok(Self::Warn),
      "info" => Ok(Self::Info),
      "debug" => Ok(Self::Debug),
      "trace" => Ok(Self::Trace),
      _ => Err(format!("invalid log level value '{}'", value)),
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
