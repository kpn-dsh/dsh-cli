use dsh_api::types::Notification;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub(crate) mod formatter;
pub(crate) mod ids_formatter;
pub(crate) mod list_formatter;
pub(crate) mod unit_formatter;

#[derive(clap::ValueEnum, Eq, Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub enum OutputFormat {
  /// Output will be formatted as comma separated values
  #[serde(rename = "csv")]
  Csv,
  /// Output will be in json format
  #[serde(rename = "json")]
  Json,
  /// Output will be in compact json format
  #[serde(rename = "json-compact")]
  JsonCompact,
  /// Output will be formatted as plain text
  #[serde(rename = "plain")]
  Plain,
  /// No output will be generated
  #[serde(rename = "quiet")]
  Quiet,
  /// Output will be formatted as a table with borders
  #[serde(rename = "table")]
  Table,
  /// Output will be formatted as a table without borders
  #[serde(rename = "table-no-border")]
  TableNoBorder,
  /// Output will be in toml format
  #[serde(rename = "toml")]
  Toml,
  /// Output will be in compact toml format
  #[serde(rename = "toml-compact")]
  TomlCompact,
  /// Output will be in yaml format
  #[serde(rename = "yaml")]
  Yaml,
}

impl Display for OutputFormat {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      OutputFormat::Csv => write!(f, "csv"),
      OutputFormat::Json => write!(f, "json"),
      OutputFormat::JsonCompact => write!(f, "json-compact"),
      OutputFormat::Plain => write!(f, "plain"),
      OutputFormat::Quiet => write!(f, "quiet"),
      OutputFormat::Table => write!(f, "table"),
      OutputFormat::TableNoBorder => write!(f, "table-no-border"),
      OutputFormat::Toml => write!(f, "toml"),
      OutputFormat::TomlCompact => write!(f, "toml-compact"),
      OutputFormat::Yaml => write!(f, "yaml"),
    }
  }
}

impl TryFrom<&str> for OutputFormat {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "csv" => Ok(Self::Csv),
      "json" => Ok(Self::Json),
      "json-compact" => Ok(Self::JsonCompact),
      "plain" => Ok(Self::Plain),
      "quiet" => Ok(Self::Quiet),
      "table" => Ok(Self::Table),
      "table-no-border" => Ok(Self::TableNoBorder),
      "toml" => Ok(Self::Toml),
      "toml-compact" => Ok(Self::TomlCompact),
      "yaml" => Ok(Self::Yaml),
      _ => Err(format!("invalid output format '{}'", value)),
    }
  }
}

pub(crate) fn notifications_to_string(notifications: &[Notification]) -> String {
  notifications.iter().map(notification_to_string).collect::<Vec<_>>().join(", ")
}

pub(crate) fn notification_to_string(notification: &Notification) -> String {
  format!(
    "{}, {}, {}",
    if notification.remove { "remove".to_string() } else { "create/update".to_string() },
    notification.message,
    notification
      .args
      .iter()
      .map(|(key, value)| format!("{}:{}", key, value))
      .collect::<Vec<_>>()
      .join(", "),
  )
}
