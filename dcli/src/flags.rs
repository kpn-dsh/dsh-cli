use clap::{Arg, ArgAction};

use crate::flags::FlagType::*;

#[derive(Debug)]
pub(crate) enum FlagType {
  All,
  AllocationStatus,
  Configuration,
  Ids,
  Properties,
  System,
  Tasks,
  Usage,
  Value,
}

impl FlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      All => "all-flag",
      AllocationStatus => "status-flag",
      Configuration => "configuration-flag",
      Ids => "ids-flag",
      Properties => "properties-flag",
      System => "system-flag",
      Tasks => "tasks-flag",
      Usage => "usage-flag",
      Value => "value-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      All => "all",
      AllocationStatus => "status",
      Configuration => "configuration",
      Ids => "ids",
      Properties => "properties",
      System => "system",
      Tasks => "tasks",
      Usage => "usage",
      Value => "value",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      All => Some('a'),
      AllocationStatus => Some('s'),
      Configuration => Some('c'),
      Ids => Some('i'),
      Properties => Some('p'),
      System => None,
      Tasks => None,
      Usage => Some('u'),
      Value => Some('v'),
    }
  }
}

pub(crate) fn create_flag(flag_type: &FlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    All => all_flag(subject, long_help),
    AllocationStatus => allocation_status_flag(subject, long_help),
    Configuration => configuration_flag(subject, long_help),
    Ids => ids_flag(subject, long_help),
    Properties => properties_flag(subject, long_help),
    System => system_flag(subject, long_help),
    Tasks => tasks_flag(subject, long_help),
    Usage => usage_flag(subject, long_help),
    Value => value_flag(subject, long_help),
  }
}

fn all_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(All, format!("Include all {} parameters.", subject), long_help)
}

fn allocation_status_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(AllocationStatus, format!("Include the {}'s allocation status.", subject), long_help)
}

fn configuration_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(Configuration, format!("Include the {}'s initial configuration.", subject), long_help)
}

fn ids_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(Ids, format!("Include the {}'s ids.", subject), long_help)
}

fn properties_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(Properties, format!("Include the {}'s properties.", subject), long_help)
}

fn system_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(System, format!("Include the system {}'s.", subject), long_help)
}

fn tasks_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(Tasks, format!("Include the {}'s tasks.", subject), long_help)
}

fn usage_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(Usage, format!("Include the {}'s usages.", subject), long_help)
}

fn value_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_flag(Value, format!("Include the {}'s value.", subject), long_help)
}

fn create_clap_flag(flag_type: FlagType, help: String, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help);
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
