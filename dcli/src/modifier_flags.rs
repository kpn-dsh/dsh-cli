use clap::{Arg, ArgAction};

use crate::modifier_flags::ModifierFlagType::*;

#[derive(Debug)]
pub(crate) enum ModifierFlagType {
  Json,
  MultiLine,
  Regex,
}

impl ModifierFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Json => "json-flag",
      MultiLine => "multi-line-flag",
      Regex => "regex-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      Json => "json",
      MultiLine => "multi-line",
      Regex => "regex",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      Json => Some('j'),
      MultiLine => Some('m'),
      Regex => Some('r'),
    }
  }
}

pub(crate) fn create_modifier_flag(flag_type: &ModifierFlagType, subject: &str, long_help: Option<&str>) -> Arg {
  match flag_type {
    Json => json_flag(subject, long_help),
    MultiLine => multi_line_flag(subject, long_help),
    Regex => regex_flag(subject, long_help),
  }
}

fn json_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_modifier_flag(Json, subject, format!("Show the {} as json.", subject), long_help)
}

fn multi_line_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_modifier_flag(MultiLine, subject, format!("Enter the {} as multi-line string.", subject), long_help)
}

fn regex_flag(subject: &str, long_help: Option<&str>) -> Arg {
  create_clap_modifier_flag(
    Regex,
    subject,
    format!("Interpret the query string as a regular expression instead of an exact matching {} value. The regular expression syntax is described on the following web-page: https://docs.rs/regex/latest/regex/#syntax.", subject),
    long_help,
  )
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
