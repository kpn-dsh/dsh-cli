use clap::{Arg, ArgAction};

use crate::filter_flags::FilterFlagType::*;

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  App,
  Killed,
  Service,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      App => "app-flag",
      Killed => "killed-flag",
      Service => "service-flag",
      Started => "started-flag",
      Stopped => "stopped-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      App => "app",
      Killed => "killed",
      Service => "service",
      Started => "started",
      Stopped => "stopped",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      App => None,
      Killed => None,
      Service => None,
      Started => None,
      Stopped => None,
    }
  }
}

pub(crate) fn create_filter_flag(flag_type: &FilterFlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    App => app_flag(subject, long_help),
    Killed => killed_flag(subject, long_help),
    Service => service_flag(subject, long_help),
    Started => started_flag(subject, long_help),
    Stopped => stopped_flag(subject, long_help),
  }
}

fn app_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_filter_flag(App, subject, format!("Include apps related to this {}.", subject), long_help)
}

fn killed_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_filter_flag(Killed, subject, format!("Include killed {}'s.", subject), long_help)
}

fn service_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_filter_flag(Service, subject, format!("Include services related to this {}.", subject), long_help)
}

fn started_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_filter_flag(Started, subject, format!("Include started {}'s.", subject), long_help)
}

fn stopped_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_filter_flag(Stopped, subject, format!("Include stopped {}'s.", subject), long_help)
}

fn create_clap_filter_flag(flag_type: FilterFlagType, _subject: &str, help: String, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help);
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
