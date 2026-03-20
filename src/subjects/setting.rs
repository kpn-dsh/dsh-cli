use crate::arguments::{platform_name_argument, tenant_name_argument};
use crate::authentication::AuthenticationMethod;
use crate::capability::{Capability, CommandExecutor, DEFAULT_COMMAND, DEFAULT_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS, SET_COMMAND, UNSET_COMMAND};
use crate::capability_builder::CapabilityBuilder;
use crate::context::{BrowserMethod, Context};
use crate::directory::get_settings;
use crate::environment_variables::get_configured_environment_variables;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::keyring::get_secret_from_keyring;
use crate::log_level::LogLevel;
use crate::settings::{upsert_settings, Settings};
use crate::style::{DshColor, DshStyle};
use crate::subject::{Requirements, Subject};
use crate::target_arguments::{get_platform_argument_or_prompt, get_tenant_argument_or_prompt};
use crate::verbosity::Verbosity;
use crate::{err, plain, DshCliResult};
use async_trait::async_trait;
use clap::builder::EnumValueParser;
use clap::{builder, Arg, ArgAction, ArgMatches, Command};
use dsh_api::platform::DshPlatform;
use dsh_api::secret::ROBOT_SECRET;
use lazy_static::lazy_static;
use serde::Serialize;
use std::fmt::Display;

struct SettingSubject {}

const SETTING_SUBJECT_TARGET: &str = "setting";

lazy_static! {
  pub(crate) static ref SETTING_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SettingSubject {});
}

