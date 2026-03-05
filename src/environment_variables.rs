use crate::context::Context;
use crate::error::DshCliError;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::global_arguments::ENVIRONMENT_VARIABLE_ARGUMENT;
use crate::{err, DshCliResult, TOOL_OPTIONS_HEADING};
use clap::builder::ValueParser;
use clap::{builder, Arg, ArgAction, ArgMatches};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::{debug, trace, warn};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::PathBuf;

/// # Get environment variable value
///
/// Gets the value of an environment variable if it is configured either via the command line or
/// as a genuine environment variable.
///
/// 1. Try if `env_var_name` is specified as a command line environment variable argument.
/// 1. Try if `env_var_name` is specified as a regular environment variable.
/// 1. Default to `None`.
///
/// # Parameters
/// * `env_var_name` - Name of the environment variable.
/// * `matches` - Parsed command line arguments.
///
/// # Returns
/// * `Ok<Some<value>>` - When the environment variable `env_var_name` is specified either
///   via the command line or as a regular environment variable.
/// * `Ok<None>` - When the environment variable `env_var_name` is not specified.
/// * `Err<message>` - When the command line specifies the environment variable `env_var_name`
///   more than once.
pub(crate) fn environment_variable(env_var_name: &str, matches: Option<&ArgMatches>) -> DshCliResult<Option<String>> {
  match EnvironmentVariables.iter().find(|env_var| env_var.name == env_var_name) {
    Some(environment_variable) => {
      if environment_variable.override_allowed {
        match matches {
          Some(matches) => {
            let environment_variable_from_arguments = environment_variable_from_arguments(env_var_name, matches);
            match environment_variable_from_arguments.len() {
              0 => environment_variable_from_file_or_os(env_var_name, Some(matches))?,
              1 => match environment_variable_from_arguments.first().cloned() {
                Some(env_var_value) => {
                  override_from_argument_warning(env_var_name);
                  Ok(Some(env_var_value))
                }
                None => unreachable!(),
              },
              _ => err!("environment variable '{}' is specified more than once on the command line", env_var_name),
            }
          }
          None => environment_variable_from_file_or_os(env_var_name, matches)?,
        }
      } else {
        environment_variable_from_file_or_os(env_var_name, matches)?
      }
    }
    None => err!("environment variable '{}' not defined", env_var_name),
  }
}

/// # Check if an environment variable is specified
///
/// Check if environment variable `env_var_name` is specified either as a command line argument
/// or as a regular environment variable.
///
/// # Parameters
/// * `env_var_name` - Name of the environment variable.
/// * `matches` - Parsed command line arguments.
///
/// # Returns
/// * `true` - When the environment variable `env_var_name` is specified either
///   via the command line or as a regular environment variable. Note that the function also
///   returns `true` when `env_var_name` is specified on the command line more than once.
/// * `false` - When the environment variable `env_var_name` is not specified.
pub(crate) fn is_environment_variable_specified(env_var_name: &str, matches: &ArgMatches) -> bool {
  !environment_variable_from_arguments(env_var_name, matches).is_empty() || get_env_var(env_var_name).is_some()
}

fn environment_variable_from_file_or_os(env_var_name: &str, matches: Option<&ArgMatches>) -> Result<DshCliResult<Option<String>>, DshCliError> {
  Ok(match environment_variable_from_file(env_var_name, matches)? {
    Some((env_var_value, env_var_file)) => {
      override_from_file_warning(env_var_name, &env_var_file);
      trace!("environment variable {}={} from file '{}'", env_var_name, env_var_value, env_var_file);
      Ok(Some(env_var_value))
    }
    None => match get_env_var(env_var_name) {
      Some(env_var_value) => {
        debug!("environment variable {}={}", env_var_name, env_var_value);
        Ok(Some(env_var_value))
      }
      None => Ok(None),
    },
  })
}

const REDACTED_SECRET: &str = "[redacted]";

/// Returns the currently configured environment variables with their values
pub(crate) fn get_configured_environment_variables() -> Vec<(&'static str, String)> {
  let mut environment_variables: Vec<(&str, String)> = vec![];
  for (os_env_var_name, os_env_var_value) in std::env::vars() {
    if let Some(environment_variable) = EnvironmentVariables
      .iter()
      .find(|environment_variable| environment_variable.name == os_env_var_name)
    {
      if environment_variable.contains_secret {
        environment_variables.push((environment_variable.name, REDACTED_SECRET.to_string()))
      } else {
        environment_variables.push((environment_variable.name, os_env_var_value))
      }
    }
  }
  environment_variables.sort_by(|(env_var_a, _), (env_var_b, _)| env_var_a.cmp(env_var_b));
  environment_variables
}

