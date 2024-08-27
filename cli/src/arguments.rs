use builder::EnumValueParser;
use clap::{builder, Arg, ArgAction};

use crate::subject::Subject;

pub(crate) const SET_VERBOSITY_ARGUMENT: &str = "set-verbosity-argument";
pub(crate) const VERBOSITY_ARGUMENT: &str = "verbosity-argument";
pub(crate) const TARGET_ARGUMENT: &str = "target-argument";
pub(crate) const _SUBTARGET_ARGUMENT: &str = "subtarget-argument";
pub(crate) const QUERY_ARGUMENT: &str = "query-argument";

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

pub(crate) fn set_verbosity_argument() -> Arg {
  Arg::new(SET_VERBOSITY_ARGUMENT)
    .long("verbosity")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<Verbosity>::new())
    .value_name("VERBOSITY")
    .help("Set the verbosity level")
    .long_help("If this option is provided, it will set the verbosity level. The possible values are 'off', 'low', 'medium' and 'high'.")
}

pub(crate) fn verbosity_argument() -> Arg {
  Arg::new(VERBOSITY_ARGUMENT)
    .short('v')
    .action(ArgAction::Count)
    .help("Verbosity level")
    .long_help("This option determines the verbosity of the information that will be written to the output.")
    .conflicts_with(SET_VERBOSITY_ARGUMENT)
}

pub(crate) fn target_argument(subject: &dyn Subject, long_help: Option<&str>) -> Arg {
  let mut target_argument = Arg::new(TARGET_ARGUMENT)
    .action(ArgAction::Set)
    .required(true)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name(subject.subject_first_upper())
    .help(format!("{} name", subject.subject_first_upper()));
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
    .help(format!("Query"));
  if let Some(long_help) = long_help {
    query_argument = query_argument.long_help(long_help.to_string())
  }
  query_argument
}