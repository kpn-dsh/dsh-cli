use crate::log_level::LogLevel;
use builder::EnumValueParser;
use clap::{builder, Arg, ArgAction};

pub(crate) const LOG_LEVEL_API_ARGUMENT: &str = "log-level-api-argument";
pub(crate) const LOG_LEVEL_ARGUMENT: &str = "log-level-argument";
pub(crate) const LOG_LEVEL_SDK_ARGUMENT: &str = "log-level-sdk-argument";

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

pub(crate) fn log_level_sdk_argument() -> Arg {
  Arg::new(LOG_LEVEL_SDK_ARGUMENT)
    .long("log-level-sdk")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<LogLevel>::new())
    .value_name("LEVEL")
    .help("Set log level for the dsh sdk crate.")
    .long_help(
      "If this option is provided, it will set the log level for the 'dsh_sdk' crate. \
      The default log level is 'error'. See option --log-level for the possible values.",
    )
    .hide_possible_values(true)
    .global(true)
}
