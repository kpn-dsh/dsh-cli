use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::settings::{upsert_settings, Settings};
use async_trait::async_trait;
use clap::builder::EnumValueParser;
use clap::{builder, Arg, ArgAction, ArgMatches, Command};
use dsh_api::platform::DshPlatform;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR, SET_COMMAND, SET_COMMAND_PAIR, UNSET_COMMAND, UNSET_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::{Context, MatchingStyle};
use crate::formatters::formatter::ENVIRONMENT_VARIABLE_LABELS;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::log_level::LogLevel;
use crate::settings::get_settings;
use crate::subject::{Requirements, Subject};
use crate::verbosity::Verbosity;
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
    Requirements::new(false, false, false, Some(OutputFormat::Table))
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(SETTING_LIST_CAPABILITY.as_ref()),
      SET_COMMAND => Some(SETTING_SETTING_CAPABILITY.as_ref()),
      UNSET_COMMAND => Some(SETTING_UNSETTING_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &SETTING_CAPABILITIES
  }
}

const SETTING_CSV_QUOTE: &str = "csv-quote";
const SETTING_CSV_SEPARATOR: &str = "csv-separator";
const SETTING_DEFAULT_PLATFORM: &str = "default-platform";
const SETTING_DEFAULT_TENANT: &str = "default-tenant";
const SETTING_DRY_RUN: &str = "dry-run";
const SETTING_LOG_LEVEL: &str = "log-level";
const SETTING_LOG_LEVEL_API: &str = "log-level-api";
const SETTING_LOG_LEVEL_SDK: &str = "log-level-sdk";
const SETTING_MATCHING_STYLE: &str = "matching-style";
const SETTING_NO_ESCAPE: &str = "no-escape";
const SETTING_NO_HEADERS: &str = "no-headers";
const SETTING_OUTPUT_FORMAT: &str = "output-format";
const SETTING_QUIET: &str = "quiet";
const SETTING_SHOW_EXECUTION_TIME: &str = "show-execution-time";
const SETTING_TERMINAL_WIDTH: &str = "terminal-width";
const SETTING_VERBOSITY: &str = "verbosity";

fn set_unset_commands(required: bool) -> Vec<Command> {
  vec![
    Command::new(SETTING_CSV_QUOTE)
      .arg(
        Arg::new(SETTING_CSV_QUOTE)
          .action(ArgAction::Set)
          .value_parser(builder::NonEmptyStringValueParser::new())
          .required(required),
      )
      .about("Character used to quote values for the csv output format"),
    Command::new(SETTING_CSV_SEPARATOR)
      .arg(
        Arg::new(SETTING_CSV_SEPARATOR)
          .action(ArgAction::Set)
          .value_parser(builder::NonEmptyStringValueParser::new())
          .required(required),
      )
      .about("Character used to separate values for the csv output format"),
    Command::new(SETTING_DEFAULT_PLATFORM)
      .arg(
        Arg::new(SETTING_DEFAULT_PLATFORM)
          .action(ArgAction::Set)
          .value_parser(builder::NonEmptyStringValueParser::new())
          .required(required),
      )
      .about("Default target platform, used for authentication and authorization"),
    Command::new(SETTING_DEFAULT_TENANT)
      .arg(
        Arg::new(SETTING_DEFAULT_TENANT)
          .action(ArgAction::Set)
          .value_parser(builder::NonEmptyStringValueParser::new())
          .required(required),
      )
      .about("Default target tenant, used for authentication and authorization"),
    Command::new(SETTING_DRY_RUN).about("Inhibits any changes to the platform"),
    Command::new(SETTING_LOG_LEVEL)
      .arg(
        Arg::new(SETTING_LOG_LEVEL)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<LogLevel>::new())
          .required(required),
      )
      .about("Log level for the dsh cli tool"),
    Command::new(SETTING_LOG_LEVEL_API)
      .arg(
        Arg::new(SETTING_LOG_LEVEL_API)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<LogLevel>::new())
          .required(required),
      )
      .about("Log level for the 'dsh_api' library functions"),
    Command::new(SETTING_LOG_LEVEL_SDK)
      .arg(
        Arg::new(SETTING_LOG_LEVEL_SDK)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<LogLevel>::new())
          .required(required),
      )
      .about("Log level for the 'dsh_sdk' library functions"),
    Command::new(SETTING_MATCHING_STYLE)
      .arg(
        Arg::new(SETTING_MATCHING_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<MatchingStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing matching results for the find functions"),
    Command::new(SETTING_NO_ESCAPE).about("Inhibits any color or other ansi escape sequences"),
    Command::new(SETTING_NO_HEADERS).about("Disables headers in the output"),
    Command::new(SETTING_OUTPUT_FORMAT)
      .arg(
        Arg::new(SETTING_OUTPUT_FORMAT)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<OutputFormat>::new())
          .required(required),
      )
      .about("Default/preferred format used when printing the output"),
    Command::new(SETTING_QUIET).about("Run in quiet mode"),
    Command::new(SETTING_SHOW_EXECUTION_TIME)
      .arg(
        Arg::new(SETTING_SHOW_EXECUTION_TIME)
          .action(ArgAction::Set)
          .value_parser(builder::BoolValueParser::new()),
      )
      .about("Enables display of the execution time of the executed DSH api functions in milliseconds"),
    Command::new(SETTING_TERMINAL_WIDTH)
      .arg(
        Arg::new(SETTING_TERMINAL_WIDTH)
          .action(ArgAction::Set)
          .value_parser(builder::RangedU64ValueParser::<usize>::from(40..))
          .required(required),
      )
      .about("Maximum terminal width"),
    Command::new(SETTING_VERBOSITY)
      .arg(
        Arg::new(SETTING_VERBOSITY)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<Verbosity>::new())
          .required(required),
      )
      .about("Default verbosity level"),
  ]
}

