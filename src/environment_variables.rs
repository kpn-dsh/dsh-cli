use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::formatters::{Label, SubjectFormatter};
use crate::TOOL_OPTIONS_HEADING;
use clap::{builder, Arg, ArgAction};
use itertools::Itertools;
use serde::Serialize;
use std::env;

// Environment variable is defined in the dsh_api crate
const ENV_VAR_DSH_API_PLATFORMS_FILE: &str = "DSH_API_PLATFORMS_FILE";

pub(crate) const ENV_VAR_DSH_CLI_AUTHENTICATION: &str = "DSH_CLI_AUTHENTICATION";
pub(crate) const ENV_VAR_DSH_CLI_BROWSER: &str = "DSH_CLI_BROWSER";
pub(crate) const ENV_VAR_DSH_CLI_CSV_QUOTE: &str = "DSH_CLI_CSV_QUOTE";
pub(crate) const ENV_VAR_DSH_CLI_CSV_SEPARATOR: &str = "DSH_CLI_CSV_SEPARATOR";
pub(crate) const ENV_VAR_DSH_CLI_DRY_RUN: &str = "DSH_CLI_DRY_RUN";
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
pub(crate) const ENV_VAR_DSH_CLI_NO_ESCAPE: &str = "DSH_CLI_NO_ESCAPE";
pub(crate) const ENV_VAR_DSH_CLI_NO_HEADERS: &str = "DSH_CLI_NO_HEADERS";
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
pub(crate) const ENV_VAR_DSH_CLI_TENANT: &str = "DSH_CLI_TENANT";
pub(crate) const ENV_VAR_DSH_CLI_TERMINAL_WIDTH: &str = "DSH_CLI_TERMINAL_WIDTH";
pub(crate) const ENV_VAR_DSH_CLI_VERBOSITY: &str = "DSH_CLI_VERBOSITY";
pub(crate) const ENV_VAR_DSH_CLI_WARNING_COLOR: &str = "DSH_CLI_WARNING_COLOR";
pub(crate) const ENV_VAR_DSH_CLI_WARNING_STYLE: &str = "DSH_CLI_WARNING_STYLE";
pub(crate) const ENV_VAR_NO_COLOR: &str = "NO_COLOR";
pub(crate) const ENV_VAR_RUST_LOG: &str = "RUST_LOG";

/// Returns the defined environment variables that are currently set
pub(crate) fn get_set_environment_variables() -> Vec<(String, String)> {
  let mut environment_variables: Vec<(String, String)> = vec![];
  for (os_env_var_name, os_env_var_value) in env::vars() {
    if ENVIRONMENT_VARIABLES.iter().any(|(defined_env_var, _, _, _)| defined_env_var == &os_env_var_name) {
      environment_variables.push((os_env_var_name, os_env_var_value));
    }
  }
  environment_variables.sort_by(|(env_var_a, _), (env_var_b, _)| env_var_a.cmp(env_var_b));
  environment_variables
}

pub(crate) fn print_environment_variables(context: &Context) {
  let mut formatter = ListFormatter::new(&ENV_VAR_LABELS_LIST, None, context);
  let styled: Vec<(String, String, Option<&str>, &str, &str)> = ENVIRONMENT_VARIABLES
    .into_iter()
    .map(|(env_var, short_explanation, default_value, long_explanation)| {
      (
        context.apply_label_style_for_stdout(env_var, Some(OutputFormat::Table)),
        env::var(env_var).unwrap_or_default(),
        default_value,
        short_explanation,
        long_explanation,
      )
    })
    .collect_vec();
  formatter.push_values(&styled);
  _ = formatter.print(Some(OutputFormat::Table));
}

