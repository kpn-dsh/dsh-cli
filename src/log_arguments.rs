use crate::global_arguments::OUTPUT_OPTIONS_HEADING;
use crate::log_level::LogLevel;
use builder::EnumValueParser;
use clap::{builder, Arg, ArgAction};

pub(crate) const LOG_LEVEL_API_ARGUMENT: &str = "log-level-api-argument";
pub(crate) const LOG_LEVEL_ARGUMENT: &str = "log-level-argument";

pub(crate) fn log_level_api_argument() -> Arg {
  Arg::new(LOG_LEVEL_API_ARGUMENT)
    .long("log-level-api")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<LogLevel>::new())
    .value_name("LEVEL")
    .long_help(
      "If this option is provided, it will set the log level for the 'dsh_api' crate. \
      The default log level is 'error'. See option --log-level for the possible values.",
    )
    .hide_short_help(true)
    .hide_possible_values(true)
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn log_level_argument() -> Arg {
  Arg::new(LOG_LEVEL_ARGUMENT)
    .long("log-level")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<LogLevel>::new())
    .value_name("LEVEL")
    .long_help(
      "If this option is provided, it will set the dsh tool's log level. \
      The default log level is 'error'.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}
