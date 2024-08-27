use clap::{Arg, ArgAction};

use crate::flags::FlagType::*;
use crate::subject::Subject;
use crate::CommandResult;

#[derive(Debug)]
pub(crate) enum FlagType {
  Actual,
  All,
  AllocationStatus,
  Applications,
  Apps,
  Configuration,
  Ids,
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
      Applications => "applications-flag",
      Apps => "apps-flag",
      Configuration => "configuration-flag",
      Ids => "ids-flag",
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
      Applications => "application",
      Apps => "app",
      Configuration => "configuration",
      Ids => "ids",
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
      Applications => None,
      Apps => None,
      Configuration => Some('c'),
      Ids => Some('i'),
      Tasks => None,
      Usage => Some('u'),
      Value => Some('v'),
    }
  }

  pub(crate) fn option_not_available(&self) -> CommandResult {
    Err(format!("option --{} not available", &self.option()))
  }
}

pub(crate) fn create_flag(flag_type: &FlagType, subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  match flag_type {
    Actual => actual_flag(subject, long_help),
    All => all_flag(subject, long_help),
    AllocationStatus => allocation_status_flag(subject, long_help),
    Applications => applications_flag(subject, long_help),
    Apps => apps_flag(subject, long_help),
    Configuration => configuration_flag(subject, long_help),
    Ids => ids_flag(subject, long_help),
    Tasks => tasks_flag(subject, long_help),
    Usage => usage_flag(subject, long_help),
    Value => value_flag(subject, long_help),
  }
}

fn actual_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Actual, subject, format!("Show actual {} configuration.", subject.subject()).as_str(), long_help)
}

fn all_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(All, subject, format!("Include all {} parameters.", subject.subject()).as_str(), long_help)
}

fn allocation_status_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    AllocationStatus,
    subject,
    format!("Show {}'s allocation status", subject.subject()).as_str(),
    long_help,
  )
}

fn apps_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Apps, subject, format!("Include apps related to this {}.", subject.subject()).as_str(), long_help)
}

fn applications_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    Applications,
    subject,
    format!("Include applications related to this {}.", subject.subject()).as_str(),
    long_help,
  )
}

fn configuration_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    Configuration,
    subject,
    format!("Show {}'s initial configuration.", subject.subject()).as_str(),
    long_help,
  )
}

fn ids_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Ids, subject, format!("Show {}'s ids.", subject.subject()).as_str(), long_help)
}

fn tasks_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Tasks, subject, format!("Show {}'s tasks.", subject.subject()).as_str(), long_help)
}

fn usage_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Usage, subject, format!("Show {}'s usages.", subject.subject()).as_str(), long_help)
}

fn value_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Value, subject, format!("Show {}'s value.", subject.subject()).as_str(), long_help)
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