lazy_static! {
  static ref SETTING_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List settings")
      .set_long_about("Lists all dsh settings.")
      .set_default_command_executor(&SettingList {})
  );
  static ref SETTING_SETTING_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SET_COMMAND_PAIR, "Set setting")
      .set_long_about("Set value to persistent storage.")
      .add_subcommands(set_unset_commands(true))
      .set_default_command_executor(&SettingSet {})
  );
  static ref SETTING_UNSETTING_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UNSET_COMMAND_PAIR, "Unset setting")
      .set_long_about("Unset value from persistent storage.")
      .add_subcommands(set_unset_commands(false))
      .set_default_command_executor(&SettingUnset {})
  );
  static ref SETTING_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![SETTING_LIST_CAPABILITY.as_ref(), SETTING_SETTING_CAPABILITY.as_ref(), SETTING_UNSETTING_CAPABILITY.as_ref()];
}

struct SettingList {}

const HIDE_PASSWORD: &str = "********";

#[async_trait]
impl CommandExecutor for SettingList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let (settings, _) = get_settings(None)?;
    if let Some(ref settings_file) = settings.file_name {
      context.print_explanation(format!("list settings from settings file '{}'", settings_file));
      UnitFormatter::new("value", &SETTING_LABELS, Some("setting"), context).print(&settings)?
    } else {
      context.print_explanation("list default settings");
      UnitFormatter::new("value", &SETTING_LABELS, Some("setting"), context).print(&settings)?
    }
    let env_vars = get_environment_variables();
    if !env_vars.is_empty() {
      context.print_explanation("list environment variables");
      let mut formatter = ListFormatter::new(&ENVIRONMENT_VARIABLE_LABELS, None, context);
      let hide_password = HIDE_PASSWORD.to_string();
      for (env_var, value) in &env_vars {
        if env_var == ENV_VAR_PASSWORD {
          formatter.push_target_id_value(env_var.clone(), &hide_password);
        } else {
          formatter.push_target_id_value(env_var.clone(), value);
        }
      }
      formatter.print()?;
    }
    Ok(())
  }
}

struct SettingSet {}

