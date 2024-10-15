use clap::{Arg, ArgAction};

use crate::modifier_flags::ModifierFlagType::*;
use crate::subject::Subject;

#[derive(Debug)]
pub(crate) enum ModifierFlagType {
  Json,
  MultiLine,
}

impl ModifierFlagType {
  pub(crate) fn id(&self) -> &'static str {
    match &self {
      Json => "json-flag",
      MultiLine => "multi-line-flag",
    }
  }

  pub(crate) fn option(&self) -> &'static str {
    match &self {
      Json => "json",
      MultiLine => "multi-line",
    }
  }

  pub(crate) fn shortcut(&self) -> Option<char> {
    match &self {
      Json => Some('j'),
      MultiLine => Some('m'),
    }
  }
}

pub(crate) fn create_modifier_flag(flag_type: &ModifierFlagType, subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  match flag_type {
    Json => json_flag(subject, long_help),
    MultiLine => multi_line_flag(subject, long_help),
  }
}

fn json_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_modifier_flag(Json, subject, format!("Show the {} as json.", subject.subject()).as_str(), long_help)
}

fn multi_line_flag(subject: &dyn Subject, long_help: &Option<&str>) -> Arg {
  create_clap_modifier_flag(
    MultiLine,
    subject,
    format!("Enter the {} as multi-line string.", subject.subject()).as_str(),
    long_help,
  )
}

fn create_clap_modifier_flag(flag_type: ModifierFlagType, _subject: &dyn Subject, help: &str, long_help: &Option<&str>) -> Arg {
  let mut flag_arg = Arg::new(flag_type.id()).long(flag_type.option()).action(ArgAction::SetTrue).help(help.to_string());
  if let Some(shortcut) = flag_type.shortcut() {
    flag_arg = flag_arg.short(shortcut)
  }
  if let Some(long_help) = long_help {
    flag_arg = flag_arg.long_help(long_help.to_string());
  }
  flag_arg
}