pub(crate) fn print_environment_variable(env_var: &str, context: &Context) {
  let matching_env_vars: Vec<&(&str, &str, Option<&str>, &str)> = ENVIRONMENT_VARIABLES
    .iter()
    .filter(|(defined_env_var, _, _, _)| defined_env_var.contains(&env_var.to_uppercase()))
    .collect_vec();
  if matching_env_vars.is_empty() {
    context.print_warning(format!("'{}' could not be matched to an environment variable recognized by the dsh tool", env_var));
  } else if matching_env_vars.len() == 1 {
    let (env_var, short_explanation, default_value, long_explanation) = matching_env_vars.first().unwrap_or_else(|| unreachable!());
    _ = UnitFormatter::new(*env_var, &ENV_VAR_LABELS, None, context).print(
      &(
        env_var.to_string(),
        env::var(env_var).unwrap_or_default(),
        *default_value,
        *short_explanation,
        *long_explanation,
      ),
      None,
    );
  } else {
    let mut formatter = ListFormatter::new(&ENV_VAR_LABELS, None, context);
    let styled_matching_env_vars: Vec<(String, String, Option<&str>, &str, &str)> = matching_env_vars
      .into_iter()
      .map(|(env_var, short_explanation, default_value, long_explanation)| {
        (
          context.apply_label_style_for_stdout(env_var, Some(OutputFormat::Table)),
          env::var(env_var).unwrap_or_default(),
          *default_value,
          *short_explanation,
          *long_explanation,
        )
      })
      .collect_vec();
    formatter.push_values(&styled_matching_env_vars);
    _ = formatter.print(Some(OutputFormat::Table));
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum EnvVarLabel {
  DefaultValue,
  EnvVar,
  LongExplanation,
  ShortExplanation,
  Value,
}

impl Label for EnvVarLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DefaultValue => "default value",
      Self::EnvVar => "environment variable",
      Self::LongExplanation => "explanation",
      Self::ShortExplanation => "short explanation",
      Self::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::EnvVar)
  }
}

// Tuple:
// * env_var name
// * value
// * default_value
// * short_explanation
// * long_explanation
impl SubjectFormatter<EnvVarLabel> for (String, String, Option<&str>, &str, &str) {
  fn value(&self, label: &EnvVarLabel, _: &str) -> String {
    let (env_var, value, default_value, short_explanation, long_explanation) = self;
    match label {
      EnvVarLabel::DefaultValue => default_value.map(|value| value.to_string()).unwrap_or_default(),
      EnvVarLabel::EnvVar => env_var.to_string(),
      EnvVarLabel::LongExplanation => {
        if long_explanation.is_empty() {
          short_explanation.to_string()
        } else {
          long_explanation.to_string()
        }
      }
      EnvVarLabel::ShortExplanation => short_explanation.to_string(),
      EnvVarLabel::Value => value.to_string(),
    }
  }
}

const ENV_VAR_LABELS: [EnvVarLabel; 4] = [EnvVarLabel::EnvVar, EnvVarLabel::Value, EnvVarLabel::DefaultValue, EnvVarLabel::LongExplanation];
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