#[async_trait]
impl CommandExecutor for SettingSet {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (target_setting, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    match target_setting {
      SETTING_CSV_QUOTE => {
        let mut csv_quote_chars = matches.get_one::<String>(SETTING_CSV_QUOTE).unwrap().chars();
        let csv_quote = csv_quote_chars.next().unwrap();
        if csv_quote_chars.next().is_some() {
          return Err("csv quote must be a single character".to_string());
        } else {
          upsert_settings(None, |settings| Ok(Settings { csv_quote: Some(csv_quote), ..settings }))?;
          context.print_outcome(format!("csv quote character set to '{}'", csv_quote));
        }
      }
      SETTING_CSV_SEPARATOR => {
        let csv_separator = matches.get_one::<String>(SETTING_CSV_SEPARATOR).unwrap();
        upsert_settings(None, |settings| Ok(Settings { csv_separator: Some(csv_separator.to_string()), ..settings }))?;
        context.print_outcome(format!("csv separator set to \"{}\"", csv_separator));
      }
      SETTING_DEFAULT_PLATFORM => {
        let platform = DshPlatform::try_from(matches.get_one::<String>(SETTING_DEFAULT_PLATFORM).unwrap().as_str())?;
        upsert_settings(None, |settings| Ok(Settings { default_platform: Some(platform.to_string()), ..settings }))?;
        context.print_outcome(format!("default platform set to {}", platform));
      }
      SETTING_DEFAULT_TENANT => {
        let tenant = matches.get_one::<String>(SETTING_DEFAULT_TENANT).unwrap();
        upsert_settings(None, |settings| Ok(Settings { default_tenant: Some(tenant.to_string()), ..settings }))?;
        context.print_outcome(format!("default tenant set to {}", tenant));
      }
      SETTING_DRY_RUN => {
        upsert_settings(None, |settings| Ok(Settings { dry_run: Some(true), ..settings }))?;
        context.print_outcome("dry run mode enabled");
      }
      SETTING_LOG_LEVEL => {
        let log_level = matches.get_one::<LogLevel>(SETTING_LOG_LEVEL).unwrap();
        upsert_settings(None, |settings| Ok(Settings { log_level: Some(log_level.clone()), ..settings }))?;
        context.print_outcome(format!("log level set to {}", log_level));
      }
      SETTING_LOG_LEVEL_API => {
        let log_level_api = matches.get_one::<LogLevel>(SETTING_LOG_LEVEL_API).unwrap();
        upsert_settings(None, |settings| Ok(Settings { log_level_api: Some(log_level_api.clone()), ..settings }))?;
        context.print_outcome(format!("log level for api set to {}", log_level_api));
      }
      SETTING_LOG_LEVEL_SDK => {
        let log_level_sdk = matches.get_one::<LogLevel>(SETTING_LOG_LEVEL_SDK).unwrap();
        upsert_settings(None, |settings| Ok(Settings { log_level_sdk: Some(log_level_sdk.clone()), ..settings }))?;
        context.print_outcome(format!("log level for sdk set to {}", log_level_sdk));
      }
      SETTING_MATCHING_STYLE => {
        let matching_style = matches.get_one::<MatchingStyle>(SETTING_MATCHING_STYLE).unwrap();
        upsert_settings(None, |settings| Ok(Settings { matching_style: Some(matching_style.clone()), ..settings }))?;
        context.print_outcome(format!("matching style set to {}", matching_style));
      }
      SETTING_NO_ESCAPE => {
        upsert_settings(None, |settings| Ok(Settings { no_escape: Some(true), ..settings }))?;
        context.print_outcome("no escape mode enabled");
      }
      SETTING_NO_HEADERS => {
        upsert_settings(None, |settings| Ok(Settings { no_headers: Some(true), ..settings }))?;
        context.print_outcome("no headers mode enabled");
      }
      SETTING_OUTPUT_FORMAT => {
        let output_format = matches.get_one::<OutputFormat>(SETTING_OUTPUT_FORMAT).unwrap();
        upsert_settings(None, |settings| Ok(Settings { output_format: Some(output_format.clone()), ..settings }))?;
        context.print_outcome(format!("output format set to {}", output_format));
      }
      SETTING_QUIET => {
        upsert_settings(None, |settings| Ok(Settings { quiet: Some(true), ..settings }))?;
        context.print_outcome("quiet mode enabled");
      }
      SETTING_SHOW_EXECUTION_TIME => {
        upsert_settings(None, |settings| Ok(Settings { show_execution_time: Some(true), ..settings }))?;
        context.print_outcome("show execution time enabled");
      }
      SETTING_TERMINAL_WIDTH => {
        let terminal_width = matches.get_one::<usize>(SETTING_TERMINAL_WIDTH).unwrap();
        if *terminal_width < 40 {
          return Err("terminal width must be greater than or equal to 40".to_string());
        } else {
          upsert_settings(None, |settings| Ok(Settings { terminal_width: Some(*terminal_width), ..settings }))?;
          context.print_outcome(format!("terminal width set to {}", terminal_width));
        }
      }
      SETTING_VERBOSITY => {
        let verbosity = matches.get_one::<Verbosity>(SETTING_VERBOSITY).unwrap();
        upsert_settings(None, |settings| Ok(Settings { verbosity: Some(verbosity.clone()), ..settings }))?;
        context.print_outcome(format!("verbosity level set to {}", verbosity));
      }
      _ => unreachable!(),
    }
    Ok(())
  }
}