pub(crate) fn print_environment_variables(context: &Context) {
  let mut formatter = ListFormatter::new(&ENV_VAR_LABELS_LIST, context);
  let styled: &Vec<(String, Option<String>, &EnvironmentVariable)> = &EnvironmentVariables
    .iter()
    .map(|environment_variable| {
      (
        context.apply_label_style_for_stdout(environment_variable.name, Some(OutputFormat::Table)),
        get_env_var(environment_variable.name),
        environment_variable,
      )
    })
    .collect_vec();
  formatter.push_values(styled);
  _ = formatter.print(Some(OutputFormat::Table));
}

pub(crate) fn print_environment_variable(env_var: &str, context: &Context) {
  let matching_env_vars: &Vec<&EnvironmentVariable> = &EnvironmentVariables
    .iter()
    .filter(|environment_variable| environment_variable.name.contains(&env_var.to_uppercase()))
    .collect_vec();
  if matching_env_vars.is_empty() {
    context.print_warning(format!("'{}' could not be matched to an environment variable recognized by the dsh tool", env_var));
  } else if matching_env_vars.len() == 1 {
    let environment_variable = matching_env_vars.first().unwrap_or_else(|| unreachable!());
    _ = UnitFormatter::new(environment_variable.name, &ENV_VAR_LABELS, context).print_non_serializable(
      &(environment_variable.name.to_string(), get_env_var(environment_variable.name), *environment_variable),
      None,
    );
  } else {
    let mut formatter = ListFormatter::new(&ENV_VAR_LABELS, context);
    let styled_matching_env_vars = matching_env_vars
      .iter()
      .map(|environment_variable| {
        (
          context.apply_label_style_for_stdout(environment_variable.name, Some(OutputFormat::Table)),
          get_env_var(environment_variable.name),
          *environment_variable,
        )
      })
      .collect_vec();
    formatter.push_values(&styled_matching_env_vars);
    _ = formatter.print(Some(OutputFormat::Table));
  }
}

fn get_env_var(env_var_name: &str) -> Option<String> {
  match std::env::var(env_var_name) {
    Ok(env_var_value) => {
      trace!("read environment variable {}={}", env_var_name, env_var_value);
      Some(env_var_value)
    }
    Err(_) => None,
  }
}

fn override_from_argument_warning(env_var_name: &str) {
  if std::env::var(env_var_name).is_ok() {
    warn!("environment variable '{}' overridden by argument", env_var_name);
  }
}

fn override_from_file_warning(env_var_name: &str, filename: &str) {
  if std::env::var(env_var_name).is_ok() {
    warn!("environment variable '{}' overridden by value from file '{}'", env_var_name, filename);
  }
}

/// # Get environment variable value from the command line
///
/// Gets the value(s) of an environment variable if it is configured via the command line.
///
/// # Parameters
/// * `env_var_name` - Name of the environment variable.
/// * `matches` - Parsed command line arguments.
///
/// # Returns
/// A vector containing the values of the environment variable when it was specified via de
/// command line. Note that the same environment variable can be configured more than once.
/// If this is the case, the return value of this function will contain all occurring values. If
/// the environment variable was not specified via the command line, this function will return the
/// empty vector.
fn environment_variable_from_arguments(env_var_name: &str, matches: &ArgMatches) -> Vec<String> {
  match matches.get_many::<String>(ENVIRONMENT_VARIABLE_ARGUMENT) {
    Some(env_var_arguments) => {
      let env_var_values = env_var_arguments
        .filter_map(|env_var_argument| parse_env_var(env_var_name, env_var_argument))
        .collect_vec();
      for env_var_value in &env_var_values {
        trace!("environment variable {}={} specified on command line", env_var_name, env_var_value);
      }
      env_var_values
    }
    None => vec![],
  }
}

const DEFAULT_ENV_FILE_NAME: &str = ".dsh_cli.env";

