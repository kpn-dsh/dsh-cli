use crate::context::MatchingStyle;
use crate::formatters::OutputFormat;
use builder::EnumValueParser;
use clap::builder::{PossibleValue, ValueParser};
use clap::{builder, Arg, ArgAction};
use dsh_api::platform::DshPlatform;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub(crate) const APP_ARGUMENT: &str = "app-argument";
pub(crate) const GUID_ARGUMENT: &str = "guid-argument";
pub(crate) const PASSWORD_FILE_ARGUMENT: &str = "password-file-argument";
pub(crate) const PLATFORM_ARGUMENT: &str = "platform-argument";
pub(crate) const SERVICE_ARGUMENT: &str = "service-argument";
pub(crate) const TENANT_ARGUMENT: &str = "tenant-argument";
pub(crate) const VENDOR_ARGUMENT: &str = "vendor-argument";
pub(crate) const VHOST_ARGUMENT: &str = "vhost-argument";

// pub(crate) const TO_CLIPBOARD_ARGUMENT: &str = "to-clipboard-argument";
// pub(crate) const FROM_CLIPBOARD_ARGUMENT: &str = "from-clipboard-argument";
pub(crate) const DRY_RUN_ARGUMENT: &str = "dry-run-argument";
pub(crate) const FORCE_ARGUMENT: &str = "force-argument";
pub(crate) const LOG_LEVEL_API_ARGUMENT: &str = "log-level-api-argument";
pub(crate) const LOG_LEVEL_ARGUMENT: &str = "log-level-argument";
pub(crate) const MATCHING_STYLE_ARGUMENT: &str = "matching-style-argument";
pub(crate) const NO_ESCAPE_ARGUMENT: &str = "no-escape-argument";
pub(crate) const OUTPUT_FORMAT_ARGUMENT: &str = "output-format-argument";
pub(crate) const QUERY_ARGUMENT: &str = "query-argument";
pub(crate) const QUIET_ARGUMENT: &str = "quiet-argument";
pub(crate) const SHOW_EXECUTION_TIME_ARGUMENT: &str = "show-execution-time-argument";
pub(crate) const TARGET_ARGUMENT: &str = "target-argument";
pub(crate) const TERMINAL_WIDTH_ARGUMENT: &str = "terminal-width-argument";
pub(crate) const VERBOSITY_ARGUMENT: &str = "set-verbosity-argument";
pub(crate) const _SUBTARGET_ARGUMENT: &str = "subtarget-argument";

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

pub(crate) fn app_argument() -> Arg {
  Arg::new(APP_ARGUMENT)
    .long("app")
    .short('a')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("APP")
    .help("Provide app.")
    .long_help("This option specifies the name of an app running on the DSH platform.")
}

pub(crate) fn dry_run_argument() -> Arg {
  Arg::new(DRY_RUN_ARGUMENT)
    .long("dry-run")
    .action(ArgAction::SetTrue)
    .help("Execute in dry-run mode.")
    .long_help(
      "When this option is provided the tool will run in dry-run mode, \
          meaning that no changes will be made to the \
          resources and applications on the DSH. Dry-run mode can also be set by the \
          environment variable DSH_CLI_DRY_RUN or in the settings file.",
    )
    .conflicts_with(FORCE_ARGUMENT)
    .global(true)
}

pub(crate) fn force_argument() -> Arg {
  Arg::new(FORCE_ARGUMENT)
    .long("force")
    .action(ArgAction::SetTrue)
    .help("Force changes without confirmation.")
    .long_help(
      "When this option is provided any change, update and delete actions \
          will be executed without asking for confirmation.",
    )
    .global(true)
}

// pub(crate) fn from_clipboard_argument() -> Arg {
//   Arg::new(FROM_CLIPBOARD_ARGUMENT)
//     .long("from-clipboard")
//     .action(ArgAction::SetTrue)
//     .help("Read input from clipboard.")
//     .long_help(
//       "When this option is provided the input for methods that require it \
//           will be read from the clipboard, \
//           instead of being read from the terminal, pipes or redirects.",
//     )
//     .global(true)
// }

