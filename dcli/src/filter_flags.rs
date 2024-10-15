use clap::{Arg, ArgAction};

use crate::filter_flags::FilterFlagType::*;
use crate::subject::Subject;

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  App,
  Application,
  Killed,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      App => "app-flag",
      Application => "application-flag",
      Killed => "killed-flag",
      Started => "started-flag",
      Stopped => "stopped-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      App => "app",
      Application => "application",
      Killed => "killed",
      Started => "started",
      Stopped => "stopped",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      App => None,
      Application => None,
      Killed => None,
      Started => None,
      Stopped => None,
    }
  }
}

pub(crate) fn create_filter_flag(flag_type: &FilterFlagType, subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  match flag_type {
    App => app_flag(subject, long_help),
    Application => application_flag(subject, long_help),
    Killed => killed_flag(subject, long_help),
    Started => started_flag(subject, long_help),
    Stopped => stopped_flag(subject, long_help),
  }
}

fn app_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_filter_flag(App, subject, format!("Include apps related to this {}.", subject.subject()).as_str(), long_help)
}

fn application_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_filter_flag(
    Application,
    subject,
    format!("Include applications related to this {}.", subject.subject()).as_str(),
    long_help,
  )
}

fn killed_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_filter_flag(Killed, subject, format!("Include killed {}'s.", subject.subject()).as_str(), long_help)
}

fn started_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_filter_flag(Started, subject, format!("Include started {}'s.", subject.subject()).as_str(), long_help)
}

fn stopped_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_filter_flag(Stopped, subject, format!("Include stopped {}'s.", subject.subject()).as_str(), long_help)
}

fn create_clap_filter_flag(flag_type: FilterFlagType, _subject: &dyn Subject, help: &str, long_help: &Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help.to_string());
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
