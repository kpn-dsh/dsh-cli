use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FlagType {
  _Actual,
  AllocationStatus,
  Configuration,
  Ids,
  Properties,
  #[cfg(feature = "manage")]
  Stream,
  System,
  Tasks,
  Usage,
  Value,
}

impl FlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::_Actual => "actual-flag",
      Self::AllocationStatus => "status-flag",
      Self::Configuration => "configuration-flag",
      Self::Ids => "ids-flag",
      Self::Properties => "properties-flag",
      #[cfg(feature = "manage")]
      Self::Stream => "stream-flag",
      Self::System => "system-flag",
      Self::Tasks => "tasks-flag",
      Self::Usage => "usage-flag",
      Self::Value => "value-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      Self::_Actual => "actual",
      Self::AllocationStatus => "status",
      Self::Configuration => "configuration",
      Self::Ids => "ids",
      Self::Properties => "properties",
      #[cfg(feature = "manage")]
      Self::Stream => "stream",
      Self::System => "system",
      Self::Tasks => "tasks",
      Self::Usage => "usage",
      Self::Value => "value",
    }
  }
}

pub(crate) fn create_flag(flag_type: &FlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    FlagType::_Actual => create_clap_flag(FlagType::_Actual, format!("Use the 'actual' {} configuration", subject), long_help),
    FlagType::AllocationStatus => create_clap_flag(FlagType::AllocationStatus, format!("Include the {}'s allocation status", subject), long_help),
    FlagType::Configuration => create_clap_flag(FlagType::Configuration, format!("Include the {}'s initial configuration", subject), long_help),
    FlagType::Ids => create_clap_flag(FlagType::Ids, format!("Include the {}'s ids", subject), long_help),
    FlagType::Properties => create_clap_flag(FlagType::Properties, format!("Include the {}'s properties", subject), long_help),
    #[cfg(feature = "manage")]
    FlagType::Stream => create_clap_flag(FlagType::Stream, format!("Include the {}'s stream", subject), long_help),
    FlagType::System => create_clap_flag(FlagType::System, format!("Include the system {}'s", subject), long_help),
    FlagType::Tasks => create_clap_flag(FlagType::Tasks, format!("Include the {}'s tasks", subject), long_help),
    FlagType::Usage => create_clap_flag(FlagType::Usage, format!("Include the {}'s usages", subject), long_help),
    FlagType::Value => create_clap_flag(FlagType::Value, format!("Include the {}'s value", subject), long_help),
  }
}

fn create_clap_flag(flag_type: FlagType, help: String, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help);
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
