use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub(crate) enum Verbosity {
  /// No logging will be printed
  #[serde(rename = "off")]
  Off = 1,
  /// Lowest verbosity level, only error messages will be printed
  #[serde(rename = "low")]
  Low = 2,
  /// Medium verbosity level, some info will be printed
  #[serde(rename = "medium")]
  Medium = 3,
  /// Highest verbosity level, all info will be printed
  #[serde(rename = "high")]
  High = 4,
}

impl TryFrom<&str> for Verbosity {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "off" => Ok(Self::Off),
      "low" => Ok(Self::Low),
      "medium" => Ok(Self::Medium),
      "high" => Ok(Self::High),
      _ => Err(format!("invalid verbosity value '{}'", value)),
    }
  }
}

impl Display for Verbosity {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Verbosity::Off => write!(f, "off"),
      Verbosity::Low => write!(f, "low"),
      Verbosity::Medium => write!(f, "medium"),
      Verbosity::High => write!(f, "high"),
    }
  }
}