/// # Get environment variable value from file
///
/// Gets the value of an environment variable if it is configured in an environment variables file.
/// If the command line argument `--env-var-file` is provided, its value will be used as the
/// filename. Else, `./.dsh_cli.env` in the current working directory will be tried.
///
/// # Parameters
/// * `env_var_name` - Name of the environment variable.
/// * `matches` - Parsed command line arguments.
///
/// # Returns
/// * `Ok<Some<(value, filename)>>` - When the environment variable `env_var_name` was found in
///   the selected or default file.
/// * `Ok<None>` - When the environment variable `env_var_name` could not be found in the selected
///   or default file, or when the default file does not exist.
/// * `Err<message>` - When the selected file does not exist or when the selected or default file
///   could not be read.
fn environment_variable_from_file(env_var_name: &str, matches: Option<&ArgMatches>) -> DshCliResult<Option<(String, String)>> {
  match matches.and_then(|matches| matches.get_one::<PathBuf>(ENV_VAR_FILE_ARGUMENT)) {
    Some(env_var_file_argument) => match OpenOptions::new().read(true).open(env_var_file_argument) {
      Ok(file) => read_environment_variable_from_file(env_var_name, file, env_var_file_argument.display().to_string()),
      Err(error) => {
        if error.kind() == ErrorKind::NotFound {
          err!(
            "environment variables file '{}' specified in argument does not exist",
            env_var_file_argument.display()
          )
        } else {
          err!(
            "environment variables file '{}' specified in argument could not be read ({})",
            env_var_file_argument.display(),
            error
          )
        }
      }
    },
    None => match OpenOptions::new().read(true).open(DEFAULT_ENV_FILE_NAME) {
      Ok(file) => read_environment_variable_from_file(env_var_name, file, DEFAULT_ENV_FILE_NAME.to_string()),
      Err(error) => {
        if error.kind() == ErrorKind::NotFound {
          Ok(None)
        } else {
          err!("default environment variables file '{}' could not be read ({})", DEFAULT_ENV_FILE_NAME, error)
        }
      }
    },
  }
}

fn read_environment_variable_from_file(env_var_name: &str, file: File, filename: String) -> DshCliResult<Option<(String, String)>> {
  match BufReader::new(&file)
    .lines()
    .find_map(|env_var_argument| env_var_argument.ok().and_then(|line| parse_env_var(env_var_name, &line)))
  {
    Some(env_var_value) => {
      trace!("environment variable {}={} read from file '{}'", env_var_name, env_var_value, filename);
      Ok(Some((env_var_value, filename)))
    }
    None => Ok(None),
  }
}

fn parse_env_var(env_var_name: &str, line: &str) -> Option<String> {
  if line.starts_with("#") || line.starts_with("//") {
    None
  } else {
    line
      .strip_prefix(env_var_name)
      .and_then(|rest| rest.strip_prefix("=").map(|value| value.trim().to_string()))
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum EnvVarLabel {
  ContainsSecret,
  DefaultValue,
  EnvVar,
  LongExplanation,
  OverrideAllowed,
  ShortExplanation,
  Value,
}

impl Label for EnvVarLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::ContainsSecret => "secret",
      Self::DefaultValue => "default value",
      Self::EnvVar => "environment variable",
      Self::LongExplanation => "explanation",
      Self::OverrideAllowed => "override",
      Self::ShortExplanation => "short explanation",
      Self::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::EnvVar)
  }
}

#[derive(Serialize)]
struct EnvironmentVariable {
  name: &'static str,
  short_explanation: &'static str,
  contains_secret: bool,
  override_allowed: bool,
  default_value: Option<&'static str>,
  long_explanation: &'static str,
}

impl EnvironmentVariable {
  fn new(
    name: &'static str,
    short_explanation: &'static str,
    contains_secret: bool,
    override_allowed: bool,
    default_value: Option<&'static str>,
    long_explanation: &'static str,
  ) -> Self {
    Self { name, short_explanation, contains_secret, override_allowed, default_value, long_explanation }
  }
}

impl SubjectFormatter<EnvVarLabel> for (String, Option<String>, &EnvironmentVariable) {
  fn value(&self, label: &EnvVarLabel, _target_id: &str) -> Value {
    let (env_var_endorsed, value, environment_variable) = self;
    match label {
      EnvVarLabel::ContainsSecret => {
        if environment_variable.contains_secret {
          Value::plain("yes")
        } else {
          Value::plain("no")
        }
      }
      EnvVarLabel::DefaultValue => Value::option(environment_variable.default_value),
      EnvVarLabel::EnvVar => Value::plain(env_var_endorsed),
      EnvVarLabel::LongExplanation => Value::plain(environment_variable.long_explanation),
      EnvVarLabel::OverrideAllowed => {
        if environment_variable.override_allowed {
          Value::plain("allowed")
        } else {
          Value::plain("not allowed")
        }
      }
      EnvVarLabel::ShortExplanation => Value::plain(environment_variable.short_explanation),
      EnvVarLabel::Value => match value {
        Some(value) => {
          if environment_variable.contains_secret {
            Value::secret()
          } else {
            Value::plain(value)
          }
        }
        None => Value::empty(),
      },
    }
  }
}

const ENV_VAR_LABELS: [EnvVarLabel; 6] =
  [EnvVarLabel::EnvVar, EnvVarLabel::Value, EnvVarLabel::ContainsSecret, EnvVarLabel::OverrideAllowed, EnvVarLabel::DefaultValue, EnvVarLabel::LongExplanation];
