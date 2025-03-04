use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  App,
  _Killed,
  Service,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::App => "app-flag",
      Self::_Killed => "killed-flag",
      Self::Service => "service-flag",
      Self::Started => "started-flag",
      Self::Stopped => "stopped-flag",
    }
  }

  fn option(&self) -> &'static str {
    match &self {
      Self::App => "app",
      Self::_Killed => "killed",
      Self::Service => "service",
      Self::Started => "started",
      Self::Stopped => "stopped",
    }
  }

  fn shortcut(&self) -> Option<char> {
    match &self {
      Self::App => None,
      Self::_Killed => None,
      Self::Service => None,
      Self::Started => None,
      Self::Stopped => None,
    }
  }
}

pub(crate) fn create_filter_flag(flag_type: &FilterFlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    FilterFlagType::App => create_clap_filter_flag(FilterFlagType::App, subject, format!("Include apps related to this {}.", subject), long_help),
    FilterFlagType::_Killed => create_clap_filter_flag(FilterFlagType::_Killed, subject, format!("Include killed {}'s.", subject), long_help),
    FilterFlagType::Service => create_clap_filter_flag(
      FilterFlagType::Service,
      subject,
      format!("Include services related to this {}.", subject),
      long_help,
    ),
    FilterFlagType::Started => create_clap_filter_flag(FilterFlagType::Started, subject, format!("Include started {}'s.", subject), long_help),
    FilterFlagType::Stopped => create_clap_filter_flag(FilterFlagType::Stopped, subject, format!("Include stopped {}'s.", subject), long_help),
  }
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
