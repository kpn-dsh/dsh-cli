use builder::EnumValueParser;
use clap::{builder, Arg, ArgAction};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub(crate) const GUID_ARGUMENT: &str = "guid-argument";
pub(crate) const NO_BORDER_ARGUMENT: &str = "no-border-argument";
pub(crate) const PASSWORD_ARGUMENT: &str = "password-argument";
pub(crate) const PLATFORM_ARGUMENT: &str = "platform-argument";
pub(crate) const SET_VERBOSITY_ARGUMENT: &str = "set-verbosity-argument";
pub(crate) const _SUBTARGET_ARGUMENT: &str = "subtarget-argument";
pub(crate) const TARGET_ARGUMENT: &str = "target-argument";
pub(crate) const TENANT_ARGUMENT: &str = "tenant-argument";
pub(crate) const QUERY_ARGUMENT: &str = "query-argument";

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub(crate) enum Verbosity {
  /// Only error messages
  #[serde(rename = "off")]
  Off = 0,
  /// Basic console log, like number of produced messages
  #[serde(rename = "low")]
  Low = 1,
  /// All arguments
  #[serde(rename = "medium")]
  Medium = 2,
  /// Most elaborate console log
  #[serde(rename = "high")]
  High = 3,
}

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub(crate) enum PlatformArgument {
  /// Non-production landing zone
  #[serde(rename = "nplz")]
  Nplz,
  /// Proof of concept
  #[serde(rename = "poc")]
  Poc,
  /// Production landing zone
  #[serde(rename = "prod")]
  Prod,
  /// Production AZ
  #[serde(rename = "prodaz")]
  Prodaz,
  /// Production LZ
  #[serde(rename = "prodlz")]
  Prodlz,
}

pub(crate) fn guid_argument() -> Arg {
  Arg::new(GUID_ARGUMENT)
    .long("guid")
    .short('g')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("GUID")
    .help("Target group and user id")
    .long_help(
      "This option specifies the group and user id of the target tenant. \
    If this argument is not provided, \
    the tenant must be specified via the environment variable DSH_API_TENANT.",
    )
}

pub(crate) fn no_border_argument() -> Arg {
  Arg::new(NO_BORDER_ARGUMENT)
    .long("no-border")
    .action(ArgAction::SetTrue)
    .help("Omit output border")
    .long_help("When this option is provided table borders will be omitted from the output.")
}

pub(crate) fn password_argument() -> Arg {
  Arg::new(PASSWORD_ARGUMENT)
    .long("secret")
    .action(ArgAction::SetTrue)
    .help("Ask for secret")
    .long_help("When this option is provided the user will always be asked to provide the api secret.")
}

pub(crate) fn platform_argument() -> Arg {
  Arg::new(PLATFORM_ARGUMENT)
    .long("platform")
    .short('p')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<PlatformArgument>::new())
    .value_name("PLATFORM")
    .help("Target platform")
    .long_help(
      "This option specifies the name of the target platform. \
    If this argument is not provided, \
    the platform must be specified via the environment variable DSH_API_PLATFORM.",
    )
}

pub(crate) fn set_verbosity_argument() -> Arg {
  Arg::new(SET_VERBOSITY_ARGUMENT)
    .long("verbosity")
    .short('v')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<Verbosity>::new())
    .value_name("VERBOSITY")
    .help("Verbosity level")
    .long_help(
      "If this option is provided, \
    it will set the verbosity level. \
    The default verbosity setting is 'low'.",
    )
}

pub(crate) fn tenant_argument() -> Arg {
  Arg::new(TENANT_ARGUMENT)
    .long("tenant")
    .short('t')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Target tenant")
    .long_help(
      "This option specifies the name of the target tenant. \
    If this argument is not provided, \
    the tenant must be specified via the environment variable DSH_API_TENANT.",
    )
}

pub(crate) fn target_argument(subject: &str, long_help: Option<&str>) -> Arg {
  let mut target_argument = Arg::new(TARGET_ARGUMENT)
    .action(ArgAction::Set)
    .required(true)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .help(format!("{} name", subject))
    .value_name(subject.to_string());
  if let Some(long_help) = long_help {
    target_argument = target_argument.long_help(long_help.to_string())
  }
  target_argument
}

pub(crate) fn _subtarget_argument(subtarget: &str, long_help: Option<&str>) -> Arg {
  let mut subtarget_argument = Arg::new(_SUBTARGET_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name(subtarget.to_string())
    .help(format!("{} name", subtarget));
  if let Some(long_help) = long_help {
    subtarget_argument = subtarget_argument.long_help(long_help.to_string())
  }
  subtarget_argument
}

pub(crate) fn query_argument(long_help: Option<&str>) -> Arg {
  let mut query_argument = Arg::new(QUERY_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("QUERY")
    .help("Query");
  if let Some(long_help) = long_help {
    query_argument = query_argument.long_help(long_help.to_string())
  }
  query_argument
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

impl Display for PlatformArgument {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PlatformArgument::Nplz => write!(f, "nplz"),
      PlatformArgument::Poc => write!(f, "poc"),
      PlatformArgument::Prod => write!(f, "prod"),
      PlatformArgument::Prodaz => write!(f, "prodaz"),
      PlatformArgument::Prodlz => write!(f, "prodlz"),
    }
  }
}
