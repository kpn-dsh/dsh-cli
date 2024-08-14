use builder::EnumValueParser;
use clap::{builder, Arg, ArgAction};

pub(crate) const ACTUAL_FLAG: &str = "actual-argument";
pub(crate) const SET_VERBOSITY_ARGUMENT: &str = "set-verbosity-argument";
pub(crate) const STATUS_FLAG: &str = "status-argument";
// pub(crate) const TENANT_ARGUMENT: &str = "tenant-argument";
pub(crate) const VERBOSITY_ARGUMENT: &str = "verbosity-argument";
pub(crate) const USAGE_FLAG: &str = "usage-argument";

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum Verbosity {
  /// Only error messages
  Off = 0,
  /// Basic console log, like number of produced messages
  Low = 1,
  /// All arguments
  Medium = 2,
  /// Most elaborate console log
  High = 3,
}

pub(crate) fn actual_flag() -> Arg {
  Arg::new(ACTUAL_FLAG)
    .long("actual")
    .short('a')
    .action(ArgAction::SetTrue)
    .help("Actual")
    .long_help("Include deployed.")
}

pub(crate) fn set_verbosity_argument() -> Arg {
  Arg::new(SET_VERBOSITY_ARGUMENT)
    .long("verbosity")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<Verbosity>::new())
    .value_name("VERBOSITY")
    .help("Set the verbosity level")
    .long_help("If this option is provided, it will set the verbosity level. The possible values are 'off', 'low', 'medium' and 'high'.")
}

pub(crate) fn status_flag() -> Arg {
  Arg::new(STATUS_FLAG)
    .long("status")
    .short('s')
    .action(ArgAction::SetTrue)
    .help("Show allocation status")
    .long_help("Show allocation status information.")
}

pub(crate) fn usage_flag(what: &str) -> Arg {
  Arg::new(USAGE_FLAG)
    .long("usage")
    .short('u')
    .action(ArgAction::SetTrue)
    .help(format!("Show {} usage", what))
    .long_help(format!("Show where this {} is used.", what))
}

// pub(crate) fn tenant_argument() -> Arg {
//   Arg::new(TENANT_ARGUMENT)
//     .long("tenant")
//     .short('t')
//     .action(ArgAction::Set)
//     .value_parser(builder::NonEmptyStringValueParser::new())
//     .value_name("TENANT")
//     .help("Target tenant")
//     .long_help("Target tenant name.")
// }

pub(crate) fn verbosity_argument() -> Arg {
  Arg::new(VERBOSITY_ARGUMENT)
    .short('v')
    .action(ArgAction::Count)
    .help("Verbosity level")
    .long_help("This option determines the verbosity of the information that will be written to the output.")
    .conflicts_with(SET_VERBOSITY_ARGUMENT)
}
