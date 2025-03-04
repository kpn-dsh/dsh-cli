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

pub(crate) fn create_modifier_flag(flag_type: &ModifierFlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    ModifierFlagType::MultiLine => create_clap_modifier_flag(
      ModifierFlagType::MultiLine,
      subject,
      format!("Enter the {} as multi-line string.", subject),
      long_help,
    ),
    ModifierFlagType::Regex => create_clap_modifier_flag(
      ModifierFlagType::Regex,
      subject,
      format!(
        "Interpret the query string as a regular expression instead of an exact matching {} value. \
         The regular expression syntax is described on \
         the following web-page: https://docs.rs/regex/latest/regex/#syntax.",
        subject
      ),
      long_help,
    ),
  }
}

fn create_clap_modifier_flag(flag_type: ModifierFlagType, _subject: &str, help: String, long_help: Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help.to_string());
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
