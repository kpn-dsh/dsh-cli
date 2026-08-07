use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum FlagType {
  _Actual,
  AllocationStatus,
  AllVersions,
  Apps,
  Bundle,
  Certificates,
  Configuration,
  Errors,
  Ids,
  Issues,
  Keys,
  OpenId,
  Properties,
  #[cfg(feature = "manage")]
  Stream,
  System,
  Usage,
  Value,
}

impl FlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::_Actual => "actual-flag",
      Self::AllocationStatus => "status-flag",
      Self::AllVersions => "all-versions-flag",
      Self::Apps => "apps-flag",
      Self::Bundle => "bundle-flag",
      Self::Certificates => "certificates-flag",
      Self::Configuration => "configuration-flag",
      Self::Errors => "erros-flag",
      Self::Ids => "ids-flag",
      Self::Issues => "issues-flag",
      Self::Keys => "keys-flag",
      Self::OpenId => "open-id-flag",
      Self::Properties => "properties-flag",
      #[cfg(feature = "manage")]
      Self::Stream => "stream-flag",
      Self::System => "system-flag",
      Self::Usage => "usage-flag",
      Self::Value => "value-flag",
    }
  }

  fn flag(&self) -> &'static str {
    match &self {
      Self::_Actual => "actual",
      Self::AllocationStatus => "status",
      Self::AllVersions => "all-versions",
      Self::Apps => "apps",
      Self::Bundle => "bundle",
      Self::Certificates => "certificates",
      Self::Configuration => "configuration",
      Self::Errors => "errors",
      Self::Ids => "ids",
      Self::Issues => "issues",
      Self::Keys => "keys",
      Self::OpenId => "openid",
      Self::Properties => "properties",
      #[cfg(feature = "manage")]
      Self::Stream => "stream",
      Self::System => "system",
      Self::Usage => "usage",
      Self::Value => "value",
    }
  }

  fn alias(&self) -> Option<&'static str> {
    match &self {
      Self::Certificates => Some("certificate"),
      _ => None,
    }
  }
}

pub(crate) fn create_flag(flag_type: &FlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    FlagType::_Actual => create_clap_flag(FlagType::_Actual, format!("Use the 'actual' {} configuration", subject), long_help),
    FlagType::AllocationStatus => create_clap_flag(FlagType::AllocationStatus, format!("Include the {}'s allocation status", subject), long_help),
    FlagType::AllVersions => create_clap_flag(FlagType::AllVersions, format!("List all {} versions", subject), long_help),
    FlagType::Apps => create_clap_flag(FlagType::Apps, "List all apps".to_string(), long_help),
    FlagType::Bundle => create_clap_flag(FlagType::Bundle, format!("List all {}s", subject), long_help),
    FlagType::Certificates => create_clap_flag(FlagType::Certificates, format!("Show {}s as certificates", subject), long_help),
    FlagType::Configuration => create_clap_flag(FlagType::Configuration, format!("Include the {}'s initial configuration", subject), long_help),
    FlagType::Errors => create_clap_flag(FlagType::Errors, format!("Check {}s and show errors", subject), long_help),
    FlagType::Ids => create_clap_flag(FlagType::Ids, format!("Include the {}'s ids", subject), long_help),
    FlagType::Issues => create_clap_flag(FlagType::Issues, format!("Check {}s and show issues", subject), long_help),
    FlagType::Keys => create_clap_flag(FlagType::Keys, format!("Show {}s as a keys", subject), long_help),
    FlagType::OpenId => create_clap_flag(FlagType::OpenId, "Show openid refresh token".to_string(), long_help),
    FlagType::Properties => create_clap_flag(FlagType::Properties, format!("Include the {}'s properties", subject), long_help),
    #[cfg(feature = "manage")]
    FlagType::Stream => create_clap_flag(FlagType::Stream, format!("Include the {}'s stream", subject), long_help),
    FlagType::System => create_clap_flag(FlagType::System, format!("Include the system {}'s", subject), long_help),
    FlagType::Usage => create_clap_flag(FlagType::Usage, format!("Include the {}'s usages", subject), long_help),
    FlagType::Value => create_clap_flag(FlagType::Value, format!("Include the {}'s value", subject), long_help),
  }
}

fn create_clap_flag(flag_type: FlagType, help: String, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.flag()).action(ArgAction::SetTrue).help(help);
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  if let Some(alias) = flag_type.alias() {
    flag_arg = flag_arg.alias(alias.to_string());
  }
  flag_arg
}