const ENV_VAR_LABELS_LIST: [EnvVarLabel; 4] = [EnvVarLabel::EnvVar, EnvVarLabel::Value, EnvVarLabel::DefaultValue, EnvVarLabel::ShortExplanation];

pub(crate) const ENV_VARS_ARGUMENT: &str = "env-vars-argument";

pub(crate) fn env_vars_argument() -> Arg {
  Arg::new(ENV_VARS_ARGUMENT)
    .long("env-vars")
    .action(ArgAction::SetTrue)
    .long_help(
      "If this option is provided the dsh tool will print a list of all \
          recognized environment variables with a short explanation. \
          For a verbose explanation, use the --env-var ENV_VAR option. \
          When this option is used, all other provided commands or options will be ignored.",
    )
    .hide_short_help(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) const ENV_VAR_ARGUMENT: &str = "env-var-argument";

pub(crate) fn env_var_argument() -> Arg {
  Arg::new(ENV_VAR_ARGUMENT)
    .long("env-var")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("ENV_VAR")
    .long_help(
      "If this option is provided the dsh tool will print an explanation \
          of the environment variable provided as argument. \
          When this option is used, all other provided commands or options will be ignored.",
    )
    .hide_short_help(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) const ENV_VAR_FILE_ARGUMENT: &str = "env-var-file";

pub(crate) fn env_var_file_argument() -> Arg {
  Arg::new(ENV_VAR_FILE_ARGUMENT)
    .long("env-var-file")
    .action(ArgAction::Set)
    .value_parser(ValueParser::path_buf())
    .value_name("FILE")
    .long_help(
      "If this option is provided it specifies the location of a file that defines values \
      for the environment variables used by the dsh tool.",
    )
    .global(true)
    .hide_short_help(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

// Environment variable is defined in the dsh_api crate
const ENV_VAR_DSH_API_PLATFORMS_FILE: &str = "DSH_API_PLATFORMS_FILE";

pub(crate) const ENV_VAR_DSH_CLI_AUTHENTICATION: &str = "DSH_CLI_AUTHENTICATION";

pub(crate) const ENV_VAR_DSH_CLI_BROWSER: &str = "DSH_CLI_BROWSER";
pub(crate) const ENV_VAR_DSH_CLI_CSV_QUOTE: &str = "DSH_CLI_CSV_QUOTE";
pub(crate) const ENV_VAR_DSH_CLI_CSV_SEPARATOR: &str = "DSH_CLI_CSV_SEPARATOR";
pub(crate) const ENV_VAR_DSH_CLI_DRY_RUN: &str = "DSH_CLI_DRY_RUN";
pub(crate) const ENV_VAR_DSH_CLI_ENV_FILE: &str = "DSH_CLI_ENV_FILE";
pub(crate) const ENV_VAR_DSH_CLI_ERROR_COLOR: &str = "DSH_CLI_ERROR_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_ERROR_STYLE: &str = "DSH_CLI_ERROR_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_HOME: &str = "DSH_CLI_HOME";
pub(crate) const ENV_VAR_DSH_CLI_LABEL_COLOR: &str = "DSH_CLI_LABEL_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_LABEL_STYLE: &str = "DSH_CLI_LABEL_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_LOG_COLOR: &str = "DSH_CLI_LOG_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_LOG_LEVEL: &str = "DSH_CLI_LOG_LEVEL";
pub(crate) const ENV_VAR_DSH_CLI_LOG_LEVEL_API: &str = "DSH_CLI_LOG_LEVEL_API";
pub(crate) const ENV_VAR_DSH_CLI_LOG_STYLE: &str = "DSH_CLI_LOG_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_MATCHING_COLOR: &str = "DSH_CLI_MATCHING_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_MATCHING_STYLE: &str = "DSH_CLI_MATCHING_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_NO_CSV_HEADERS: &str = "DSH_CLI_NO_CSV_HEADERS";
pub(crate) const ENV_VAR_DSH_CLI_NO_ESCAPE: &str = "DSH_CLI_NO_ESCAPE";
pub(crate) const ENV_VAR_DSH_CLI_OUTPUT_FORMAT: &str = "DSH_CLI_OUTPUT_FORMAT";
pub(crate) const ENV_VAR_DSH_CLI_PASSWORD: &str = "DSH_CLI_PASSWORD";
pub(crate) const ENV_VAR_DSH_CLI_PASSWORD_FILE: &str = "DSH_CLI_PASSWORD_FILE";
pub(crate) const ENV_VAR_DSH_CLI_PLATFORM: &str = "DSH_CLI_PLATFORM";
pub(crate) const ENV_VAR_DSH_CLI_QUIET: &str = "DSH_CLI_QUIET";
pub(crate) const ENV_VAR_DSH_CLI_SHOW_EXECUTION_TIME: &str = "DSH_CLI_SHOW_EXECUTION_TIME";
pub(crate) const ENV_VAR_DSH_CLI_STDERR_COLOR: &str = "DSH_CLI_STDERR_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_STDERR_STYLE: &str = "DSH_CLI_STDERR_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_STDOUT_COLOR: &str = "DSH_CLI_STDOUT_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_STDOUT_STYLE: &str = "DSH_CLI_STDOUT_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_SUPPRESS_EXIT_STATUS: &str = "DSH_CLI_SUPPRESS_EXIT_STATUS";
pub(crate) const ENV_VAR_DSH_CLI_TARGET_COLOR: &str = "DSH_CLI_TARGET_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_TARGET_STYLE: &str = "DSH_CLI_TARGET_STYLE";
pub(crate) const ENV_VAR_DSH_CLI_TENANT: &str = "DSH_CLI_TENANT";
pub(crate) const ENV_VAR_DSH_CLI_TERMINAL_WIDTH: &str = "DSH_CLI_TERMINAL_WIDTH";
pub(crate) const ENV_VAR_DSH_CLI_VERBOSITY: &str = "DSH_CLI_VERBOSITY";
pub(crate) const ENV_VAR_DSH_CLI_WARNING_COLOR: &str = "DSH_CLI_WARNING_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_WARNING_STYLE: &str = "DSH_CLI_WARNING_STYLE";
pub(crate) const ENV_VAR_NO_COLOR: &str = "NO_COLOR";
pub(crate) const ENV_VAR_RUST_LOG: &str = "RUST_LOG";

lazy_static! {
  static ref EnvironmentVariables: [EnvironmentVariable; 40] = [
    EnvironmentVariable::new(
      ENV_VAR_DSH_API_PLATFORMS_FILE,
      "Overrides the default list of available platforms.",
      false,
      true,
      Some("built-in"),
      "Set this environment variable to override the default list of available platforms. The \n\
       value of the environment variable must be the name of the alternative platforms file. It \n\
       can either be an absolute file name, or a relative file name from the working directory. \n\
       When this environment variable is set, the normal list of default platforms will not be \n\
       included. If you need these too, make sure that you also include the default platforms in \n\
       your platforms file.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_AUTHENTICATION,
      "Specifies the authentication method.",
      false,
      true,
      None,
      "This environment variable specifies the authentication method that will be used to access \n\
      the resource management api. The allowed values are 'robot' and 'single-sign-on' (sso). If this \n\
      variable is not provided, the value from the settings file will be used, if it exists. Else, \n\
      the default value will be `single-sign-on` when the cli tool is run interactive ('stdin' is \n\
      a terminal) and 'robot' if not.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_BROWSER,
      "Specifies whether the tool will try to open a browser.",
      false,
      true,
      None,
      "This environment variable specifies whether the cli tool will try to automatically open a \n\
      browser (e.g. for authentication or to open the console) or will only instruct the user to \n\
      open it. The allowed values are 'instruct' and 'open'. If this variable is not provided, the \n\
      value from the settings file will be used, if it exists. Else, the default value will be \n\
      'open' when the cli tool is run interactive ('stdin' is a terminal) and 'instruct' if not.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_CSV_QUOTE,
      "Specifies the quote character that will be used when printing csv data.",
      false,
      true,
      Some("no quotes"),
      "This environment variable specifies the quote character that will be used when printing csv \n\
      data. If this variable is not provided, the value from the settings file will be used. The \n\
      default setting is not to use any quote characters. Note that the dsh tool will fail when \n\
      the generated output already contains the quote character.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_CSV_SEPARATOR,
      "Specifies the separator string that will be used when printing csv data.",
      false,
      true,
      Some(","),
      "This environment variable specifies the separator string that will be used when printing \n\
      csv data. If this variable is not provided, the value from the settings file will be used. \n\
      The default separator is ',' (comma). Note that the dsh tool will fail when the generated \n\
      output already contains the csv separator string.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_DRY_RUN,
      "Inhibits operations that could potentially make changes to the DSH platform, \
       like delete, create or change.",
      false,
      true,
      Some("execute"),
      "If this environment variable is set (to any value) the dsh tool will not call any api \n\
      operations that could potentially make changes, like delete, create or change. The input \n\
      parameters will be validated and checked. The same effect can be accomplished via the \n\
      --dry-run command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_ENV_FILE,
      "Specifies the location of a file that defines values for the environment variables used by \
      the dsh tool.",
      false,
      false,
      Some(DEFAULT_ENV_FILE_NAME),
      "This environment variable specifies the location of a file that defines values for the \n\
      environment variables used by the dsh tool. Variables defined in this file will override \n\
      the values of genuine environment variables set by the os shell, but can by themselves be \n\
      overridden by values defined using the --environment-variable command line argument. Note \n\
      that when the --env-var-file command line argument is provided, this environment variable \n\
      will not be used. The default location is .dsh_cli.env in the current working directory. \n\
      This environment variable itself cannot be overridden via the --environment-variable \n\
      command line argument, nor can it be defined in itself.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_ERROR_COLOR,
      "Specify the color to be used when printing error messages.",
      false,
      true,
      Some("red"),
      "This environment variable specifies the color to be used when printing error messages. If \n\
      this variable is not set, the settings file will be checked for the 'error-color' entry. \n\
      Else the default color 'red' will be used. The supported colors are: 'normal' (terminal \n\
      default), 'black', 'blue', 'cyan', 'green', 'magenta', 'red', 'white' and 'yellow'.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_ERROR_STYLE,
      "Specifies the styling to be used when printing error messages.",
      false,
      true,
      Some("bold"),
      "This environment variable specifies the styling to be used when printing error messages. \n\
      If this variable is not set, the settings file will be checked for the 'error-style' entry. \n\
      Else the default value 'bold' will be used. The supported styles are: 'normal' (no styling), \n\
      'bold', 'dim', 'italic', 'underline' or 'reverse'.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_HOME,
      "Specifies the location of the directory where the dsh tool stores its settings and \
       targets information.",
      false,
      false,
      Some("$HOME/.dsh_cli"),
      "Use this environment variable to change the location where dsh stores its settings and \n\
      targets information. The default location is $HOME/.dsh_cli. This environment variable \n\
      cannot be overridden via the --environment-variable command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_LABEL_COLOR,
      "Specify the color to be used when printing table headers or labels.",
      false,
      true,
      Some("blue"),
      "This environment variable specifies the color to be used when printing table headers or \n\
      labels. If this variable is not set, the settings file will be checked for the 'label-color' \n\
      entry. Else the default color 'blue' will be used. See environment variable \n\
      'DSH_CLI_ERROR_COLOR' for the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_LABEL_STYLE,
      "Specifies the styling to be used when printing table headers or labels.",
      false,
      true,
      Some("bold"),
      "This environment variable specifies the styling to be used when printing table headers or \n\
      labels. If this variable is not set, the settings file will be checked for the 'label-style' \n\
      entry. Else the default value 'bold' will be used. See environment variable \n\
      'DSH_CLI_ERROR_STYLE' for the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_LOG_COLOR,
      "Specify the color to be used when logging is enabled.",
      false,
      true,
      Some("red"),
      "This environment variable specifies the color to be used when printing logging information. \n\
      If this variable is not set, the settings file will be checked for the 'log-color' entry. \n\
      Else the default color 'red' will be used. See environment variable 'DSH_CLI_ERROR_COLOR' \n\
      for the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_LOG_LEVEL,
      "Specifies the log level of the dsh tool.",
      false,
      true,
      Some("error"),
      "Use this environment variable to set the log level of the dsh tool. The available log \n\
       levels are: off, error, warn, info, debug or trace. If this argument is not provided, the \n\
       settings file will be checked. When the --log-level command line argument is provided this \n\
       will override this environment variable or the value in the settings file. The default log \n\
       level is 'error'.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_LOG_LEVEL_API,
      "Specifies the log level for the dsh_api library functions.",
      false,
      true,
      Some("error"),
      "Use this environment variable to set the log level for the functions in the library crate \n\
      dsh_api, that supports the dsh tool. For the available log levels see the description of the \n\
      DSH_CLI_LOG_LEVEL environment variable. If this argument is not provided, the settings file \n\
      will be checked. When the --log-level-api command line argument is provided this will \n\
      override this environment variable or the value in the settings file. The default log level \n\
      is 'error'.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_LOG_STYLE,
      "Specifies the styling to be used when logging is enabled.",
      false,
      true,
      Some("dim"),
      "This environment variable specifies the styling to be used when printing logging \n\
      information. If this variable is not set, the settings file will be checked for the \n\
      'log-style' entry. Else the default value 'dim' will be used. See environment variable \n\
      'DSH_CLI_ERROR_STYLE' for the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_MATCHING_COLOR,
      "Specifies the color to be used when printing matching results for the find functions.",
      false,
      true,
      Some("green"),
      "This environment variable specifies the color to be used when printing matching results for \n\
      the find functions, e.q. when matching regular expressions. If this variable is not set, the \n\
      settings file will be checked for the 'matching-color' entry. Else the default color 'green' \n\
      will be used. See environment variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_MATCHING_STYLE,
      "Specifies the styling to be used when printing matching results for the find functions.",
      false,
      true,
      Some("bold"),
      "This environment variable specifies the styling to be used when printing matching results \n\
      for the find functions, e.q. when matching regular expressions. If this variable is not set, \n\
      the settings file will be checked for the 'matching-style' entry. Else the default value \n\
      'bold' will be used. See environment variable 'DSH_CLI_ERROR_STYLE' for the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_NO_CSV_HEADERS,
      "Disables headers in csv output.",
      false,
      true,
      Some("enabled"),
      "When this environment variables is set (to any value) csv output will not contain headers. \n\
      This environment variable can be overridden via the --no-csv-headers command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_NO_ESCAPE,
      "Disables color and styling ansi escape sequences in the generated output.",
      false,
      true,
      Some("enabled"),
      "When this environment variable is set (to any value) the output will not contain any ansi \n\
      color or other escape sequences. This environment variable can be overridden via the \n\
      --no-color or --no-ansi command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_OUTPUT_FORMAT,
      "Specifies the format used when printing the output.",
      false,
      true,
      Some("table / json"),
      "This option specifies the format used when printing the output. If this argument is not \n\
      provided, the value from the settings file will be used. Else, when stdout is a terminal the \n\
      default 'table' will be used, or if 'stdout' is not a terminal the value 'json' will be \n\
      used. The supported values are: 'csv', 'json', 'json-compact', 'plain', 'quiet', 'table', \n
      'table-no-border', 'toml', 'toml-compact' or 'yaml'. This environment variable can be \n\
      overridden via the --output-format command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_PASSWORD,
      "Specifies the secret api token/password for the target tenant.",
      true,
      false,
      None,
      "This environment variable specifies the secret api token/password for the target tenant. \n\
      Note that when the environment variable 'DSH_CLI_PASSWORD_FILE' or the argument \n\
      --password-file command line argument is provided, this environment variable will never be \n\
      used. For better security, consider using one of these two options instead of defining \n\
      'DSH_CLI_PASSWORD'. This environment variable cannot be overridden via the \n\
      --environment-variable command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_PASSWORD_FILE,
      "Specifies the location of a file containing the secret api token/password \
       for the target tenant.",
      false,
      true,
      None,
      "This environment variable specifies a file containing the secret api token/password for the \n\
      target tenant. Note that when the --password-file command line argument is provided, this \n\
      environment variable will not be used.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_PLATFORM,
      "Specifies the target platform on which the target tenant environments live.",
      false,
      true,
      None,
      "Target platform on which the tenants environment lives. The supported platforms are: \n\
      'np-aws-lz-dsh' / 'nplz', 'poc-aws-dsh' / 'poc', 'prod-aws-dsh' / 'prod', \n\
      'prod-aws-lz-dsh' / 'prodlz', 'prod-aws-lz-laas' / 'prodls' or 'prod-azure-dsh' / 'prodaz'. \n\
      This environment variable can be overridden via the --platform command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_QUIET,
      "Enables quiet mode, which means that no output will be produced to the terminal.",
      false,
      true,
      Some("print"),
      "When this environment variable is set (to any value) the dsh tool will run in quiet mode, \n\
      meaning that no output will be produced to the terminal (stdout and stderr). This \n\
      environment variable can be overridden via the --quiet command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_SHOW_EXECUTION_TIME,
      "Enables printing the execution time of the executed api functions, in milliseconds.",
      false,
      true,
      Some("disabled"),
      "When this environment variable is set (to any value) the execution time of the executed \n\
      function will be shown, in milliseconds. The execution time will also be shown when the \n\
      verbosity level is set to 'high'. This environment variable can be overridden via the \n\
      '--show-execution-time' command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_STDERR_COLOR,
      "Specifies the color to be used when printing explanations and metadata.",
      false,
      true,
      Some("terminal default"),
      "This environment variable specifies the color to be used when printing explanations and \n\
      metadata. If this variable is not set, the settings file will be checked for the \n\
      'stderr-color' entry. Else the default color for the terminal will be used. See environment \n\
      variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_STDERR_STYLE,
      "Specifies the styling to be used when printing explanations and metadata.",
      false,
      true,
      Some("dim"),
      "This environment variable specifies the styling to be used when printing explanations and \n\
      metadata. If this variable is not set, the settings file will be checked for the \n\
      'stderr-style' entry. Else the default value 'dim' will be used. See environment variable \n\
      'DSH_CLI_ERROR_STYLE' for the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_STDOUT_COLOR,
      "Specifies the color to be used when printing results.",
      false,
      true,
      Some("terminal default"),
      "This environment variable specifies the color to be used when printing results. If this \n\
      variable is not set, the settings file will be checked for the 'stdout-color' entry. Else \n\
      the default color for the terminal will be used. See environment variable \n\
      'DSH_CLI_ERROR_COLOR' for the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_STDOUT_STYLE,
      "Specifies the styling to be used when printing results.",
      false,
      true,
      Some("normal"),
      "This environment variable specifies the styling to be used when printing results. If this \n\
      variable is not set, the settings file will be checked for the 'stdout-style' entry. Else \n\
      the default value 'normal' (no styling) will be used. See environment variable \n\
      'DSH_CLI_ERROR_STYLE' for the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_SUPPRESS_EXIT_STATUS,
      "Suppress the returned exit status of the tool (will always be 0).",
      false,
      true,
      Some("return status"),
      "If this environment variable is set (to any value) the dsh tool will always return exit \n\
      status 0, even when an error has occurred. This can be useful in scripting environments. The \n\
      same effect can be accomplished via the '--suppress-exit-code' command line argument or the \n\
      'suppress-exit-status' setting.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_TARGET_COLOR,
      "Specify the color to be used when printing target identifiers.",
      false,
      true,
      Some("terminal default"),
      "This environment variable specifies the color to be used when printing target \n\
      identifiers. If this variable is not set, the settings file will be checked for the \n\
      'target-color' entry. Else the default terminal color will be used. See environment \n\
      variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_TARGET_STYLE,
      "Specifies the styling to be used when printing target identifiers.",
      false,
      true,
      Some("italic"),
      "This environment variable specifies the styling to be used when printing table headers or \n\
      labels. If this variable is not set, the settings file will be checked for the 'label-style' \n\
      entry. Else the default value 'italic' will be used. See environment variable \n\
      'DSH_CLI_ERROR_STYLE' for the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_TENANT,
      "Specifies the target tenant name.",
      false,
      true,
      None,
      "Tenant name for the target tenant. The target tenant is the tenant whose resources will \n\
      be managed via the api. This environment variable can be overridden via the --tenant \n\
      command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_TERMINAL_WIDTH,
      "Specifies the maximum terminal width.",
      false,
      true,
      Some("actual width"),
      "When this environment variable is set it will define the maximum terminal width. This \n\
      environment variable can be overridden via the --terminal-width command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_VERBOSITY,
      "Specifies the verbosity level of the dsh tool.",
      false,
      true,
      Some("low"),
      "If this option is provided, it will set the verbosity level. The default verbosity setting \n\
      is 'low'. The supported verbosity levels are: off, low, medium or high. This environment \n\
      variable can be overridden via the '--verbosity' command line argument. Also, when the \n\
      environment variable 'DSH_CLI_QUIET' is set or the command line argument '--quiet' is \n\
      provided, nothing will be printed.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_WARNING_COLOR,
      "Specifies the color to be used when printing warnings.",
      false,
      true,
      Some("blue"),
      "This environment variable specifies the color to be used when printing warnings. If this \n\
      variable is not set, the settings file will be checked for the 'warning-color' entry. Else \n\
      the default color 'blue' will be used. See environment variable 'DSH_CLI_ERROR_COLOR' for \n\
      the supported colors.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_DSH_CLI_WARNING_STYLE,
      "Specifies the styling to be used when printing warnings.",
      false,
      true,
      Some("bold"),
      "This environment variable specifies the styling to be used when printing warnings. If this \n\
      variable is not set, the settings file will be checked for the 'warning-style' entry. Else \n\
      the default value 'bold' will be used. See environment variable 'DSH_CLI_ERROR_STYLE' for \n\
      the supported styles.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_NO_COLOR,
      "Disables color and styling escape sequences in the generated output.",
      false,
      true,
      Some("enabled"),
      "When this environment variable is set (to any value) the output will not contain any color \n\
      or other escape sequences. This environment variable can be overridden via the '--no-color' \n\
      or '--no-ansi' command line argument.",
    ),
    EnvironmentVariable::new(
      ENV_VAR_RUST_LOG,
      "Specifies the log level of the rust env_logger crate.",
      false,
      true,
      Some("off"),
      "Since the dsh tool depends on the env_logger crate for its logging, it also recognizes log \n\
      configuration via the RUST_LOG environment variable. Although the use of this variable is \n\
      not recommended, there might be situations when this can be useful. See the crate's github \n\
      repository for more information.",
    ),
  ];
}
