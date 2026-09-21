use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FilterFlagType {
  Complete,
  Draft,
  Internal,
  Public,
  Started,
  Stopped,
}

impl FilterFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::Complete => "complete-flag",
      Self::Draft => "draft-flag",
      Self::Internal => "internal-flag",
      Self::Public => "public-flag",
      Self::Started => "started-flag",
      Self::Stopped => "stopped-flag",
    }
  }

  fn flag(&self) -> &'static str {
    match &self {
      Self::Complete => "complete",
      Self::Draft => "draft",
      Self::Internal => "internal",
      Self::Public => "public",
      Self::Started => "started",
      Self::Stopped => "stopped",
    }
  }

  fn help(&self) -> &'static str {
    match &self {
      Self::Complete => "Include all parameters",
      Self::Draft => "Include draft versions",
      Self::Internal => "Include internal streams",
      Self::Public => "Include public streams",
      Self::Started => "Include only started apps/services",
      Self::Stopped => "Include only stopped apps/services",
    }
  }
}

pub(crate) fn create_filter_flag(flag_type: &FilterFlagType, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.flag()).action(ArgAction::SetTrue).help(flag_type.help());
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
