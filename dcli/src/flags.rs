use clap::{Arg, ArgAction};

use crate::flags::FlagType::*;
use crate::subject::Subject;

#[derive(Debug)]
pub(crate) enum FlagType {
  Actual,
  All,
  AllocationStatus,
  App,
  Application,
  Configuration,
  Ids,
  Json,
  Killed,
  MultiLine,
  Started,
  Stopped,
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
      App => "app-flag",
      Application => "application-flag",
      Configuration => "configuration-flag",
      Ids => "ids-flag",
      Json => "json-flag",
      Killed => "killed-flag",
      MultiLine => "multi-line-flag",
      Started => "started-flag",
      Stopped => "stopped-flag",
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
      App => "app",
      Application => "application",
      Configuration => "configuration",
      Ids => "ids",
      Json => "json",
      Killed => "killed",
      MultiLine => "multi-line",
      Started => "started",
      Stopped => "stopped",
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
      App => None,
      Application => None,
      Configuration => Some('c'),
      Ids => Some('i'),
      Json => Some('j'),
      Killed => None,
      MultiLine => Some('m'),
      Started => None,
      Stopped => None,
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
    App => app_flag(subject, long_help),
    Application => application_flag(subject, long_help),
    Configuration => configuration_flag(subject, long_help),
    Ids => ids_flag(subject, long_help),
    Json => json_flag(subject, long_help),
    Killed => killed_flag(subject, long_help),
    MultiLine => multi_line_flag(subject, long_help),
    Started => started_flag(subject, long_help),
    Stopped => stopped_flag(subject, long_help),
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

fn app_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(App, subject, format!("Include apps related to this {}.", subject.subject()).as_str(), long_help)
}

fn application_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    Application,
    subject,
    format!("Include applications related to this {}.", subject.subject()).as_str(),
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

fn json_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Json, subject, format!("Show the {} as json.", subject.subject()).as_str(), long_help)
}

fn killed_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Killed, subject, format!("Include killed {}'s.", subject.subject()).as_str(), long_help)
}

fn multi_line_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(
    MultiLine,
    subject,
    format!("Enter the {} as multi-line string.", subject.subject()).as_str(),
    long_help,
  )
}

fn started_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Started, subject, format!("Include started {}'s.", subject.subject()).as_str(), long_help)
}

fn stopped_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Stopped, subject, format!("Include stopped {}'s.", subject.subject()).as_str(), long_help)
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
