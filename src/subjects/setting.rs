use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::settings::Settings;
use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::ENVIRONMENT_VARIABLE_LABELS;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::settings::read_settings;
use crate::subject::{Requirements, Subject};
use crate::{get_environment_variables, DshCliResult, ENV_VAR_PASSWORD};

pub(crate) struct SettingSubject {}

const SETTING_SUBJECT_TARGET: &str = "setting";

lazy_static! {
  pub static ref SETTING_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SettingSubject {});
}

#[async_trait]
impl Subject for SettingSubject {
  fn subject(&self) -> &'static str {
    SETTING_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list dsh settings.".to_string()
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(false, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(SETTING_LIST_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &SETTING_CAPABILITIES
  }
}

lazy_static! {
  static ref SETTING_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List settings")
      .set_long_about("Lists all dsh settings.")
      .set_default_command_executor(&SettingList {})
  );
  static ref SETTING_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![SETTING_LIST_CAPABILITY.as_ref()];
}

struct SettingList {}

#[async_trait]
impl CommandExecutor for SettingList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let password = "********".to_string();
    context.print_explanation("list default settings");
    match read_settings(None)? {
      Some(settings) => UnitFormatter::new("value", &SETTING_LABELS, Some("setting"), &settings, context).print()?,
      None => context.print_outcome("no default settings found"),
    }
    let env_vars = get_environment_variables();
    if !env_vars.is_empty() {
      context.print_explanation("list environment variables");
      let mut formatter = ListFormatter::new(&ENVIRONMENT_VARIABLE_LABELS, None, context);
      for (env_var, value) in &env_vars {
        if env_var == ENV_VAR_PASSWORD {
          formatter.push_target_id_value(env_var.clone(), &password);
        } else {
          formatter.push_target_id_value(env_var.clone(), value);
        }
      }
      formatter.print()?;
    }
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum SettingLabel {
  CsvQuote,
  CsvSeparator,
  DefaultPlatform,
  DefaultTenant,
  DryRun,
  FileName,
  MatchingStyle,
  NoEscape,
  OutputFormat,
  Quiet,
  ShowExecutionTime,
  Target,
  TerminalWidth,
  Verbosity,
}

impl Label for SettingLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DefaultPlatform => "default platform",
      Self::DefaultTenant => "default tenant",
      Self::FileName => "settings file name",
      Self::ShowExecutionTime => "show execution time",
      Self::Target => "setting",
      Self::Verbosity => "verbosity",
      Self::CsvQuote => "csv quote character",
      Self::CsvSeparator => "csv separator",
      Self::DryRun => "dry run mode",
      Self::MatchingStyle => "matching style",
      Self::NoEscape => "no-escape",
      Self::OutputFormat => "output format",
      Self::Quiet => "quiet mode",
      Self::TerminalWidth => "terminal width",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<SettingLabel> for Settings {
  fn value(&self, label: &SettingLabel, target_id: &str) -> String {
    match label {
      SettingLabel::DefaultPlatform => self.default_platform.clone().unwrap_or_default(),
      SettingLabel::DefaultTenant => self.default_tenant.clone().unwrap_or_default(),
      SettingLabel::FileName => self.file_name.clone().unwrap_or_default(),
      SettingLabel::ShowExecutionTime => self
        .show_execution_time
        .map(|show_execution_time| show_execution_time.to_string())
        .unwrap_or_default(),
      SettingLabel::Target => target_id.to_string(),
      SettingLabel::Verbosity => self.verbosity.clone().map(|verbosity| verbosity.to_string()).unwrap_or_default(),
      SettingLabel::CsvQuote => self.csv_quote.map(|csv_quote| csv_quote.to_string()).unwrap_or_default(),
      SettingLabel::CsvSeparator => self.csv_separator.clone().unwrap_or_default(),
      SettingLabel::DryRun => self.dry_run.map(|dry_run| dry_run.to_string()).unwrap_or_default(),
      SettingLabel::MatchingStyle => self.matching_style.clone().map(|style| style.to_string()).unwrap_or_default(),
      SettingLabel::NoEscape => self.no_escape.map(|no_escape| no_escape.to_string()).unwrap_or_default(),
      SettingLabel::OutputFormat => self.output_format.clone().map(|format| format.to_string()).unwrap_or_default(),
      SettingLabel::Quiet => self.quiet.map(|quiet| quiet.to_string()).unwrap_or_default(),
      SettingLabel::TerminalWidth => self.terminal_width.map(|width| width.to_string()).unwrap_or_default(),
    }
  }

  fn target_label(&self) -> Option<SettingLabel> {
    Some(SettingLabel::Target)
  }
}

pub static SETTING_LABELS: [SettingLabel; 14] = [
  SettingLabel::CsvQuote,
  SettingLabel::CsvSeparator,
  SettingLabel::DefaultPlatform,
  SettingLabel::DefaultTenant,
  SettingLabel::DryRun,
  SettingLabel::MatchingStyle,
  SettingLabel::NoEscape,
  SettingLabel::OutputFormat,
  SettingLabel::Quiet,
  SettingLabel::ShowExecutionTime,
  SettingLabel::Target,
  SettingLabel::TerminalWidth,
  SettingLabel::Verbosity,
  SettingLabel::FileName,
];
