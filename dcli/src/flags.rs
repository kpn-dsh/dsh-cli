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
  MultiLine,
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
      MultiLine => "multi-line-flag",
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
      MultiLine => "multi-line",
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
      MultiLine => Some('m'),
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
    MultiLine => multi_line_flag(subject, long_help),
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
    format!("Show {}'s initial configuration.", subject.subject()).as_str(),
    long_help,
  )
}

fn ids_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Ids, subject, format!("Show {}'s ids.", subject.subject()).as_str(), long_help)
}

fn json_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(Json, subject, format!("Show {} as json.", subject.subject()).as_str(), long_help)
}

fn multi_line_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_flag(MultiLine, subject, format!("Enter {} as multi-line string.", subject.subject()).as_str(), long_help)
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