// Tuples describing environment variables:
// * Environment variable name
// * Short explanation
// * Optional default value as string
// * Long explanation
const ENVIRONMENT_VARIABLES: [(&str, &str, Option<&str>, &str); 37] = [
  (
    ENV_VAR_DSH_API_PLATFORMS_FILE,
    "Overrides the default list of available platforms.",
    Some("built-in"),
    "Set this environment variable to override the default list of available platforms. The \n\
     value of the environment variable must be the name of the alternative platforms file. It \n\
     can either be an absolute file name, or a relative file name from the working directory. \n\
     When this environment variable is set, the normal list of default platforms will not be \n\
     included. If you need these too, make sure that you also include the default platforms in \n\
     your platforms file.",
  ),
  (
    ENV_VAR_DSH_CLI_AUTHENTICATION,
    "Specifies the authentication method.",
    None,
    "This environment variable specifies the authentication method that will be used to access \n\
    the resource management api. The allowed values are 'robot' and 'single-sign-on'. If this \n\
    variable is not provided, the value from the settings file will be used, if it exists. Else, \n\
    the default value will be `single-sign-on` when the cli tool is run interactive ('stdin' is \n\
    a terminal) and 'robot' if not.",
  ),
  (
    ENV_VAR_DSH_CLI_BROWSER,
    "Specifies whether the tool will try to open a browser.",
    None,
    "This environment variable specifies whether the cli tool will try to automatically open a \n\
    browser (e.g. for authentication or to open the console) or will only instruct the user to \n\
    open it. The allowed values are 'instruct' and 'open'. If this variable is not provided, the \n\
    value from the settings file will be used, if it exists. Else, the default value will be \n\
    'open' when the cli tool is run interactive ('stdin' is a terminal) and 'instruct' if not.",
  ),
  (
    ENV_VAR_DSH_CLI_CSV_QUOTE,
    "Specifies the quote character that will be used when printing csv data.",
    Some("no quotes"),
    "This environment variable specifies the quote character that will be used when printing csv \n\
    data. If this variable is not provided, the value from the settings file will be used. The \n\
    default setting is not to use any quote characters. Note that the dsh tool will fail when \n\
    the generated output already contains the quote character.",
  ),
  (
    ENV_VAR_DSH_CLI_CSV_SEPARATOR,
    "Specifies the separator string that will be used when printing csv data.",
    Some(","),
    "This environment variable specifies the separator string that will be used when printing \n\
    csv data. If this variable is not provided, the value from the settings file will be used. \n\
    The default separator is ',' (comma). Note that the dsh tool will fail when the generated \n\
    output already contains the csv separator string.",
  ),
  (
    ENV_VAR_DSH_CLI_DRY_RUN,
    "Inhibits api operations that could potentially make changes to the DSH platform, \
     like delete, create or change.",
    Some("execute operation"),
    "If this environment variable is set (to any value) the dsh tool will not call any api \n\
    operations that could potentially make changes, like delete, create or change. The input \n\
    parameters will be validated and checked. The same effect can be accomplished via the \n\
    --dry-run command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_ERROR_COLOR,
    "Specify the color to be used when printing error messages.",
    Some("red"),
    "This environment variable specifies the color to be used when printing error messages. If \n\
    this variable is not set, the settings file will be checked for the 'error-color' entry. \n\
    Else the default color 'red' will be used. The supported colors are: 'normal' (terminal \n\
    default), 'black', 'blue', 'cyan', 'green', 'magenta', 'red', 'white' and 'yellow'.",
  ),
  (
    ENV_VAR_DSH_CLI_ERROR_STYLE,
    "Specifies the styling to be used when printing error messages.",
    Some("bold"),
    "This environment variable specifies the styling to be used when printing error messages. \n\
    If this variable is not set, the settings file will be checked for the 'error-style' entry. \n\
    Else the default value 'bold' will be used. The supported styles are: 'normal' (no styling), \n\
    'bold', 'dim', 'italic', 'underline' or 'reverse'.",
  ),
  (
    ENV_VAR_DSH_CLI_HOME,
    "Specifies the location of the directory where the dsh tool stores its settings and \
     targets information.",
    Some("$HOME/.dsh_cli"),
    "Use this environment variable to change the location where dsh stores its settings and \n\
    targets information. The default location is $HOME/.dsh_cli. This environment variable \n\
    cannot be overridden via the --environment-variable command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_LABEL_COLOR,
    "Specify the color to be used when printing table headers or labels.",
    Some("blue"),
    "This environment variable specifies the color to be used when printing table headers or \n\
    labels. If this variable is not set, the settings file will be checked for the 'label-color' \n\
    entry. Else the default color 'blue' will be used. See environment variable \n\
    'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_DSH_CLI_LABEL_STYLE,
    "Specifies the styling to be used when printing table headers or labels.",
    Some("bold"),
    "This environment variable specifies the styling to be used when printing table headers or \n\
    labels. If this variable is not set, the settings file will be checked for the 'label-style' \n\
    entry. Else the default value 'bold' will be used. See environment variable \n\
    'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_DSH_CLI_LOG_COLOR,
    "Specify the color to be used when logging is enabled.",
    Some("cyan"),
    "This environment variable specifies the color to be used when printing logging information. \n\
    If this variable is not set, the settings file will be checked for the 'log-color' entry. \n\
    Else the default color 'cyan' will be used. See environment variable 'DSH_CLI_ERROR_COLOR' \n\
    for the supported colors.",
  ),
  (
    ENV_VAR_DSH_CLI_LOG_LEVEL,
    "Specifies the log level of the dsh tool.",
    Some("error"),
    "Use this environment variable to set the log level of the dsh tool. The available log \n\
     levels are: off, error, warn, info, debug or trace. If this argument is not provided, the \n\
     settings file will be checked. When the --log-level command line argument is provided this \n\
     will override this environment variable or the value in the settings file. The default log \n\
     level is 'error'.",
  ),
  (
    ENV_VAR_DSH_CLI_LOG_LEVEL_API,
    "Specifies the log level for the dsh_api library functions.",
    Some("error"),
    "Use this environment variable to set the log level for the functions in the library crate \n\
    dsh_api, that supports the dsh tool. For the available log levels see the description of the \n\
    DSH_CLI_LOG_LEVEL environment variable. If this argument is not provided, the settings file \n\
    will be checked. When the --log-level-api command line argument is provided this will \n\
    override this environment variable or the value in the settings file. The default log level \n\
    is 'error'.",
  ),
  (
    ENV_VAR_DSH_CLI_LOG_STYLE,
    "Specifies the styling to be used when logging is enabled.",
    Some("normal"),
    "This environment variable specifies the styling to be used when printing logging \n\
    information. If this variable is not set, the settings file will be checked for the \n\
    'log-style' entry. Else the default value 'dim' will be used. See environment variable \n\
    'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_DSH_CLI_MATCHING_COLOR,
    "Specifies the color to be used when printing matching results for the find functions.",
    Some("green"),
    "This environment variable specifies the color to be used when printing matching results for \n\
    the find functions, e.q. when matching regular expressions. If this variable is not set, the \n\
    settings file will be checked for the 'matching-color' entry. Else the default color 'green' \n\
    will be used. See environment variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_DSH_CLI_MATCHING_STYLE,
    "Specifies the styling to be used when printing matching results for the find functions.",
    Some("bold"),
    "This environment variable specifies the styling to be used when printing matching results \n\
    for the find functions, e.q. when matching regular expressions. If this variable is not set, \n\
    the settings file will be checked for the 'matching-style' entry. Else the default value \n\
    'bold' will be used. See environment variable 'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_DSH_CLI_NO_ESCAPE,
    "Disables color and styling ansi escape sequences in the generated output.",
    Some("ansi enabled"),
    "When this environment variable is set (to any value) the output will not contain any ansi \n\
    color or other escape sequences. This environment variable can be overridden via the \n\
    --no-color or --no-ansi command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_NO_HEADERS,
    "Disables headers in the generated output.",
    Some("headers enabled"),
    "When this environment variables is set (to any value) the output will not contain headers. \n\
    This environment variable can be overridden via the --no-headers command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_OUTPUT_FORMAT,
    "Specifies the format used when printing the output.",
    Some("table / json"),
    "This option specifies the format used when printing the output. If this argument is not \n\
    provided, the value from the settings file will be used. Else, when stdout is a terminal the \n\
    default 'table' will be used, or if 'stdout' is not a terminal the value 'json' will be \n\
    used. The supported values are: 'csv', 'json', 'json-compact', 'plain', 'quiet', 'table', \n
    'table-no-border', 'toml', 'toml-compact' or 'yaml'. This environment variable can be \n\
    overridden via the --output-format command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_PASSWORD,
    "Specifies the secret api token/password for the target tenant.",
    None,
    "This environment variable specifies the secret api token/password for the target tenant. \n\
    Note that when the environment variable 'DSH_CLI_PASSWORD_FILE' or the argument \n\
    --password-file command line argument is provided, this environment variable will never be \n\
    used. For better security, consider using one of these two options instead of defining \n\
    'DSH_CLI_PASSWORD'. This environment variable cannot be overridden via the \n\
    --environment-variable command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_PASSWORD_FILE,
    "Specifies the location of a file containing the secret api token/password \
     for the target tenant.",
    None,
    "This environment variable specifies a file containing the secret api token/password for the \n\
    target tenant. Note that when the --password-file command line argument is provided, this \n\
    environment variable will not be used.",
  ),
  (
    ENV_VAR_DSH_CLI_PLATFORM,
    "Specifies the target platform on which the target tenant environments live.",
    None,
    "Target platform on which the tenants environment lives. The supported platforms are: \n\
    'np-aws-lz-dsh' / 'nplz', 'poc-aws-dsh' / 'poc', 'prod-aws-dsh' / 'prod', \n\
    'prod-aws-lz-dsh' / 'prodlz', 'prod-aws-lz-laas' / 'prodls' or 'prod-azure-dsh' / 'prodaz'. \n\
    This environment variable can be overridden via the --platform command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_QUIET,
    "Enables quiet mode, which means that no output will be produced to the terminal.",
    Some("output produced"),
    "When this environment variable is set (to any value) the dsh tool will run in quiet mode, \n\
    meaning that no output will be produced to the terminal (stdout and stderr). This \n\
    environment variable can be overridden via the --quit command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_SHOW_EXECUTION_TIME,
    "Enables printing the execution time of the executed api functions, in milliseconds.",
    Some("not shown"),
    "When this environment variable is set (to any value) the execution time of the executed \n\
    function will be shown, in milliseconds. The execution time will also be shown when the \n\
    verbosity level is set to 'high'. This environment variable can be overridden via the \n\
    '--show-execution-time' command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_STDERR_COLOR,
    "Specifies the color to be used when printing explanations and metadata.",
    Some("terminal default"),
    "This environment variable specifies the color to be used when printing explanations and \n\
    metadata. If this variable is not set, the settings file will be checked for the \n\
    'stderr-color' entry. Else the default color for the terminal will be used. See environment \n\
    variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_DSH_CLI_STDERR_STYLE,
    "Specifies the styling to be used when printing explanations and metadata.",
    Some("dim"),
    "This environment variable specifies the styling to be used when printing explanations and \n\
    metadata. If this variable is not set, the settings file will be checked for the \n\
    'stderr-style' entry. Else the default value 'dim' will be used. See environment variable \n\
    'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_DSH_CLI_STDOUT_COLOR,
    "Specifies the color to be used when printing results.",
    Some("terminal default"),
    "This environment variable specifies the color to be used when printing results. If this \n\
    variable is not set, the settings file will be checked for the 'stdout-color' entry. Else \n\
    the default color for the terminal will be used. See environment variable \n\
    'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_DSH_CLI_STDOUT_STYLE,
    "Specifies the styling to be used when printing results.",
    Some("normal"),
    "This environment variable specifies the styling to be used when printing results. If this \n\
    variable is not set, the settings file will be checked for the 'stdout-style' entry. Else \n\
    the default value 'normal' (no styling) will be used. See environment variable \n\
    'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_DSH_CLI_SUPPRESS_EXIT_STATUS,
    "Suppress the returned exit status of the tool (will always be 0).",
    Some("not suppressed"),
    "If this environment variable is set (to any value) the dsh tool will always return exit \n\
    status 0, even when an error has occurred. This can be useful in scripting environments. The \n\
    same effect can be accomplished via the '--suppress-exit-code' command line argument or the \n\
    'suppress-exit-status' setting.",
  ),
  (
    ENV_VAR_DSH_CLI_TERMINAL_WIDTH,
    "Specifies the maximum terminal width.",
    Some("actual width"),
    "When this environment variable is set it will define the maximum terminal width. This \n\
    environment variable can be overridden via the --terminal-width command line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_TENANT,
    "Specifies the target tenant.",
    None,
    "Tenant id for the target tenant. The target tenant is the tenant whose resources will be \n\
    managed via the api. This environment variable can be overridden via the --tenant command \n\
    line argument.",
  ),
  (
    ENV_VAR_DSH_CLI_VERBOSITY,
    "Specifies the verbosity level of the dsh tool.",
    Some("low"),
    "If this option is provided, it will set the verbosity level. The default verbosity setting \n\
    is 'low'. The supported verbosity levels are: off, low, medium or high. This environment \n\
    variable can be overridden via the '--verbosity' command line argument. Also, when the \n\
    environment variable 'DSH_CLI_QUIET' is set or the command line argument '--quiet' is \n\
    provided, nothing will be printed.",
  ),
  (
    ENV_VAR_DSH_CLI_WARNING_COLOR,
    "Specifies the color to be used when printing warnings.",
    Some("blue"),
    "This environment variable specifies the color to be used when printing warnings. If this \n\
    variable is not set, the settings file will be checked for the 'warning-color' entry. Else \n\
    the default color 'blue' will be used. See environment variable 'DSH_CLI_ERROR_COLOR' for \n\
    the supported colors.",
  ),
  (
    ENV_VAR_DSH_CLI_WARNING_STYLE,
    "Specifies the styling to be used when printing warnings.",
    Some("bold"),
    "This environment variable specifies the styling to be used when printing warnings. If this \n\
    variable is not set, the settings file will be checked for the 'warning-style' entry. Else \n\
    the default value 'bold' will be used. See environment variable 'DSH_CLI_ERROR_STYLE' for \n\
    the supported styles.",
  ),
  (
    ENV_VAR_NO_COLOR,
    "Disables color and styling escape sequences in the generated output.",
    Some("colors enabled"),
    "When this environment variable is set (to any value) the output will not contain any color \n\
    or other escape sequences. This environment variable can be overridden via the '--no-color' \n\
    or '--no-ansi' command line argument.",
  ),
  (
    ENV_VAR_RUST_LOG,
    "Specifies the log level of the rust env_logger crate.",
    Some("disabled"),
    "Since the dsh tool depends on the env_logger crate for its logging, it also recognizes log \n\
    configuration via the RUST_LOG environment variable. Although the use of this variable is \n\
    not recommended, there might be situations when this can be useful. See the crate's github \n\
    repository for more information.",
  ),
];
