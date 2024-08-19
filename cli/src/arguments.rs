use builder::EnumValueParser;
use clap::{builder, Arg, ArgAction};

use crate::CommandResult;

pub(crate) enum Flag {
  All,
  AllocationStatus,
  Configuration,
  Ids,
  Tasks,
  Usage,
  Value,
}

impl Flag {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Flag::All => "all-flag",
      Flag::AllocationStatus => "status-flag",
      Flag::Configuration => "configuration-flag",
      Flag::Ids => "ids-flag",
      Flag::Tasks => "tasks-flag",
      Flag::Usage => "usage-flag",
      Flag::Value => "value-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      Flag::All => "all",
      Flag::AllocationStatus => "status",
      Flag::Configuration => "configuration",
      Flag::Ids => "ids",
      Flag::Tasks => "tasks",
      Flag::Usage => "usage",
      Flag::Value => "value",
    }
  }

  pub(crate) fn option_not_available(&self) -> CommandResult {
    Err(format!("option --{} not available", &self.option()))
  }
}

pub(crate) const SET_VERBOSITY_ARGUMENT: &str = "set-verbosity-argument";
pub(crate) const VERBOSITY_ARGUMENT: &str = "verbosity-argument";

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
