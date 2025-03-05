use clap::{Arg, ArgAction};

#[derive(Debug)]
pub(crate) enum ModifierFlagType {
  MultiLine,
  Regex,
}

impl ModifierFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Self::MultiLine => "multi-line-flag",
      Self::Regex => "regex-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      Self::MultiLine => "multi-line",
      Self::Regex => "regex",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      Self::MultiLine => Some('m'),
      Self::Regex => Some('r'),
    }
  }
}

pub(crate) fn create_modifier_flag(flag_type: &ModifierFlagType, subject: &str) -> Arg {
  match flag_type {
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
  }
}

fn create_clap_modifier_flag(flag_type: ModifierFlagType, help: String, long_help: String) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id())
    .long(flag_type.option())
    .action(ArgAction::SetTrue)
    .help(help)
    .long_help(long_help);
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  flag_arg
}
