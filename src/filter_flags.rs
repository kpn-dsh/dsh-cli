use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  Complete,
  Draft,
  #[cfg(feature = "manage")]
  Internal,
  #[cfg(feature = "manage")]
  Public,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::Complete => "complete-flag",
      Self::Draft => "draft-flag",
      #[cfg(feature = "manage")]
      Self::Internal => "internal-flag",
      #[cfg(feature = "manage")]
      Self::Public => "public-flag",
      Self::Started => "started-flag",
      Self::Stopped => "stopped-flag",
    }
  }

  fn option(&self) -> &'static str {
    match &self {
      Self::Complete => "complete",
      Self::Draft => "draft",
      #[cfg(feature = "manage")]
      Self::Internal => "internal",
      #[cfg(feature = "manage")]
      Self::Public => "public",
      Self::Started => "started",
      Self::Stopped => "stopped",
    }
  }

  fn help(&self) -> &'static str {
    match &self {
      Self::Complete => "Include all parameters",
      Self::Draft => "Include draft versions",
      #[cfg(feature = "manage")]
      Self::Internal => "Include internal streams",
      #[cfg(feature = "manage")]
      Self::Public => "Include public streams",
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