#[async_trait]
impl Subject for SettingSubject {
  fn subject(&self) -> &'static str {
    SETTING_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list dsh settings.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DEFAULT_COMMAND => Some(SETTING_DEFAULT_CAPABILITY.as_ref()),
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

const SETTING_AUTHENTICATION: &str = "authentication";
const SETTING_BROWSER: &str = "browser";
const SETTING_CSV_QUOTE: &str = "csv-quote";
const SETTING_CSV_SEPARATOR: &str = "csv-separator";
const SETTING_DEFAULT_PLATFORM: &str = "default-platform";
const SETTING_DEFAULT_TENANT: &str = "default-tenant";
const SETTING_DRY_RUN: &str = "dry-run";
const SETTING_ERROR_COLOR: &str = "error-color";
const SETTING_ERROR_STYLE: &str = "error-style";
const SETTING_LABEL_COLOR: &str = "label-color";
const SETTING_LABEL_STYLE: &str = "label-style";
const SETTING_LOG_COLOR: &str = "log-color";
const SETTING_LOG_LEVEL: &str = "log-level";
const SETTING_LOG_LEVEL_API: &str = "log-level-api";
const SETTING_LOG_STYLE: &str = "log-style";
const SETTING_MATCHING_COLOR: &str = "matching-color";
const SETTING_MATCHING_STYLE: &str = "matching-style";
const SETTING_NO_CSV_HEADERS: &str = "no-csv-headers";
const SETTING_NO_ESCAPE: &str = "no-escape";
const SETTING_OUTPUT_FORMAT: &str = "output-format";
const SETTING_QUIET: &str = "quiet";
const SETTING_SHOW_EXECUTION_TIME: &str = "show-execution-time";
const SETTING_STDERR_COLOR: &str = "stderr-color";
const SETTING_STDERR_STYLE: &str = "stderr-style";
const SETTING_STDOUT_COLOR: &str = "stdout-color";
const SETTING_STDOUT_STYLE: &str = "stdout-style";
const SETTING_SUPPRESS_EXIT_STATUS: &str = "suppress-exit-status";
const SETTING_TARGET_COLOR: &str = "target-color";
const SETTING_TARGET_STYLE: &str = "target-style";
const SETTING_TERMINAL_WIDTH: &str = "terminal-width";
const SETTING_VERBOSITY: &str = "verbosity";
const SETTING_WARNING_COLOR: &str = "warning-color";
const SETTING_WARNING_STYLE: &str = "warning-style";

fn set_unset_commands(required: bool) -> Vec<Command> {
  vec![
    Command::new(SETTING_AUTHENTICATION)
      .arg(
        Arg::new(SETTING_AUTHENTICATION)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<AuthenticationMethod>::new())
          .required(required),
      )
      .about("Authentication method"),
    Command::new(SETTING_BROWSER)
      .arg(
        Arg::new(SETTING_BROWSER)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<BrowserMethod>::new())
          .required(required),
      )
      .about("Specifies whether the tool may try to open a browser"),
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
    Command::new(SETTING_ERROR_COLOR)
      .arg(
        Arg::new(SETTING_ERROR_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing error messages"),
    Command::new(SETTING_ERROR_STYLE)
      .arg(
        Arg::new(SETTING_ERROR_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing error messages"),
    Command::new(SETTING_LABEL_COLOR)
      .arg(
        Arg::new(SETTING_LABEL_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing table headers and labels"),
    Command::new(SETTING_LABEL_STYLE)
      .arg(
        Arg::new(SETTING_LABEL_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing table headers and labels"),
    Command::new(SETTING_LOG_COLOR)
      .arg(
        Arg::new(SETTING_LOG_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing logging information"),
    Command::new(SETTING_LOG_LEVEL)
      .arg(
        Arg::new(SETTING_LOG_LEVEL)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<LogLevel>::new())
          .required(required),
      )
      .about("Log level for the dsh tool"),
    Command::new(SETTING_LOG_LEVEL_API)
      .arg(
        Arg::new(SETTING_LOG_LEVEL_API)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<LogLevel>::new())
          .required(required),
      )
      .about("Log level for the 'dsh_api' library functions"),
    Command::new(SETTING_LOG_STYLE)
      .arg(
        Arg::new(SETTING_LOG_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing logging information"),
    Command::new(SETTING_MATCHING_COLOR)
      .arg(
        Arg::new(SETTING_MATCHING_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing matching results for the find functions"),
    Command::new(SETTING_MATCHING_STYLE)
      .arg(
        Arg::new(SETTING_MATCHING_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing matching results for the find functions"),
    Command::new(SETTING_NO_CSV_HEADERS).about("Disables headers in csv output"),
    Command::new(SETTING_NO_ESCAPE).about("Inhibits any color or other ansi escape sequences"),
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
    Command::new(SETTING_STDERR_COLOR)
      .arg(
        Arg::new(SETTING_STDERR_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing explanations and metadata to stderr"),
    Command::new(SETTING_STDERR_STYLE)
      .arg(
        Arg::new(SETTING_STDERR_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing explanations and metadata to stderr"),
    Command::new(SETTING_STDOUT_COLOR)
      .arg(
        Arg::new(SETTING_STDOUT_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing results"),
    Command::new(SETTING_STDOUT_STYLE)
      .arg(
        Arg::new(SETTING_STDOUT_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing results"),
    Command::new(SETTING_SUPPRESS_EXIT_STATUS).about("Suppress exit status"),
    Command::new(SETTING_TARGET_COLOR)
      .arg(
        Arg::new(SETTING_TARGET_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing target identifiers"),
    Command::new(SETTING_TARGET_STYLE)
      .arg(
        Arg::new(SETTING_TARGET_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing target identifiers"),
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
    Command::new(SETTING_WARNING_COLOR)
      .arg(
        Arg::new(SETTING_WARNING_COLOR)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshColor>::new())
          .required(required),
      )
      .about("Color to be used when printing warning messages"),
    Command::new(SETTING_WARNING_STYLE)
      .arg(
        Arg::new(SETTING_WARNING_STYLE)
          .action(ArgAction::Set)
          .value_parser(EnumValueParser::<DshStyle>::new())
          .required(required),
      )
      .about("Styling to be used when printing warning messages"),
  ]
}

lazy_static! {
  static ref SETTING_DEFAULT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DEFAULT_COMMAND, Some(DEFAULT_COMMAND_ALIAS), &SettingDefault {}, "Set default platform and tenant")
      .set_long_about("Sets the default target platform and target tenant.")
      .add_target_argument(platform_name_argument())
      .add_target_argument(tenant_name_argument())
  );
  static ref SETTING_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &SettingList {}, "List settings").set_long_about("Lists all dsh settings."));
  static ref SETTING_SETTING_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SET_COMMAND, None, &SettingSet {}, "Set setting")
      .set_long_about("Set value to persistent storage.")
      .add_subcommands(set_unset_commands(true))
  );
  static ref SETTING_UNSETTING_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UNSET_COMMAND, None, &SettingUnset {}, "Unset setting")
      .set_long_about("Unset value from persistent storage.")
      .add_subcommands(set_unset_commands(false))
  );
  static ref SETTING_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![SETTING_DEFAULT_CAPABILITY.as_ref(), SETTING_LIST_CAPABILITY.as_ref(), SETTING_SETTING_CAPABILITY.as_ref(), SETTING_UNSETTING_CAPABILITY.as_ref()];
}

struct SettingDefault {}

#[async_trait]
impl CommandExecutor for SettingDefault {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("set default platform and tenant");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
    if get_secret_from_keyring(&platform, &tenant, ROBOT_SECRET)?.is_none() {
      return err!("keyring contains no password for target '{}@{}'", tenant, platform);
    }
    upsert_settings(|settings| Ok(Settings { default_platform: Some(platform.to_string()), ..settings }))?;
    context.print_outcome(format!("default platform set to {}", platform));
    upsert_settings(|settings| Ok(Settings { default_tenant: Some(tenant.to_string()), ..settings }))?;
    context.print_outcome(format!("default tenant set to {}", tenant));
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct SettingList {}

#[async_trait]
impl CommandExecutor for SettingList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let (settings, _) = get_settings()?;
    if let Some(ref settings_file) = settings.file_name {
      context.print_explanation(format!("list settings from settings file '{}'", settings_file));
      UnitFormatter::new("value", &SETTING_LABELS, context).print(&settings, None)?
    } else {
      context.print_explanation("list default settings");
      UnitFormatter::new("value", &SETTING_LABELS, context).print(&settings, None)?
    }
    let configured_environment_variables = get_configured_environment_variables();
    if !configured_environment_variables.is_empty() {
      context.print_explanation("list environment variables");
      let mut formatter = ListFormatter::new(&ENVIRONMENT_VARIABLE_LABELS, context);
      for (env_var_name, env_var_value) in &configured_environment_variables {
        formatter.push_target_id_value(env_var_name.to_string(), env_var_value);
      }
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

fn get_some<T>(setting: &str, matches: &ArgMatches, context: &Context) -> DshCliResult<Option<T>>
where
  T: Clone + Display + Send + Sync + 'static,
{
  match matches.get_one::<T>(setting) {
    Some(one) => {
      let cloned = one.clone();
      context.print_outcome(format!("{} set to {}", setting, &cloned));
      Ok(Some(cloned))
    }
    None => err!("{}", setting),
  }
}

struct SettingSet {}

#[async_trait]
impl CommandExecutor for SettingSet {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let (target_setting, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    match target_setting {
      SETTING_AUTHENTICATION => {
        upsert_settings(move |settings| Ok(Settings { authentication: get_some(SETTING_AUTHENTICATION, matches, context)?, ..settings }))?;
      }
      SETTING_BROWSER => {
        upsert_settings(move |settings| Ok(Settings { browser: get_some(SETTING_BROWSER, matches, context)?, ..settings }))?;
      }
      SETTING_CSV_QUOTE => match matches.get_one::<String>(SETTING_CSV_QUOTE) {
        Some(csv_quote_argument) => {
          let mut csv_quote_chars = csv_quote_argument.chars();
          match csv_quote_chars.next() {
            Some(csv_quote) => {
              if csv_quote_chars.next().is_some() {
                return err!("csv quote must be a single character");
              } else {
                upsert_settings(|settings| Ok(Settings { csv_quote: Some(csv_quote), ..settings }))?;
                context.print_outcome(format!("csv quote character set to '{}'", csv_quote));
              }
            }
            None => unreachable!(),
          }
        }
        None => unreachable!(),
      },
      SETTING_CSV_SEPARATOR => {
        upsert_settings(move |settings| Ok(Settings { csv_separator: get_some(SETTING_CSV_SEPARATOR, matches, context)?, ..settings }))?;
      }
      SETTING_DEFAULT_PLATFORM => match matches.get_one::<String>(SETTING_DEFAULT_PLATFORM) {
        Some(platform_name) => match DshPlatform::try_from(platform_name.as_str()) {
          Ok(platform) => {
            upsert_settings(|settings| Ok(Settings { default_platform: Some(platform.to_string()), ..settings }))?;
            context.print_outcome(format!("default platform set to {}", platform));
          }
          Err(_) => unreachable!(),
        },
        None => unreachable!(),
      },
      SETTING_DEFAULT_TENANT => {
        upsert_settings(move |settings| Ok(Settings { default_tenant: get_some(SETTING_DEFAULT_TENANT, matches, context)?, ..settings }))?;
      }
      SETTING_DRY_RUN => {
        upsert_settings(|settings| Ok(Settings { dry_run: Some(true), ..settings }))?;
        context.print_outcome("dry run mode enabled");
      }
      SETTING_ERROR_COLOR => {
        upsert_settings(move |settings| Ok(Settings { error_color: get_some(SETTING_ERROR_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_ERROR_STYLE => {
        upsert_settings(move |settings| Ok(Settings { error_style: get_some(SETTING_ERROR_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_LABEL_COLOR => {
        upsert_settings(move |settings| Ok(Settings { label_color: get_some(SETTING_LABEL_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_LABEL_STYLE => {
        upsert_settings(move |settings| Ok(Settings { label_style: get_some(SETTING_LABEL_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_LOG_COLOR => {
        upsert_settings(move |settings| Ok(Settings { log_color: get_some(SETTING_LOG_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_LOG_LEVEL => {
        upsert_settings(move |settings| Ok(Settings { log_level: get_some(SETTING_LOG_LEVEL, matches, context)?, ..settings }))?;
      }
      SETTING_LOG_LEVEL_API => {
        upsert_settings(move |settings| Ok(Settings { log_level_api: get_some(SETTING_LOG_LEVEL_API, matches, context)?, ..settings }))?;
      }
      SETTING_LOG_STYLE => {
        upsert_settings(move |settings| Ok(Settings { log_style: get_some(SETTING_LOG_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_MATCHING_COLOR => {
        upsert_settings(move |settings| Ok(Settings { matching_color: get_some(SETTING_MATCHING_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_MATCHING_STYLE => {
        upsert_settings(move |settings| Ok(Settings { matching_style: get_some(SETTING_MATCHING_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_NO_CSV_HEADERS => {
        upsert_settings(|settings| Ok(Settings { no_csv_headers: Some(true), ..settings }))?;
        context.print_outcome("no csv headers mode enabled");
      }
      SETTING_NO_ESCAPE => {
        upsert_settings(|settings| Ok(Settings { no_escape: Some(true), ..settings }))?;
        context.print_outcome("no escape mode enabled");
      }
      SETTING_OUTPUT_FORMAT => {
        upsert_settings(move |settings| Ok(Settings { output_format: get_some(SETTING_OUTPUT_FORMAT, matches, context)?, ..settings }))?;
      }
      SETTING_QUIET => {
        upsert_settings(|settings| Ok(Settings { quiet: Some(true), ..settings }))?;
        context.print_outcome("quiet mode enabled");
      }
      SETTING_SHOW_EXECUTION_TIME => {
        upsert_settings(|settings| Ok(Settings { show_execution_time: Some(true), ..settings }))?;
        context.print_outcome("show execution time enabled");
      }
      SETTING_STDERR_COLOR => {
        upsert_settings(move |settings| Ok(Settings { stderr_color: get_some(SETTING_STDERR_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_STDERR_STYLE => {
        upsert_settings(move |settings| Ok(Settings { stderr_style: get_some(SETTING_STDERR_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_STDOUT_COLOR => {
        upsert_settings(move |settings| Ok(Settings { stdout_color: get_some(SETTING_STDOUT_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_STDOUT_STYLE => {
        upsert_settings(move |settings| Ok(Settings { stdout_style: get_some(SETTING_STDOUT_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_SUPPRESS_EXIT_STATUS => {
        upsert_settings(|settings| Ok(Settings { suppress_exit_status: Some(true), ..settings }))?;
        context.print_outcome("suppress exit status enabled");
      }
      SETTING_TARGET_COLOR => {
        upsert_settings(move |settings| Ok(Settings { target_color: get_some(SETTING_TARGET_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_TARGET_STYLE => {
        upsert_settings(move |settings| Ok(Settings { target_style: get_some(SETTING_TARGET_STYLE, matches, context)?, ..settings }))?;
      }
      SETTING_TERMINAL_WIDTH => {
        let terminal_width = matches.get_one::<usize>(SETTING_TERMINAL_WIDTH).unwrap_or_else(|| unreachable!());
        if *terminal_width < 40 {
          return err!("terminal width must be greater than or equal to 40");
        } else {
          upsert_settings(|settings| Ok(Settings { terminal_width: Some(*terminal_width), ..settings }))?;
          context.print_outcome(format!("terminal width set to {}", terminal_width));
        }
      }
      SETTING_VERBOSITY => {
        upsert_settings(move |settings| Ok(Settings { verbosity: get_some(SETTING_VERBOSITY, matches, context)?, ..settings }))?;
      }
      SETTING_WARNING_COLOR => {
        upsert_settings(move |settings| Ok(Settings { warning_color: get_some(SETTING_WARNING_COLOR, matches, context)?, ..settings }))?;
      }
      SETTING_WARNING_STYLE => {
        upsert_settings(move |settings| Ok(Settings { warning_style: get_some(SETTING_WARNING_STYLE, matches, context)?, ..settings }))?;
      }
      _ => unreachable!(),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct SettingUnset {}

#[async_trait]
impl CommandExecutor for SettingUnset {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let (target_setting, _) = matches.subcommand().unwrap_or_else(|| unreachable!());
    match target_setting {
      SETTING_AUTHENTICATION => {
        upsert_settings(|settings| Ok(Settings { authentication: None, ..settings }))?;
        context.print_outcome("authentication method unset");
      }
      SETTING_BROWSER => {
        upsert_settings(|settings| Ok(Settings { browser: None, ..settings }))?;
        context.print_outcome("browser method unset");
      }
      SETTING_CSV_QUOTE => {
        upsert_settings(|settings| Ok(Settings { csv_quote: None, ..settings }))?;
        context.print_outcome("csv quote unset");
      }
      SETTING_CSV_SEPARATOR => {
        upsert_settings(|settings| Ok(Settings { csv_separator: None, ..settings }))?;
        context.print_outcome("csv separator unset");
      }
      SETTING_DEFAULT_PLATFORM => {
        upsert_settings(|settings| Ok(Settings { default_platform: None, ..settings }))?;
        context.print_outcome("default platform unset");
      }
      SETTING_DEFAULT_TENANT => {
        upsert_settings(|settings| Ok(Settings { default_tenant: None, ..settings }))?;
        context.print_outcome("default tenant unset");
      }
      SETTING_DRY_RUN => {
        upsert_settings(|settings| Ok(Settings { dry_run: None, ..settings }))?;
        context.print_outcome("dry run mode disabled");
      }
      SETTING_ERROR_COLOR => {
        upsert_settings(|settings| Ok(Settings { error_color: None, ..settings }))?;
        context.print_outcome("error color unset");
      }
      SETTING_ERROR_STYLE => {
        upsert_settings(|settings| Ok(Settings { error_style: None, ..settings }))?;
        context.print_outcome("error style unset");
      }
      SETTING_LABEL_COLOR => {
        upsert_settings(|settings| Ok(Settings { label_color: None, ..settings }))?;
        context.print_outcome("label color unset");
      }
      SETTING_LABEL_STYLE => {
        upsert_settings(|settings| Ok(Settings { label_style: None, ..settings }))?;
        context.print_outcome("label style unset");
      }
      SETTING_LOG_COLOR => {
        upsert_settings(|settings| Ok(Settings { log_color: None, ..settings }))?;
        context.print_outcome("log color unset");
      }
      SETTING_LOG_LEVEL => {
        upsert_settings(|settings| Ok(Settings { log_level: None, ..settings }))?;
        context.print_outcome("log level unset");
      }
      SETTING_LOG_LEVEL_API => {
        upsert_settings(|settings| Ok(Settings { log_level_api: None, ..settings }))?;
        context.print_outcome("log level for api unset");
      }
      SETTING_LOG_STYLE => {
        upsert_settings(|settings| Ok(Settings { log_style: None, ..settings }))?;
        context.print_outcome("log style unset");
      }
      SETTING_MATCHING_COLOR => {
        upsert_settings(|settings| Ok(Settings { matching_color: None, ..settings }))?;
        context.print_outcome("matching color unset");
      }
      SETTING_MATCHING_STYLE => {
        upsert_settings(|settings| Ok(Settings { matching_style: None, ..settings }))?;
        context.print_outcome("matching style unset");
      }
      SETTING_NO_CSV_HEADERS => {
        upsert_settings(|settings| Ok(Settings { no_csv_headers: None, ..settings }))?;
        context.print_outcome("no csv headers mode disabled");
      }
      SETTING_NO_ESCAPE => {
        upsert_settings(|settings| Ok(Settings { no_escape: None, ..settings }))?;
        context.print_outcome("no escape mode disabled");
      }
      SETTING_OUTPUT_FORMAT => {
        upsert_settings(|settings| Ok(Settings { output_format: None, ..settings }))?;
        context.print_outcome("output format unset");
      }
      SETTING_QUIET => {
        upsert_settings(|settings| Ok(Settings { quiet: None, ..settings }))?;
        context.print_outcome("quiet mode disabled");
      }
      SETTING_SHOW_EXECUTION_TIME => {
        upsert_settings(|settings| Ok(Settings { show_execution_time: None, ..settings }))?;
        context.print_outcome("show execution mode unset");
      }
      SETTING_STDERR_COLOR => {
        upsert_settings(|settings| Ok(Settings { stderr_color: None, ..settings }))?;
        context.print_outcome("stderr color unset");
      }
      SETTING_STDERR_STYLE => {
        upsert_settings(|settings| Ok(Settings { stderr_style: None, ..settings }))?;
        context.print_outcome("stderr style unset");
      }
      SETTING_STDOUT_COLOR => {
        upsert_settings(|settings| Ok(Settings { stdout_color: None, ..settings }))?;
        context.print_outcome("stdout color unset");
      }
      SETTING_STDOUT_STYLE => {
        upsert_settings(|settings| Ok(Settings { stdout_style: None, ..settings }))?;
        context.print_outcome("stdout style unset");
      }
      SETTING_SUPPRESS_EXIT_STATUS => {
        upsert_settings(|settings| Ok(Settings { suppress_exit_status: None, ..settings }))?;
        context.print_outcome("suppress exit status disabled");
      }
      SETTING_TARGET_COLOR => {
        upsert_settings(|settings| Ok(Settings { target_color: None, ..settings }))?;
        context.print_outcome("target color unset");
      }
      SETTING_TARGET_STYLE => {
        upsert_settings(|settings| Ok(Settings { target_style: None, ..settings }))?;
        context.print_outcome("target style unset");
      }
      SETTING_TERMINAL_WIDTH => {
        upsert_settings(|settings| Ok(Settings { terminal_width: None, ..settings }))?;
        context.print_outcome("terminal width unset");
      }
      SETTING_VERBOSITY => {
        upsert_settings(|settings| Ok(Settings { verbosity: None, ..settings }))?;
        context.print_outcome("verbosity level unset");
      }
      SETTING_WARNING_COLOR => {
        upsert_settings(|settings| Ok(Settings { warning_color: None, ..settings }))?;
        context.print_outcome("warning color unset");
      }
      SETTING_WARNING_STYLE => {
        upsert_settings(|settings| Ok(Settings { warning_style: None, ..settings }))?;
        context.print_outcome("warning style unset");
      }
      _ => unreachable!(),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum SettingLabel {
  Authentication,
  Browser,
  CsvQuote,
  CsvSeparator,
  DefaultPlatform,
  DefaultTenant,
  DryRun,
  ErrorColor,
  ErrorStyle,
  FileName,
  LabelColor,
  LabelStyle,
  LogColor,
  LogLevel,
  LogLevelApi,
  LogStyle,
  MatchingColor,
  MatchingStyle,
  NoCsvHeaders,
  NoEscape,
  OutputFormat,
  Quiet,
  ShowExecutionTime,
  StderrColor,
  StderrStyle,
  StdoutColor,
  StdoutStyle,
  SuppressExitStatus,
  Target,
  TargetColor,
  TargetStyle,
  TerminalWidth,
  Verbosity,
  WarningColor,
  WarningStyle,
}

impl Label for SettingLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Authentication => SETTING_AUTHENTICATION,
      Self::Browser => SETTING_BROWSER,
      Self::CsvQuote => SETTING_CSV_QUOTE,
      Self::CsvSeparator => SETTING_CSV_SEPARATOR,
      Self::DefaultPlatform => SETTING_DEFAULT_PLATFORM,
      Self::DefaultTenant => SETTING_DEFAULT_TENANT,
      Self::DryRun => SETTING_DRY_RUN,
      Self::ErrorColor => SETTING_ERROR_COLOR,
      Self::ErrorStyle => SETTING_ERROR_STYLE,
      Self::FileName => "settings file name",
      Self::LabelColor => SETTING_LABEL_COLOR,
      Self::LabelStyle => SETTING_LABEL_STYLE,
      Self::LogColor => SETTING_LOG_COLOR,
      Self::LogLevel => SETTING_LOG_LEVEL,
      Self::LogLevelApi => SETTING_LOG_LEVEL_API,
      Self::LogStyle => SETTING_LOG_STYLE,
      Self::MatchingColor => SETTING_MATCHING_COLOR,
      Self::MatchingStyle => SETTING_MATCHING_STYLE,
      Self::NoCsvHeaders => SETTING_NO_CSV_HEADERS,
      Self::NoEscape => SETTING_NO_ESCAPE,
      Self::OutputFormat => SETTING_OUTPUT_FORMAT,
      Self::Quiet => SETTING_QUIET,
      Self::ShowExecutionTime => SETTING_SHOW_EXECUTION_TIME,
      Self::StderrColor => SETTING_STDERR_COLOR,
      Self::StderrStyle => SETTING_STDERR_STYLE,
      Self::StdoutColor => SETTING_STDOUT_COLOR,
      Self::StdoutStyle => SETTING_STDOUT_STYLE,
      Self::SuppressExitStatus => SETTING_SUPPRESS_EXIT_STATUS,
      Self::Target => "setting",
      Self::TargetColor => SETTING_TARGET_COLOR,
      Self::TargetStyle => SETTING_TARGET_STYLE,
      Self::TerminalWidth => SETTING_TERMINAL_WIDTH,
      Self::Verbosity => SETTING_VERBOSITY,
      Self::WarningColor => SETTING_WARNING_COLOR,
      Self::WarningStyle => SETTING_WARNING_STYLE,
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<SettingLabel> for Settings {
  fn value(&self, label: &SettingLabel, target_id: &str) -> Value {
    match label {
      SettingLabel::Authentication => Value::option(self.authentication.as_ref()),
      SettingLabel::Browser => Value::option(self.browser.as_ref()),
      SettingLabel::CsvQuote => Value::option(self.csv_quote),
      SettingLabel::CsvSeparator => Value::option(self.csv_separator.clone()),
      SettingLabel::DefaultPlatform => match self.default_platform.clone().map(|platform| DshPlatform::try_from(platform.as_str())) {
        Some(Ok(platform)) => plain!("{} / {}", platform.name(), platform.alias()),
        _ => Value::empty(),
      },
      SettingLabel::DefaultTenant => Value::option(self.default_tenant.clone()),
      SettingLabel::DryRun => Value::option(self.dry_run),
      SettingLabel::ErrorColor => Value::option(self.error_color.as_ref()),
      SettingLabel::ErrorStyle => Value::option(self.error_style.as_ref()),
      SettingLabel::FileName => Value::option(self.file_name.clone()),
      SettingLabel::LabelColor => Value::option(self.label_color.as_ref()),
      SettingLabel::LabelStyle => Value::option(self.label_style.as_ref()),
      SettingLabel::LogColor => Value::option(self.log_color.as_ref()),
      SettingLabel::LogLevel => Value::option(self.log_level.as_ref()),
      SettingLabel::LogLevelApi => Value::option(self.log_level_api.as_ref()),
      SettingLabel::LogStyle => Value::option(self.log_style.as_ref()),
      SettingLabel::MatchingColor => Value::option(self.matching_color.as_ref()),
      SettingLabel::MatchingStyle => Value::option(self.matching_style.as_ref()),
      SettingLabel::NoCsvHeaders => Value::option(self.no_csv_headers),
      SettingLabel::NoEscape => Value::option(self.no_escape),
      SettingLabel::OutputFormat => Value::option(self.output_format.as_ref()),
      SettingLabel::Quiet => Value::option(self.quiet),
      SettingLabel::ShowExecutionTime => Value::option(self.show_execution_time),
      SettingLabel::StderrColor => Value::option(self.stderr_color.as_ref()),
      SettingLabel::StderrStyle => Value::option(self.stderr_style.as_ref()),
      SettingLabel::StdoutColor => Value::option(self.stdout_color.as_ref()),
      SettingLabel::StdoutStyle => Value::option(self.stdout_style.as_ref()),
      SettingLabel::SuppressExitStatus => Value::option(self.suppress_exit_status),
      SettingLabel::Target => Value::target(target_id),
      SettingLabel::TargetColor => Value::option(self.target_color.as_ref()),
      SettingLabel::TargetStyle => Value::option(self.target_style.as_ref()),
      SettingLabel::TerminalWidth => Value::option(self.terminal_width),
      SettingLabel::Verbosity => Value::option(self.verbosity.as_ref()),
      SettingLabel::WarningColor => Value::option(self.warning_color.as_ref()),
      SettingLabel::WarningStyle => Value::option(self.warning_style.as_ref()),
    }
  }
}

static SETTING_LABELS: [SettingLabel; 35] = [
  SettingLabel::Authentication,
  SettingLabel::Browser,
  SettingLabel::CsvQuote,
  SettingLabel::CsvSeparator,
  SettingLabel::DefaultPlatform,
  SettingLabel::DefaultTenant,
  SettingLabel::DryRun,
  SettingLabel::ErrorColor,
  SettingLabel::ErrorStyle,
  SettingLabel::FileName,
  SettingLabel::LabelColor,
  SettingLabel::LabelStyle,
  SettingLabel::LogLevel,
  SettingLabel::LogLevelApi,
  SettingLabel::LogColor,
  SettingLabel::LogStyle,
  SettingLabel::MatchingColor,
  SettingLabel::MatchingStyle,
  SettingLabel::NoCsvHeaders,
  SettingLabel::NoEscape,
  SettingLabel::OutputFormat,
  SettingLabel::Quiet,
  SettingLabel::ShowExecutionTime,
  SettingLabel::StderrColor,
  SettingLabel::StderrStyle,
  SettingLabel::StdoutColor,
  SettingLabel::StdoutStyle,
  SettingLabel::SuppressExitStatus,
  SettingLabel::Target,
  SettingLabel::TargetColor,
  SettingLabel::TargetStyle,
  SettingLabel::TerminalWidth,
  SettingLabel::Verbosity,
  SettingLabel::WarningColor,
  SettingLabel::WarningStyle,
];

#[derive(Eq, Hash, PartialEq, Serialize)]
enum EnvironmentVariableLabel {
  Variable,
  Value,
}

impl Label for EnvironmentVariableLabel {
  fn as_str(&self) -> &str {
    match self {
      EnvironmentVariableLabel::Variable => "environment variable",
      EnvironmentVariableLabel::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Variable)
  }
}

static ENVIRONMENT_VARIABLE_LABELS: [EnvironmentVariableLabel; 2] = [EnvironmentVariableLabel::Variable, EnvironmentVariableLabel::Value];