struct SettingUnset {}

#[async_trait]
impl CommandExecutor for SettingUnset {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (target_setting, _) = matches.subcommand().unwrap_or_else(|| unreachable!());
    match target_setting {
      SETTING_CSV_QUOTE => {
        upsert_settings(None, |settings| Ok(Settings { csv_quote: None, ..settings }))?;
        context.print_outcome("csv quote unset");
      }
      SETTING_CSV_SEPARATOR => {
        upsert_settings(None, |settings| Ok(Settings { csv_separator: None, ..settings }))?;
        context.print_outcome("csv separator unset");
      }
      SETTING_DEFAULT_PLATFORM => {
        upsert_settings(None, |settings| Ok(Settings { default_platform: None, ..settings }))?;
        context.print_outcome("default platform unset");
      }
      SETTING_DEFAULT_TENANT => {
        upsert_settings(None, |settings| Ok(Settings { default_tenant: None, ..settings }))?;
        context.print_outcome("default tenant unset");
      }
      SETTING_DRY_RUN => {
        upsert_settings(None, |settings| Ok(Settings { dry_run: None, ..settings }))?;
        context.print_outcome("dry run mode disabled");
      }
      SETTING_LOG_LEVEL => {
        upsert_settings(None, |settings| Ok(Settings { log_level: None, ..settings }))?;
        context.print_outcome("log level unset");
      }
      SETTING_LOG_LEVEL_API => {
        upsert_settings(None, |settings| Ok(Settings { log_level_api: None, ..settings }))?;
        context.print_outcome("log level for api unset");
      }
      SETTING_LOG_LEVEL_SDK => {
        upsert_settings(None, |settings| Ok(Settings { log_level_sdk: None, ..settings }))?;
        context.print_outcome("log level for sdk unset");
      }
      SETTING_MATCHING_STYLE => {
        upsert_settings(None, |settings| Ok(Settings { matching_style: None, ..settings }))?;
        context.print_outcome("matching style unset");
      }
      SETTING_NO_ESCAPE => {
        upsert_settings(None, |settings| Ok(Settings { no_escape: None, ..settings }))?;
        context.print_outcome("no escape mode disabled");
      }
      SETTING_NO_HEADERS => {
        upsert_settings(None, |settings| Ok(Settings { no_headers: None, ..settings }))?;
        context.print_outcome("no headers mode disabled");
      }
      SETTING_OUTPUT_FORMAT => {
        upsert_settings(None, |settings| Ok(Settings { output_format: None, ..settings }))?;
        context.print_outcome("output format unset");
      }
      SETTING_QUIET => {
        upsert_settings(None, |settings| Ok(Settings { quiet: None, ..settings }))?;
        context.print_outcome("quiet mode disabled");
      }
      SETTING_SHOW_EXECUTION_TIME => {
        upsert_settings(None, |settings| Ok(Settings { show_execution_time: None, ..settings }))?;
        context.print_outcome("show execution mode unset");
      }
      SETTING_TERMINAL_WIDTH => {
        upsert_settings(None, |settings| Ok(Settings { terminal_width: None, ..settings }))?;
        context.print_outcome("terminal width unset");
      }
      SETTING_VERBOSITY => {
        upsert_settings(None, |settings| Ok(Settings { verbosity: None, ..settings }))?;
        context.print_outcome("verbosity level unset");
      }
      _ => unreachable!(),
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
  LogLevel,
  LogLevelApi,
  LogLevelSdk,
  MatchingStyle,
  NoEscape,
  NoHeaders,
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
      Self::CsvQuote => SETTING_CSV_QUOTE,
      Self::CsvSeparator => SETTING_CSV_SEPARATOR,
      Self::DefaultPlatform => SETTING_DEFAULT_PLATFORM,
      Self::DefaultTenant => SETTING_DEFAULT_TENANT,
      Self::DryRun => SETTING_DRY_RUN,
      Self::FileName => "settings file name",
      Self::LogLevel => SETTING_LOG_LEVEL,
      Self::LogLevelApi => SETTING_LOG_LEVEL_API,
      Self::LogLevelSdk => SETTING_LOG_LEVEL_SDK,
      Self::MatchingStyle => SETTING_MATCHING_STYLE,
      Self::NoEscape => SETTING_NO_ESCAPE,
      Self::NoHeaders => SETTING_NO_HEADERS,
      Self::OutputFormat => SETTING_OUTPUT_FORMAT,
      Self::Quiet => SETTING_QUIET,
      Self::ShowExecutionTime => SETTING_SHOW_EXECUTION_TIME,
      Self::Target => "setting",
      Self::TerminalWidth => SETTING_TERMINAL_WIDTH,
      Self::Verbosity => SETTING_VERBOSITY,
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<SettingLabel> for Settings {
  fn value(&self, label: &SettingLabel, target_id: &str) -> String {
    match label {
      SettingLabel::CsvQuote => self.csv_quote.map(|csv_quote| csv_quote.to_string()).unwrap_or_default(),
      SettingLabel::CsvSeparator => self.csv_separator.clone().unwrap_or_default(),
      SettingLabel::DefaultPlatform => self.default_platform.clone().unwrap_or_default(),
      SettingLabel::DefaultTenant => self.default_tenant.clone().unwrap_or_default(),
      SettingLabel::DryRun => self.dry_run.map(|dry_run| dry_run.to_string()).unwrap_or_default(),
      SettingLabel::FileName => self.file_name.clone().unwrap_or_default(),
      SettingLabel::MatchingStyle => self.matching_style.clone().map(|style| style.to_string()).unwrap_or_default(),
      SettingLabel::LogLevel => self.log_level.clone().map(|log_level| log_level.to_string()).unwrap_or_default(),
      SettingLabel::LogLevelApi => self.log_level_api.clone().map(|log_level_api| log_level_api.to_string()).unwrap_or_default(),
      SettingLabel::LogLevelSdk => self.log_level_sdk.clone().map(|log_level_sdk| log_level_sdk.to_string()).unwrap_or_default(),
      SettingLabel::NoEscape => self.no_escape.map(|no_escape| no_escape.to_string()).unwrap_or_default(),
      SettingLabel::NoHeaders => self.no_headers.map(|no_headers| no_headers.to_string()).unwrap_or_default(),
      SettingLabel::OutputFormat => self.output_format.clone().map(|format| format.to_string()).unwrap_or_default(),
      SettingLabel::Quiet => self.quiet.map(|quiet| quiet.to_string()).unwrap_or_default(),
      SettingLabel::ShowExecutionTime => self
        .show_execution_time
        .map(|show_execution_time| show_execution_time.to_string())
        .unwrap_or_default(),
      SettingLabel::Target => target_id.to_string(),
      SettingLabel::TerminalWidth => self.terminal_width.map(|width| width.to_string()).unwrap_or_default(),
      SettingLabel::Verbosity => self.verbosity.clone().map(|verbosity| verbosity.to_string()).unwrap_or_default(),
    }
  }

  fn target_label(&self) -> Option<SettingLabel> {
    Some(SettingLabel::Target)
  }
}

pub static SETTING_LABELS: [SettingLabel; 18] = [
  SettingLabel::CsvQuote,
  SettingLabel::CsvSeparator,
  SettingLabel::DefaultPlatform,
  SettingLabel::DefaultTenant,
  SettingLabel::DryRun,
  SettingLabel::MatchingStyle,
  SettingLabel::LogLevel,
  SettingLabel::LogLevelApi,
  SettingLabel::LogLevelSdk,
  SettingLabel::NoEscape,
  SettingLabel::NoHeaders,
  SettingLabel::OutputFormat,
  SettingLabel::Quiet,
  SettingLabel::ShowExecutionTime,
  SettingLabel::Target,
  SettingLabel::TerminalWidth,
  SettingLabel::Verbosity,
  SettingLabel::FileName,
];