pub(crate) fn guid_argument() -> Arg {
  Arg::new(GUID_ARGUMENT)
    .long("guid")
    .short('g')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("GUID")
    .help("Provide target group and user id.")
    .long_help(
      "This option specifies the group and user id of the target tenant. \
          If this argument is not provided, \
          the guid must be specified via the environment variable DSH_CLI_GUID, \
          as a default setting in the settings file, or else the user will be prompted. \
          Note that if the tenant is already provided, the target settings file will also be \
          checked for the guid value.",
    )
}

pub(crate) fn log_level_argument() -> Arg {
  Arg::new(LOG_LEVEL_ARGUMENT)
    .long("log-level")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<LogLevel>::new())
    .value_name("LEVEL")
    .help("Set log level.")
    .long_help(
      "If this option is provided, it will set the tool's log level. \
      The default log level is 'error'.",
    )
    .global(true)
}

pub(crate) fn log_level_api_argument() -> Arg {
  Arg::new(LOG_LEVEL_API_ARGUMENT)
    .long("log-level-api")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<LogLevel>::new())
    .value_name("LEVEL")
    .help("Set log level for the dsh api crate.")
    .long_help(
      "If this option is provided, it will set the log level for the 'dsh_api' crate. \
      The default log level is 'error'. See option --log-level for the possible values.",
    )
    .hide_possible_values(true)
    .global(true)
}

pub(crate) fn matching_style_argument() -> Arg {
  Arg::new(MATCHING_STYLE_ARGUMENT)
    .long("matching-style")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<MatchingStyle>::new())
    .value_name("STYLE")
    .help("Set styling for matches.")
    .long_help(
      "This option specifies the styling to be used when printing matching results \
          for the find functions, e.q. when matching regular expressions. \
          If this argument is not provided, the value from environment variable \
          DSH_CLI_MATCHING_STYLE or the value from the settings file will be used. \
          The default style is 'bold'.",
    )
}

pub(crate) fn no_escape_argument() -> Arg {
  Arg::new(NO_ESCAPE_ARGUMENT)
    .long("no-color")
    .alias("no-ansi")
    .action(ArgAction::SetTrue)
    .help("No color.")
    .long_help(
      "When this option is provided the output will not contain \
          any color or other ansi escape sequences. \
          If this argument is not provided, the environment variable \
          DSH_CLI_NO_ESCAPE or the value from the settings file will be used. \
          The default behavior is to use ansi escape styling where applicable.",
    )
}

pub(crate) fn output_format_argument() -> Arg {
  Arg::new(OUTPUT_FORMAT_ARGUMENT)
    .long("output-format")
    .short('o')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<OutputFormat>::new())
    .value_name("FORMAT")
    .help("Set output format.")
    .long_help(
      "This option specifies the format used when printing the output. \
          If this argument is not provided, the value from the environment variable \
          DSH_CLI_OUTPUT_FORMAT of the value from the settings file will be used. \
          By default, when stdout is a terminal 'table' will be used, \
          while if stdout is not a terminal 'json' will be used.",
    )
}

pub(crate) fn password_file_argument() -> Arg {
  Arg::new(PASSWORD_FILE_ARGUMENT)
    .long("password-file")
    .action(ArgAction::Set)
    .value_parser(ValueParser::path_buf())
    .value_name("FILE")
    .help("Provide password file name.")
    .long_help(
      "This option specifies the name of a file that contains the api password. \
          If this flag is not provided, the environment variable \
          DSH_CLI_PASSWORD_FILE will be tried. Else, if the platform and tenant are known, \
          the target settings file will be checked. \
          Finally, the user will be prompted for the password.",
    )
}

pub(crate) fn platform_argument() -> Arg {
  let possible_values = DshPlatform::all()
    .iter()
    .map(|platform| {
      PossibleValue::new(platform.name())
        .alias(platform.alias())
        .help(format!("{} ({})", platform.description(), platform.alias()))
    })
    .collect::<Vec<_>>();
  Arg::new(PLATFORM_ARGUMENT)
    .long("platform")
    .short('p')
    .action(ArgAction::Set)
    .value_parser(possible_values)
    .value_name("PLATFORM")
    .help("Provide target platform.")
    .long_help(
      "This option specifies the name of the target platform. \
          If this argument is not provided, \
          the platform must be specified via the environment variable DSH_CLI_PLATFORM, \
          as a default setting in the settings file, or else the user will be prompted. \
          The value between parentheses can be used as an alias for the platform name.",
    )
    .global(true)
}

