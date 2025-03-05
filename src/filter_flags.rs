use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  App,
  Service,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::App => "app-flag",
      Self::Service => "service-flag",
      Self::Started => "started-flag",
      Self::Stopped => "stopped-flag",
    }
  }

  fn option(&self) -> &'static str {
    match &self {
      Self::App => "app",
      Self::Service => "service",
      Self::Started => "started",
      Self::Stopped => "stopped",
    }
  }

  fn help(&self) -> &'static str {
    match &self {
      Self::App => "Include apps",
      Self::Service => "Include services",
      Self::Started => "Include only started apps/services",
      Self::Stopped => "Include only stopped apps/services",
    }
  }
}

pub(crate) fn create_filter_flag(flag_type: &FilterFlagType, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(flag_type.help());
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
