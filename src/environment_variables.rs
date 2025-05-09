use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::OutputFormat;
use clap::{builder, Arg, ArgAction};
use serde::Serialize;
use std::env;

// Environment variable is defined in the dsh_api crate
const ENV_VAR_PLATFORMS_FILE_NAME: &str = "DSH_API_PLATFORMS_FILE";

pub(crate) const ENV_VAR_CSV_QUOTE: &str = "DSH_CLI_CSV_QUOTE";
pub(crate) const ENV_VAR_CSV_SEPARATOR: &str = "DSH_CLI_CSV_SEPARATOR";
pub(crate) const ENV_VAR_DRY_RUN: &str = "DSH_CLI_DRY_RUN";
pub(crate) const ENV_VAR_ERROR_COLOR: &str = "DSH_CLI_ERROR_COLOR";
pub(crate) const ENV_VAR_ERROR_STYLE: &str = "DSH_CLI_ERROR_STYLE";
pub(crate) const ENV_VAR_HOME_DIRECTORY: &str = "DSH_CLI_HOME";
pub(crate) const ENV_VAR_LABEL_COLOR: &str = "DSH_CLI_LABEL_COLOR";
pub(crate) const ENV_VAR_LABEL_STYLE: &str = "DSH_CLI_LABEL_STYLE";
pub(crate) const ENV_VAR_LOG_LEVEL: &str = "DSH_CLI_LOG_LEVEL";
pub(crate) const ENV_VAR_LOG_LEVEL_API: &str = "DSH_CLI_LOG_LEVEL_API";
pub(crate) const ENV_VAR_MATCHING_COLOR: &str = "DSH_CLI_MATCHING_COLOR";
pub(crate) const ENV_VAR_MATCHING_STYLE: &str = "DSH_CLI_MATCHING_STYLE";
pub(crate) const ENV_VAR_NO_COLOR: &str = "NO_COLOR";
pub(crate) const ENV_VAR_NO_ESCAPE: &str = "DSH_CLI_NO_ESCAPE";
pub(crate) const ENV_VAR_NO_HEADERS: &str = "DSH_CLI_NO_HEADERS";
pub(crate) const ENV_VAR_OUTPUT_FORMAT: &str = "DSH_CLI_OUTPUT_FORMAT";
pub(crate) const ENV_VAR_PASSWORD: &str = "DSH_CLI_PASSWORD";
pub(crate) const ENV_VAR_PASSWORD_FILE: &str = "DSH_CLI_PASSWORD_FILE";
pub(crate) const ENV_VAR_PLATFORM: &str = "DSH_CLI_PLATFORM";
pub(crate) const ENV_VAR_QUIET: &str = "DSH_CLI_QUIET";
pub(crate) const ENV_VAR_RUST_LOG: &str = "RUST_LOG";
pub(crate) const ENV_VAR_SHOW_EXECUTION_TIME: &str = "DSH_CLI_SHOW_EXECUTION_TIME";
pub(crate) const ENV_VAR_STDERR_COLOR: &str = "DSH_CLI_STDERR_COLOR";
pub(crate) const ENV_VAR_STDERR_STYLE: &str = "DSH_CLI_STDERR_STYLE";
pub(crate) const ENV_VAR_STDOUT_COLOR: &str = "DSH_CLI_STDOUT_COLOR";
pub(crate) const ENV_VAR_STDOUT_STYLE: &str = "DSH_CLI_STDOUT_STYLE";
pub(crate) const ENV_VAR_SUPPRESS_EXIT_STATUS: &str = "DSH_CLI_SUPPRESS_EXIT_STATUS";
pub(crate) const ENV_VAR_TENANT: &str = "DSH_CLI_TENANT";
pub(crate) const ENV_VAR_TERMINAL_WIDTH: &str = "DSH_CLI_TERMINAL_WIDTH";
pub(crate) const ENV_VAR_VERBOSITY: &str = "DSH_CLI_VERBOSITY";
pub(crate) const ENV_VAR_WARNING_COLOR: &str = "DSH_CLI_WARNING_COLOR";
pub(crate) const ENV_VAR_WARNING_STYLE: &str = "DSH_CLI_WARNING_STYLE";

