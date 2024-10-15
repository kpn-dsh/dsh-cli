use clap::{Arg, ArgAction};

use crate::flags::FlagType::*;
use crate::subject::Subject;

#[derive(Debug)]
pub(crate) enum FlagType {
  Actual,
  All,
  AllocationStatus,
  Configuration,
  Ids,
  Properties,
  Tasks,
  Usage,
  Value,
}

impl FlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Actual => "actual-flag",
      All => "all-flag",
      AllocationStatus => "status-flag",
      Configuration => "configuration-flag",
      Ids => "ids-flag",
      Properties => "properties-flag",
      Tasks => "tasks-flag",
      Usage => "usage-flag",
      Value => "value-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      Actual => "actual",
      All => "all",
      AllocationStatus => "status",
      Configuration => "configuration",
      Ids => "ids",
      Properties => "properties",
      Tasks => "tasks",
      Usage => "usage",
      Value => "value",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      Actual => None,
      All => Some('a'),
      AllocationStatus => Some('s'),
      Configuration => Some('c'),
      Ids => Some('i'),
      Properties => Some('p'),
      Tasks => None,
      Usage => Some('u'),
      Value => Some('v'),
    }
  }
}

pub(crate) fn create_flag(flag_type: &FlagType, subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  match flag_type {
    Actual => actual_flag(subject, long_help),
    All => all_flag(subject, long_help),
    AllocationStatus => allocation_status_flag(subject, long_help),
    Configuration => configuration_flag(subject, long_help),
    Ids => ids_flag(subject, long_help),
    Properties => properties_flag(subject, long_help),
    Tasks => tasks_flag(subject, long_help),
    Usage => usage_flag(subject, long_help),
    Value => value_flag(subject, long_help),
  }
}

fn actual_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    Actual,
    subject,
    format!("Include the {}'s actual configuration.", subject.subject()).as_str(),
    long_help,
  )
}

fn all_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(All, subject, format!("Include all {} parameters.", subject.subject()).as_str(), long_help)
}

fn allocation_status_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    AllocationStatus,
    subject,
    format!("Include the {}'s allocation status.", subject.subject()).as_str(),
    long_help,
  )
}

fn configuration_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    Configuration,
    subject,
    format!("Include the {}'s initial configuration.", subject.subject()).as_str(),
    long_help,
  )
}

fn ids_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Ids, subject, format!("Include the {}'s ids.", subject.subject()).as_str(), long_help)
}

fn properties_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Properties, subject, format!("Include the {}'s properties.", subject.subject()).as_str(), long_help)
}

fn tasks_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Tasks, subject, format!("Include the {}'s tasks.", subject.subject()).as_str(), long_help)
}

fn usage_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Usage, subject, format!("Include the {}'s usages.", subject.subject()).as_str(), long_help)
}

fn value_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Value, subject, format!("Include the {}'s value.", subject.subject()).as_str(), long_help)
}

fn create_clap_flag(flag_type: FlagType, _subject: &dyn Subject, help: &str, long_help: &Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help.to_string());
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
