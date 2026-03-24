use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum ModifierFlagType {
  IgnoreCase,
  ImplicitDefaults,
  MultiLine,
  Regex,
  Substring,
}

impl ModifierFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::IgnoreCase => "ignore-case-flag",
      Self::ImplicitDefaults => "implicit-defaults-flag",
      Self::MultiLine => "multi-line-flag",
      Self::Regex => "regex-flag",
      Self::Substring => "substring-flag",
    }
  }

  fn flag(&self) -> &'static str {
    match &self {
      Self::IgnoreCase => "ignore-case",
      Self::ImplicitDefaults => "implicit-defaults",
      Self::MultiLine => "multi-line",
      Self::Regex => "regex",
      Self::Substring => "substring",
    }
  }

  fn shortcut(&self) -> Option<char> {
    match &self {
      Self::IgnoreCase => Some('i'),
      Self::ImplicitDefaults => None,
      Self::MultiLine => Some('m'),
      Self::Regex => Some('r'),
      Self::Substring => None,
    }
  }
}

pub(crate) fn create_modifier_flag(flag_type: &ModifierFlagType, subject: &str) -> Arg {
  match flag_type {
    ModifierFlagType::IgnoreCase => create_clap_modifier_flag(
      ModifierFlagType::IgnoreCase,
      "Ignore case when matching values".to_string(),
      "If this flag is provided matching will ignore case.".to_string(),
    ),
    ModifierFlagType::ImplicitDefaults => create_clap_modifier_flag(
      ModifierFlagType::ImplicitDefaults,
      "Implicitly use default values for optional parameters".to_string(),
      "If this flag is provided the user will not be prompted for optional parameters. Instead the default value will be used.".to_string(),
    ),
    ModifierFlagType::MultiLine => create_clap_modifier_flag(
      ModifierFlagType::MultiLine,
      format!("Enter {} as multi-line string", subject),
      format!("Enter the {} as a multi-line string. Terminate the input with ctrl-d after last line.", subject),
    ),
    ModifierFlagType::Regex => create_clap_modifier_flag(
      ModifierFlagType::Regex,
      format!("Query string is regular expression matching {} value", subject),
      format!(
        "Interpret the query string as a regular expression instead of an exact matching {} value. \
         The regular expression syntax is described on \
         the following web-page: https://docs.rs/regex/latest/regex/#syntax.",
        subject
      ),
    ),
    ModifierFlagType::Substring => create_clap_modifier_flag(
      ModifierFlagType::Substring,
      "Match substrings".to_string(),
      "If this flag is provided substring matching will be used.".to_string(),
    ),
  }
}

fn create_clap_modifier_flag(flag_type: ModifierFlagType, help: String, long_help: String) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id())
    .long(flag_type.flag())
    .action(ArgAction::SetTrue)
    .help(help)
    .long_help(long_help);
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  flag_arg
}