/// Returns the defined environment variables that are currently set
pub(crate) fn get_set_environment_variables() -> Vec<(String, String)> {
  let mut environment_variables: Vec<(String, String)> = vec![];
  for (os_env_var_name, os_env_var_value) in env::vars() {
    if ENVIRONMENT_VARIABLES.iter().any(|(defined_env_var, _, _)| defined_env_var == &os_env_var_name) {
      environment_variables.push((os_env_var_name, os_env_var_value));
    }
  }
  environment_variables.sort_by(|(env_var_a, _), (env_var_b, _)| env_var_a.cmp(env_var_b));
  environment_variables
}

pub(crate) fn print_environment_variables(context: &Context) {
  let mut formatter = ListFormatter::new(&[EnvVarLabel::Name, EnvVarLabel::Value, EnvVarLabel::ShortExplanation], None, context);
  let styled: Vec<(String, String, &str, &str)> = ENVIRONMENT_VARIABLES
    .into_iter()
    .map(|(env_var, short_explanation, explanation)| {
      (
        context.apply_label_style_for_stdout(env_var, Some(OutputFormat::Table)),
        env::var(env_var).unwrap_or_default(),
        short_explanation,
        explanation,
      )
    })
    .collect::<Vec<_>>();
  formatter.push_values(&styled);
  formatter.print(Some(OutputFormat::Table)).unwrap()
}

pub(crate) fn print_environment_variable(env_var: &str, context: &Context) {
  let matching_env_vars: Vec<&(&str, &str, &str)> = ENVIRONMENT_VARIABLES
    .iter()
    .filter(|(defined_env_var, _, _)| defined_env_var.contains(&env_var.to_uppercase()))
    .collect::<Vec<_>>();
  if matching_env_vars.is_empty() {
    context.print_warning(format!("'{}' could not be matched to an environment variable recognized by the dsh tool", env_var));
  } else if matching_env_vars.len() == 1 {
    let (env_var, short_explanation, long_explanation) = matching_env_vars.first().unwrap();
    if let Ok(env_var_value) = std::env::var(env_var) {
      context.print_warning(format!("{}={}", env_var, env_var_value));
    } else {
      context.print_warning(env_var);
    }
    context.print(if long_explanation.is_empty() { short_explanation } else { long_explanation });
  } else {
    let mut formatter = ListFormatter::new(&[EnvVarLabel::Name, EnvVarLabel::Value, EnvVarLabel::LongExplanation], None, context);
    let styled_matching_env_vars: Vec<(String, String, &str, &str)> = matching_env_vars
      .into_iter()
      .map(|(env_var, short_explanation, long_explanation)| {
        (
          context.apply_label_style_for_stdout(env_var, Some(OutputFormat::Table)),
          env::var(env_var).unwrap_or_default(),
          *short_explanation,
          *long_explanation,
        )
      })
      .collect::<Vec<_>>();
    formatter.push_values(&styled_matching_env_vars);
    formatter.print(Some(OutputFormat::Table)).unwrap();
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum EnvVarLabel {
  LongExplanation,
  Name,
  ShortExplanation,
  Value,
}

impl Label for EnvVarLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::LongExplanation => "explanation",
      Self::Name => "environment variable",
      Self::ShortExplanation => "short explanation",
      Self::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Name)
  }
}

impl SubjectFormatter<EnvVarLabel> for (String, String, &str, &str) {
  fn value(&self, label: &EnvVarLabel, _: &str) -> String {
    match label {
      EnvVarLabel::LongExplanation => {
        if self.3.is_empty() {
          self.2.to_string()
        } else {
          self.3.to_string()
        }
      }
      EnvVarLabel::Name => self.0.to_string(),
      EnvVarLabel::ShortExplanation => self.2.to_string(),
      EnvVarLabel::Value => self.1.to_string(),
    }
  }
}

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
}