pub(crate) fn quiet_argument() -> Arg {
  Arg::new(QUIET_ARGUMENT)
    .long("quiet")
    .short('q')
    .action(ArgAction::SetTrue)
    .help("Run in quiet mode.")
    .long_help(
      "When this option is provided the tool will run in quiet mode, \
          meaning that no output will be produced to the terminal (stdout and stderr).",
    )
    .global(true)
}

pub(crate) fn set_verbosity_argument() -> Arg {
  Arg::new(VERBOSITY_ARGUMENT)
    .long("verbosity")
    .short('v')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<Verbosity>::new())
    .value_name("VERBOSITY")
    .help("Set verbosity level.")
    .long_help(
      "If this option is provided, \
    it will set the verbosity level. \
    The default verbosity setting is 'low'.",
    )
    .global(true)
}

pub(crate) fn service_argument() -> Arg {
  Arg::new(SERVICE_ARGUMENT)
    .long("service")
    .short('s')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("SERVICE")
    .help("Provide service.")
    .long_help(
      "This option specifies the name of a service running on the DSH platform. \
          If this argument is not provided, \
          the service could be specified via the environment variable DSH_CLI_SERVICE \
          or else the user will be prompted.",
    )
}

pub(crate) fn show_execution_time_argument() -> Arg {
  Arg::new(SHOW_EXECUTION_TIME_ARGUMENT)
    .long("show-execution-time")
    .action(ArgAction::SetTrue)
    .help("Show execution time.")
    .long_help(
      "When this option is provided the execution time of the executed function \
          will be shown, in milliseconds.",
    )
}

pub(crate) fn target_argument(subject: &str, long_help: Option<&str>) -> Arg {
  let mut target_argument = Arg::new(TARGET_ARGUMENT)
    .action(ArgAction::Set)
    .required(true)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .help(format!("{} name", subject))
    .value_name(subject.to_uppercase());
  if let Some(long_help) = long_help {
    target_argument = target_argument.long_help(long_help.to_string())
  }
  target_argument
}

pub(crate) fn tenant_argument() -> Arg {
  Arg::new(TENANT_ARGUMENT)
    .long("tenant")
    .short('t')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Provide target tenant.")
    .long_help(
      "This option specifies the name of the target tenant. \
          If this argument is not provided, \
          the tenant should be specified via the environment variable DSH_CLI_TENANT,\
          as a default setting in the settings file, or else the user will be prompted.",
    )
    .global(true)
}

pub(crate) fn terminal_width_argument() -> Arg {
  Arg::new(TERMINAL_WIDTH_ARGUMENT)
    .long("terminal-width")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<usize>::from(40..))
    .value_name("WIDTH")
    .help("Set terminal width.")
    .long_help("With this option the maximum terminal width can be set. If not set, the environment variable  By default")
}

// pub(crate) fn to_clipboard_argument() -> Arg {
//   Arg::new(TO_CLIPBOARD_ARGUMENT)
//     .long("to-clipboard")
//     .action(ArgAction::SetTrue)
//     .help("Copy output to clipboard.")
//     .long_help(
//       "When this option is provided the output will be copied to the clipboard, \
//           instead of being printed to the terminal.",
//     )
//     .global(true)
// }

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

pub(crate) fn vendor_argument() -> Arg {
  Arg::new(VENDOR_ARGUMENT)
    .long("vendor")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VENDOR")
    .help("Provide app vendor.")
    .long_help("This option specifies the name of an app vendor. Allowed values are \"kpn\".")
}

pub(crate) fn vhost_argument() -> Arg {
  Arg::new(VHOST_ARGUMENT)
    .long("vhost")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VHOST")
    .help("Provide vhost.")
    .long_help("This option specifies the name of a vhost.")
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

impl TryFrom<&str> for Verbosity {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "off" => Ok(Self::Off),
      "low" => Ok(Self::Low),
      "medium" => Ok(Self::Medium),
      "high" => Ok(Self::Medium),
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
