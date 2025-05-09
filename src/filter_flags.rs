use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  App,
  Complete,
  #[cfg(feature = "manage")]
  Internal,
  #[cfg(feature = "manage")]
  Public,
  Service,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::App => "app-flag",
      Self::Complete => "complete-flag",
      #[cfg(feature = "manage")]
      Self::Internal => "internal-flag",
      #[cfg(feature = "manage")]
      Self::Public => "public-flag",
      Self::Service => "service-flag",
      Self::Started => "started-flag",
      Self::Stopped => "stopped-flag",
    }
  }

  fn option(&self) -> &'static str {
    match &self {
      Self::App => "app",
      Self::Complete => "complete",
      #[cfg(feature = "manage")]
      Self::Internal => "internal",
      #[cfg(feature = "manage")]
      Self::Public => "public",
      Self::Service => "service",
      Self::Started => "started",
      Self::Stopped => "stopped",
    }
  }

  fn help(&self) -> &'static str {
    match &self {
      Self::App => "Include apps",
      Self::Complete => "Include all parameters",
      #[cfg(feature = "manage")]
      Self::Internal => "Include internal streams",
      #[cfg(feature = "manage")]
      Self::Public => "Include public streams",
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