const ENVIRONMENT_VARIABLES: [(&str, &str, &str); 33] = [
  (
    ENV_VAR_PLATFORMS_FILE_NAME,
    "Overrides the default list of available platforms.",
    "Set this environment variable to override the default list of available platforms. \
     The value of the environment variable must be the name \
     of the alternative platforms file. It can either be an absolute file name, \
     or a relative file name from the working directory. \
     When this environment variable is set, the normal list of default platforms \
     will not be included. If you need these too, make sure that you also \
     include the default platforms in your platforms file. \
     See the bottom of this page for more information.",
  ),
  (
    ENV_VAR_CSV_QUOTE,
    "Specifies the quote character that will be used when printing csv data.",
    "This environment variable specifies the quote character that will be used \
     when printing csv data. If this variable is not provided, the value from the \
     settings file will be used. The default setting is not to use any quote characters. \
     Note that the dsh tool will fail when the generated output already \
     contains the quote character.",
  ),
  (
    ENV_VAR_CSV_SEPARATOR,
    "Specifies the separator string that will be used when printing csv data.",
    "This environment variable specifies the separator string that will be used \
     when printing csv data. If this variable is not provided, the value from the \
     settings file will be used. The default separator is ',' (comma). \
     Note that the dsh tool will fail when the generated output already \
     contains the csv separator string.",
  ),
  (
    ENV_VAR_DRY_RUN,
    "Inhibits api operations that could potentially make changes to the DSH platform, \
     like delete, create or change.",
    "If this environment variable is set (to any value) the dsh tool will not \
     call any api operations that could potentially make changes, like delete, create or \
     change. The input parameters will be validated and checked. \
     The same effect can be accomplished via the --dry-run \
     command line argument.",
  ),
  (
    ENV_VAR_ERROR_COLOR,
    "Specify the color to be used when printing error messages.",
    "This environment variable specifies the color to be used when printing error messages. \
    If this variable is not set, the settings file will be checked for the \
    'error-color' entry. Else the default color 'red' will be used. \
    The supported colors are: 'normal' (terminal default), 'black', 'blue', 'cyan', 'green', \
    'magenta', 'red', 'white' and 'yellow'.",
  ),
  (
    ENV_VAR_ERROR_STYLE,
    "Specifies the styling to be used when printing error messages.",
    "This environment variable specifies the styling to be used when printing error \
     messages. If this variable is not set, the settings file will be checked for the \
     'error-style' entry. Else the default value 'bold' will be used. \
     The supported styles are: 'normal' (no styling), 'bold', 'dim', 'italic', 'underline' \
     or 'reverse'.",
  ),
  (
    ENV_VAR_HOME_DIRECTORY,
    "Specifies the location of the directory where the dsh tool stores its settings and \
     targets information.",
    "Use this environment variable to change the location where dsh \
     stores its settings and targets information. \
     The default location is $HOME/.dsh_cli.",
  ),
  (
    ENV_VAR_LABEL_COLOR,
    "Specify the color to be used when printing table headers or labels.",
    "This environment variable specifies the color to be used when printing \
     table headers or labels. \
     If this variable is not set, the settings file will be checked for the \
     'label-color' entry. Else the default color 'blue' will be used. \
     See environment variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_LABEL_STYLE,
    "Specifies the styling to be used when printing table headers or labels.",
    "This environment variable specifies the styling to be used when printing \
     table headers or labels. \
     If this variable is not set, the settings file will be checked for the \
     'label-style' entry. Else the default value 'bold' will be used. \
     See environment variable 'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_LOG_LEVEL,
    "Specifies the log level of the dsh tool.",
    "Use this environment variable to set the log level of the dsh tool. \
     The available log levels are: off, error, warn, info, debug or trace. \
     If this argument is not provided, the settings file will be checked. \
     When the --log-level command line argument is provided this will override \
     this environment variable or the value in the settings file. \
     The default log level is 'error'.",
  ),
  (
    ENV_VAR_LOG_LEVEL_API,
    "Specifies the log level for the dsh_api library functions.",
    "Use this environment variable to set the log level for the functions \
     in the library crate dsh_api, that supports the dsh tool. \
     For the available log levels see the description of the \
     DSH_CLI_LOG_LEVEL environment variable. \
     If this argument is not provided, the settings file will be checked. \
     When the --log-level-api command line argument is provided this will \
     override this environment variable or the value in the settings file. \
     The default log level is 'error'.",
  ),
  (
    ENV_VAR_MATCHING_COLOR,
    "Specifies the color to be used when printing matching results for the find functions.",
    "This environment variable specifies the color to be used when printing matching \
     results for the find functions, e.q. when matching regular expressions. \
     If this variable is not set, the settings file will be checked for the \
     'matching-color' entry. Else the default color 'green' will be used. \
     See environment variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_MATCHING_STYLE,
    "Specifies the styling to be used when printing matching results for the find functions.",
    "This environment variable specifies the styling to be used when printing matching \
     results for the find functions, e.q. when matching regular expressions. \
     If this variable is not set, the settings file will be checked for the \
     'matching-style' entry. \
     Else the default value 'bold' will be used. \
     See environment variable 'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_NO_ESCAPE,
    "Disables color and styling escape sequences in the generated output.",
    "When this environment variable is set (to any value) \
     the output will not contain any color or other escape sequences. \
     This environment variable can be overridden via the \
     --no-color or --no-ansi command line argument.",
  ),
  (
    ENV_VAR_NO_HEADERS,
    "Disables headers in the generated output.",
    "When this environment variables is set (to any value) the output will not contain headers. \
     This environment variable can be overridden via the --no-headers command line argument.",
  ),
  (
    ENV_VAR_OUTPUT_FORMAT,
    "Specifies the format used when printing the output.",
    "This option specifies the format used when printing the output. \
     If this argument is not provided, the value from the settings file will be used. \
     Else, when stdout is a terminal the default 'table' will be used, \
     or if 'stdout' is not a terminal the value 'json' will be used. \
     The supported values are: 'csv', 'json', 'json-compact', 'plain', 'quiet', 'table', \
     'table-no-border', 'toml', 'toml-compact' or 'yaml'. \
     This environment variable can be overridden via the \
     --output-format command line argument.",
  ),
  (
    ENV_VAR_PASSWORD,
    "Specifies the secret api token/password for the target tenant.",
    "This environment variable specifies the secret api token/password for the target \
     tenant. Note that when the environment variable 'DSH_CLI_PASSWORD_FILE' \
     or the argument --password-file command line argument is provided, \
     this environment variable will never be used. \
     For better security, consider using one of these two options instead of \
     defining 'DSH_CLI_PASSWORD'.",
  ),
  (
    ENV_VAR_PASSWORD_FILE,
    "Specifies the location of a file containing the secret api token/password \
     for the target tenant.",
    "This environment variable specifies a file containing the secret api \
     token/password for the target tenant. \
     Note that when the --password-file command line argument is provided, \
     this environment variable will not be used.",
  ),
  (
    ENV_VAR_PLATFORM,
    "Specifies the target platform on which the target tenant environments live.",
    "Target platform on which the tenants environment lives. \
     The supported platforms are: 'np-aws-lz-dsh' / 'nplz', 'poc-aws-dsh' / 'poc', \
     'prod-aws-dsh' / 'prod', 'prod-aws-lz-dsh' / 'prodlz', 'prod-aws-lz-laas' / 'prodls' \
     or 'prod-azure-dsh' / 'prodaz'. \
     This environment variable can be overridden via the --platform command line argument.",
  ),
  (
    ENV_VAR_QUIET,
    "Enables quiet mode, which means that no output will be produced to the terminal.",
    "When this environment variable is set (to any value) the dsh tool \
     will run in quiet mode, meaning that no output will be produced to the terminal \
     (stdout and stderr). \
     This environment variable can be overridden via the --quit command line argument.",
  ),
  (
    ENV_VAR_SHOW_EXECUTION_TIME,
    "Enables printing the execution time of the executed api functions, in milliseconds.",
    "When this environment variable is set (to any value) the execution time of the \
     executed function will be shown, in milliseconds. \
     The execution time will also be shown when the verbosity level is set to \
     'high'. This environment variable can be overridden via the \
     '--show-execution-time' command line argument.",
  ),
  (
    ENV_VAR_STDERR_COLOR,
    "Specifies the color to be used when printing explanations and metadata.",
    "This environment variable specifies the color to be used when printing explanations \
     and metadata. \
     If this variable is not set, the settings file will be checked for the \
     'stderr-color' entry. Else the default color for the terminal will be used. \
     See environment variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_STDERR_STYLE,
    "Specifies the styling to be used when printing explanations and metadata.",
    "This environment variable specifies the styling to be used when printing explanations \
     and metadata. \
     If this variable is not set, the settings file will be checked for the \
     'stderr-style' entry. Else the default value 'dim' will be used. \
     See environment variable 'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_STDOUT_COLOR,
    "Specifies the color to be used when printing results.",
    "This environment variable specifies the color to be used when printing results. \
     If this variable is not set, the settings file will be checked for the \
     'stdout-color' entry. Else the default color for the terminal will be used. \
     See environment variable 'DSH_CLI_ERROR_COLOR' for the supported colors.",
  ),
  (
    ENV_VAR_STDOUT_STYLE,
    "Specifies the styling to be used when printing results.",
    "This environment variable specifies the styling to be used when printing results. \
     If this variable is not set, the settings file will be checked for the \
     'stdout-style' entry. Else the default value 'normal' \
     (no styling) will be used. \
     See environment variable 'DSH_CLI_ERROR_STYLE' for the supported styles.",
  ),
  (
    ENV_VAR_SUPPRESS_EXIT_STATUS,
    "Suppress the returned exit status of the tool (will always be 0).",
    "If this environment variable is set (to any value) the dsh tool will \
     always return exit status 0, even when an error has occurred. \
     This can be useful in scripting environments. \
     The same effect can be accomplished via the '--suppress-exit-code' \
     command line argument or the 'suppress-exit-status' setting.",
  ),
  (
    ENV_VAR_TERMINAL_WIDTH,
    "Specifies the maximum terminal width.",
    "When this environment variable is set it will define the maximum terminal width. \
     This environment variable can be overridden via the \
     --terminal-width command line argument.",
  ),
  (
    ENV_VAR_TENANT,
    "Specifies the target tenant.",
    "Tenant id for the target tenant. The target tenant is the tenant whose resources \
     will be managed via the api. \
     This environment variable can be overridden via the --tenant command line argument.",
  ),
  (
    ENV_VAR_VERBOSITY,
    "Specifies the verbosity level of the dsh tool.",
    "If this option is provided, it will set the verbosity level. \
     The default verbosity setting is 'low'. \
     The supported verbosity levels are: off, low, medium or high. \
     This environment variable can be overridden via the \
     '--verbosity' command line argument. \
     Also, when the environment variable 'DSH_CLI_QUIET' is set \
     or the command line argument '--quiet' is provided, nothing will be printed.",
  ),
  (
    ENV_VAR_WARNING_COLOR,
    "Specifies the color to be used when printing warnings.",
    "This environment variable specifies the color to be used when printing warnings. \
     If this variable is not set, the settings file will be checked for the \
     'warning-color' entry. Else the default color 'blue' will be \
     used. See environment variable 'DSH_CLI_ERROR_COLOR' for the supported \
     colors.",
  ),
  (
    ENV_VAR_WARNING_STYLE,
    "Specifies the styling to be used when printing warnings.",
    "This environment variable specifies the styling to be used when printing warnings. \
     If this variable is not set, the settings file will be checked for the \
     'warning-style' entry. Else the default value 'bold' will be \
     used. See environment variable 'DSH_CLI_ERROR_STYLE' for the supported \
     styles.",
  ),
  (
    ENV_VAR_NO_COLOR,
    "Disables color and styling escape sequences in the generated output.",
    "When this environment variable is set (to any value) \
     the output will not contain any color or other escape sequences. \
     This environment variable can be overridden via the \
     '--no-color' or '--no-ansi' command line argument.",
  ),
  (
    ENV_VAR_RUST_LOG,
    "Specifies the log level of the rust env_logger crate.",
    "Since the dsh tool depends on the env_logger crate for its logging, \
     it also recognizes log configuration via the RUST_LOG environment variable. \
     Although the use of this variable is not recommended, \
     there might be situations when this can be useful. \
     See the crate's github repository for more information.",
  ),
];
