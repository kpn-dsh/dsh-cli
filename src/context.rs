use crate::authentication::AuthenticationMethod;
use crate::environment_variables::{
  environment_variable, is_environment_variable_specified, ENV_VAR_DSH_CLI_AUTHENTICATION, ENV_VAR_DSH_CLI_BROWSER, ENV_VAR_DSH_CLI_CSV_QUOTE, ENV_VAR_DSH_CLI_CSV_SEPARATOR,
  ENV_VAR_DSH_CLI_DRY_RUN, ENV_VAR_DSH_CLI_ERROR_COLOR, ENV_VAR_DSH_CLI_ERROR_STYLE, ENV_VAR_DSH_CLI_LABEL_COLOR, ENV_VAR_DSH_CLI_LABEL_STYLE, ENV_VAR_DSH_CLI_MATCHING_COLOR,
  ENV_VAR_DSH_CLI_MATCHING_STYLE, ENV_VAR_DSH_CLI_NO_ESCAPE, ENV_VAR_DSH_CLI_NO_HEADERS, ENV_VAR_DSH_CLI_OUTPUT_FORMAT, ENV_VAR_DSH_CLI_QUIET, ENV_VAR_DSH_CLI_SHOW_EXECUTION_TIME,
  ENV_VAR_DSH_CLI_STDERR_COLOR, ENV_VAR_DSH_CLI_STDERR_STYLE, ENV_VAR_DSH_CLI_STDOUT_COLOR, ENV_VAR_DSH_CLI_STDOUT_STYLE, ENV_VAR_DSH_CLI_SUPPRESS_EXIT_STATUS,
  ENV_VAR_DSH_CLI_TERMINAL_WIDTH, ENV_VAR_DSH_CLI_VERBOSITY, ENV_VAR_DSH_CLI_WARNING_COLOR, ENV_VAR_DSH_CLI_WARNING_STYLE, ENV_VAR_NO_COLOR,
};
use crate::error::DshCliError;
use crate::formatters::OutputFormat;
use crate::global_arguments::{
  AUTHENTICATION_ARGUMENT, BROWSER_ARGUMENT, DRY_RUN_ARGUMENT, FORCE_ARGUMENT, NO_ESCAPE_ARGUMENT, NO_HEADERS_ARGUMENT, OUTPUT_FORMAT_ARGUMENT, QUIET_ARGUMENT,
  SHOW_EXECUTION_TIME_ARGUMENT, SUPPRESS_EXIT_STATUS_ARGUMENT, TERMINAL_WIDTH_ARGUMENT, VERBOSITY_ARGUMENT,
};
use crate::settings::Settings;
use crate::style::{apply_default_warning_style, style_from, DshColor, DshStyle};
use crate::verbosity::Verbosity;
use crate::{error, error_append, DshCliResult};
use clap::builder::styling::Style;
use clap::ArgMatches;
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::query_processor::Part::{Matching, NonMatching};
use dsh_api::query_processor::{Part, QueryProcessor, RegexQueryProcessor};
use dsh_api::types::{AllocationStatus, Notification};
use getch_rs::{Getch, Key};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::io::{stderr, stdin, stdout, IsTerminal, Write};
use std::process;
use std::time::Instant;
use terminal_size::{terminal_size, Height, Width};
use OutputFormat::Csv;

#[derive(clap::ValueEnum, Eq, Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum BrowserMethod {
  /// User will be instructed to open the browser
  #[serde(rename = "instruct")]
  Instruct,
  /// Tool will try to open the browser automatically
  #[serde(rename = "open")]
  Open,
}

impl TryFrom<&str> for BrowserMethod {
  type Error = DshCliError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "instruct" => Ok(Self::Instruct),
      "open" => Ok(Self::Open),
      _ => Err(error!("invalid browser method '{}'", value)),
    }
  }
}

impl Display for BrowserMethod {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Instruct => write!(f, "instruct"),
      Self::Open => write!(f, "open"),
    }
  }
}

impl Default for BrowserMethod {
  fn default() -> Self {
    Self::Instruct
  }
}

#[derive(Debug, Default)]
pub(crate) struct Context {
  authentication_method: AuthenticationMethod,
  browser_method: BrowserMethod,
  csv_quote: Option<char>,
  csv_separator: String,
  dry_run: bool,
  error_style: Style,
  force: bool,
  label_style: Style,
  matching_style: Style,
  output_format_specification: Option<OutputFormat>,
  quiet: bool,
  settings: Settings,
  show_execution_time: bool,
  show_headers: bool,
  stderr_is_terminal: bool,
  stderr_no_escape: bool,
  stderr_style: Style,
  stdin_is_terminal: bool,
  stdout_is_terminal: bool,
  stdout_no_escape: bool,
  stdout_style: Style,
  suppress_exit_status: bool,
  terminal_width: Option<usize>,
  verbosity: Verbosity,
  warning_style: Style,
}

impl Context {
  pub(crate) fn create(matches: &ArgMatches, settings: Settings) -> DshCliResult<Context> {
    let stderr_is_terminal = stderr().is_terminal();
    let stdin_is_terminal = stdin().is_terminal();
    let stdout_is_terminal = stdout().is_terminal();
    let csv_quote = Self::get_csv_quote(matches, &settings)?;
    let csv_separator = Self::get_csv_separator(matches, &settings)?;
    if let Some(quote) = csv_quote {
      if csv_separator.contains(quote) {
        return Err(error!("csv separator string cannot contain quote character"));
      }
    }
    let authentication_method = Self::get_authentication_method(matches, &settings, stdin_is_terminal)?;
    let browser_method = Self::get_browser_method(matches, &settings, stdin_is_terminal)?;
    let dry_run = Self::get_dry_run(matches, &settings);
    let (stderr_no_escape, stdout_no_escape) = if Self::get_no_escape(matches, &settings) { (true, true) } else { (!stderr_is_terminal, !stdout_is_terminal) };

    let error_style = style_from(
      &Self::get_style(ENV_VAR_DSH_CLI_ERROR_STYLE, matches, &settings.error_style, DshStyle::Bold)?,
      &Self::get_color(ENV_VAR_DSH_CLI_ERROR_COLOR, matches, &settings.error_color, DshColor::Red)?,
    );
    let label_style = style_from(
      &Self::get_style(ENV_VAR_DSH_CLI_LABEL_STYLE, matches, &settings.label_style, DshStyle::Bold)?,
      &Self::get_color(ENV_VAR_DSH_CLI_LABEL_COLOR, matches, &settings.label_color, DshColor::Blue)?,
    );
    let matching_style = style_from(
      &Self::get_style(ENV_VAR_DSH_CLI_MATCHING_STYLE, matches, &settings.matching_style, DshStyle::Bold)?,
      &Self::get_color(ENV_VAR_DSH_CLI_MATCHING_COLOR, matches, &settings.matching_color, DshColor::Green)?,
    );
    let stderr_style = style_from(
      &Self::get_style(ENV_VAR_DSH_CLI_STDERR_STYLE, matches, &settings.stderr_style, DshStyle::Dim)?,
      &Self::get_color(ENV_VAR_DSH_CLI_STDERR_COLOR, matches, &settings.stderr_color, DshColor::Normal)?,
    );
    let stdout_style = style_from(
      &Self::get_style(ENV_VAR_DSH_CLI_STDOUT_STYLE, matches, &settings.stdout_style, DshStyle::Normal)?,
      &Self::get_color(ENV_VAR_DSH_CLI_STDOUT_COLOR, matches, &settings.stdout_color, DshColor::Normal)?,
    );
    let warning_style = style_from(
      &Self::get_style(ENV_VAR_DSH_CLI_WARNING_STYLE, matches, &settings.warning_style, DshStyle::Bold)?,
      &Self::get_color(ENV_VAR_DSH_CLI_WARNING_COLOR, matches, &settings.warning_color, DshColor::Blue)?,
    );
    let quiet = Self::get_quiet(matches, &settings);
    let force = Self::get_force(matches, &settings);
    let suppress_exit_status = Self::get_suppress_exit_status(matches, &settings);
    let output_format_specification = Self::get_output_format_specification(matches, &settings)?;
    let show_execution_time = Self::get_show_execution_time(matches, &settings);
    let verbosity = Self::get_verbosity(matches, &settings)?;
    let show_headers = !Self::get_no_headers(matches, &settings);
    let terminal_width = Self::get_terminal_width(matches, &settings)?;
    if dry_run && verbosity >= Verbosity::Medium {
      eprintln!("dry-run mode enabled");
    }
    Ok(Context {
      authentication_method,
      browser_method,
      csv_quote,
      csv_separator,
      dry_run,
      error_style,
      force,
      label_style,
      matching_style,
      output_format_specification,
      quiet,
      settings,
      show_execution_time,
      show_headers,
      stderr_is_terminal,
      stderr_no_escape,
      stderr_style,
      stdin_is_terminal,
      stdout_is_terminal,
      stdout_no_escape,
      stdout_style,
      suppress_exit_status,
      terminal_width,
      verbosity,
      warning_style,
    })
  }

  pub(crate) fn authentication_method(&self) -> &AuthenticationMethod {
    &self.authentication_method
  }

  pub(crate) fn browser_method(&self) -> &BrowserMethod {
    &self.browser_method
  }

  pub(crate) fn csv_quote(&self) -> &Option<char> {
    &self.csv_quote
  }

  pub(crate) fn csv_separator(&self) -> &String {
    &self.csv_separator
  }

  pub(crate) fn dry_run(&self) -> bool {
    self.dry_run
  }

  pub(crate) fn settings(&self) -> &Settings {
    &self.settings
  }

  pub(crate) fn show_headers(&self) -> bool {
    self.show_headers
  }

  pub(crate) fn stdin_is_terminal(&self) -> bool {
    self.stdin_is_terminal
  }

  pub(crate) fn suppress_exit_status(&self) -> bool {
    self.suppress_exit_status
  }

  pub(crate) fn terminal_width(&self) -> Option<usize> {
    self.terminal_width
  }

  /// Gets authentication method
  ///
  /// 1. Try command line argument --authentication
  /// 1. Try environment variable `DSH_CLI_AUTHENTICATION`
  /// 1. Try value `authentication-method` in settings file
  /// 1. If stdin is a terminal, default to `AuthenticationMethod::SingleSignOn`
  /// 1. Else default to `AuthenticationMethod::Robot`
  fn get_authentication_method(matches: &ArgMatches, settings: &Settings, stdin_is_terminal: bool) -> DshCliResult<AuthenticationMethod> {
    match matches.get_one::<AuthenticationMethod>(AUTHENTICATION_ARGUMENT) {
      Some(authentication_argument) => Ok(authentication_argument.to_owned()),
      None => match environment_variable(ENV_VAR_DSH_CLI_AUTHENTICATION, Some(matches))? {
        Some(authentication_env_var) => AuthenticationMethod::try_from(authentication_env_var.as_str()),
        None => match &settings.authentication {
          Some(authentication_setting) => Ok(authentication_setting.to_owned()),
          None => {
            if stdin_is_terminal {
              Ok(AuthenticationMethod::SingleSignOn)
            } else {
              Ok(AuthenticationMethod::Robot)
            }
          }
        },
      },
    }
  }

  /// Gets browser method
  ///
  /// 1. Try command line argument --browser
  /// 1. Try environment variable `DSH_CLI_BROWSER`
  /// 1. Try settings file
  /// 1. If stdin is a terminal, default to `BrowserMethod::Open`
  /// 1. Else default to `BrowserMethod::Instruct`
  fn get_browser_method(matches: &ArgMatches, settings: &Settings, stdin_is_terminal: bool) -> DshCliResult<BrowserMethod> {
    match matches.get_one::<BrowserMethod>(BROWSER_ARGUMENT) {
      Some(browser_argument) => Ok(browser_argument.to_owned()),
      None => match environment_variable(ENV_VAR_DSH_CLI_BROWSER, Some(matches))? {
        Some(browser_env_var) => BrowserMethod::try_from(browser_env_var.as_str()),
        None => match &settings.browser {
          Some(browser_setting) => Ok(browser_setting.to_owned()),
          None => {
            if stdin_is_terminal {
              Ok(BrowserMethod::Open)
            } else {
              Ok(BrowserMethod::Instruct)
            }
          }
        },
      },
    }
  }

  /// Ask for confirmation
  ///
  /// 1. If `force` is enabled, confirmation is always `true`.
  /// 1. Else, if run from a terminal the user will be prompted for confirmation.
  /// 1. When not run from a terminal confirmation is always false.
  pub(crate) fn confirmed(&self, prompt: impl Display) -> DshCliResult<bool> {
    if self.force {
      self.eprintln(format!("{}, confirmed by --force option", prompt));
      Ok(true)
    } else if self.stdin_is_terminal {
      self.eprint(format!("{} [y/N]", prompt));
      let _ = stdout().lock().flush();
      match Getch::new().getch() {
        Ok(key) => match key {
          Key::Char('y') | Key::Char('Y') => {
            eprintln!();
            Ok(true)
          }
          Key::Ctrl('c') => {
            eprintln!("{}", apply_default_warning_style("\ninterrupted"));
            process::exit(0);
          }
          _ => {
            eprintln!();
            Ok(false)
          }
        },
        Err(error) => Err(error!("\nerror getting key event ({})", error)),
      }
    } else {
      Ok(false)
    }
  }

  /// Gets csv quote context value
  ///
  /// 1. Try environment variable `DSH_CLI_CSV_QUOTE`
  /// 1. Try settings file
  /// 1. Default to `None`
  fn get_csv_quote(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<char>> {
    match environment_variable(ENV_VAR_DSH_CLI_CSV_QUOTE, Some(matches))? {
      Some(csv_quote_env_var) => {
        if csv_quote_env_var.len() == 1 {
          Ok(csv_quote_env_var.chars().next())
        } else {
          Err(error!("csv quote must one character"))
        }
      }
      None => Ok(settings.csv_quote),
    }
  }

  /// Gets csv separator context value
  ///
  /// 1. Try environment variable `DSH_CLI_CSV_SEPARATOR`
  /// 1. Try settings file
  /// 1. Default to `","` (comma)
  fn get_csv_separator(matches: &ArgMatches, settings: &Settings) -> DshCliResult<String> {
    match environment_variable(ENV_VAR_DSH_CLI_CSV_SEPARATOR, Some(matches))? {
      Some(csv_separator_env_var) => {
        if !csv_separator_env_var.is_empty() {
          Ok(csv_separator_env_var)
        } else {
          Err(error!("seperator cannot be empty"))
        }
      }
      None => match settings.csv_separator.clone() {
        Some(csv_separator_setting) => {
          if !csv_separator_setting.is_empty() {
            Ok(csv_separator_setting)
          } else {
            Err(error!("seperator cannot be empty"))
          }
        }
        None => Ok(",".to_string()),
      },
    }
  }

  /// Gets dry_run context value
  ///
  /// 1. Try flag `--dry-run`
  /// 1. Try if environment variable `DSH_CLI_DRY_RUN` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_dry_run(matches: &ArgMatches, settings: &Settings) -> bool {
    if matches.get_flag(DRY_RUN_ARGUMENT) {
      debug!("dry run mode enabled (argument)");
      true
    } else if is_environment_variable_specified(ENV_VAR_DSH_CLI_DRY_RUN, matches) {
      debug!("dry run mode enabled (environment variable '{}')", ENV_VAR_DSH_CLI_DRY_RUN);
      true
    } else if let Some(dry_run) = settings.dry_run {
      if dry_run {
        debug!("dry run mode enabled (settings)");
      }
      dry_run
    } else {
      false
    }
  }

  /// Gets force context value
  ///
  /// 1. Try flag `--force`
  /// 1. Try if environment variable `DSH_CLI_FORCE` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_force(matches: &ArgMatches, settings: &Settings) -> bool {
    if matches.get_flag(FORCE_ARGUMENT) {
      debug!("force mode enabled (argument)");
      true
    } else if is_environment_variable_specified(ENV_VAR_DSH_CLI_DRY_RUN, matches) {
      debug!("force mode enabled (environment variable '{}')", ENV_VAR_DSH_CLI_DRY_RUN);
      true
    } else if let Some(dry_run) = settings.dry_run {
      if dry_run {
        debug!("force mode enabled (settings)");
      }
      dry_run
    } else {
      false
    }
  }

  /// Gets suppress_status context value
  ///
  /// 1. Try flag `--suppress-exit-status`
  /// 1. Try if environment variable `DSH_CLI_SUPPRESS_STATUS` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_suppress_exit_status(matches: &ArgMatches, settings: &Settings) -> bool {
    if matches.get_flag(SUPPRESS_EXIT_STATUS_ARGUMENT) {
      debug!("suppress exit status enabled (argument)");
      true
    } else if is_environment_variable_specified(ENV_VAR_DSH_CLI_SUPPRESS_EXIT_STATUS, matches) {
      debug!("suppress exit status enabled (environment variable '{}')", ENV_VAR_DSH_CLI_SUPPRESS_EXIT_STATUS);
      true
    } else if let Some(suppress_exit_status) = settings.suppress_exit_status {
      if suppress_exit_status {
        debug!("suppress exit status enabled (settings)");
      }
      suppress_exit_status
    } else {
      false
    }
  }

  /// Gets dsh color context value
  ///
  /// 1. Try environment variable `env_var`
  /// 1. Try settings file value
  /// 1. Default to `default_color`
  pub(crate) fn get_color(env_var: &str, matches: &ArgMatches, settings_color: &Option<DshColor>, default_color: DshColor) -> DshCliResult<DshColor> {
    match environment_variable(env_var, Some(matches))? {
      Some(color_from_env_var) => DshColor::try_from(color_from_env_var.as_str()),
      None => match settings_color {
        Some(ref color_from_settings) => Ok(color_from_settings.clone()),
        None => Ok(default_color),
      },
    }
  }

  /// Gets dsh style context value
  ///
  /// 1. Try environment variable `env_var`
  /// 1. Try settings file value
  /// 1. Default to `default_style`
  pub(crate) fn get_style(env_var: &str, matches: &ArgMatches, settings_style: &Option<DshStyle>, default_style: DshStyle) -> DshCliResult<DshStyle> {
    match environment_variable(env_var, Some(matches))? {
      Some(style_from_env_var) => DshStyle::try_from(style_from_env_var.as_str()),
      None => match settings_style {
        Some(ref style_from_settings) => Ok(style_from_settings.clone()),
        None => Ok(default_style),
      },
    }
  }

  /// Gets no escape context value
  ///
  /// 1. Try flag `--no-color` or `--no-ansi`
  /// 1. Try if environment variable `NO_COLOR` exists
  /// 1. Try if environment variable `DSH_CLI_NO_ESCAPE` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_no_escape(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(NO_ESCAPE_ARGUMENT)
      || is_environment_variable_specified(ENV_VAR_NO_COLOR, matches)
      || is_environment_variable_specified(ENV_VAR_DSH_CLI_NO_ESCAPE, matches)
      || settings.no_escape.unwrap_or(false)
  }

  /// Gets no headers context value
  ///
  /// 1. Try flag `--no-headers`
  /// 1. Try if environment variable `DSH_CLI_NO_HEADERS` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_no_headers(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(NO_HEADERS_ARGUMENT) || is_environment_variable_specified(ENV_VAR_DSH_CLI_NO_HEADERS, matches) || settings.no_headers.unwrap_or(false)
  }

  /// Gets output format specification
  ///
  /// 1. Try flag `--output-format`
  /// 1. Try environment variable `DSH_CLI_OUTPUT_FORMAT`
  /// 1. Try settings file
  /// 1. Else default to `None`
  fn get_output_format_specification(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<OutputFormat>> {
    match matches.get_one::<OutputFormat>(OUTPUT_FORMAT_ARGUMENT) {
      Some(output_format_argument) => Ok(Some(output_format_argument.to_owned())),
      None => match environment_variable(ENV_VAR_DSH_CLI_OUTPUT_FORMAT, Some(matches))? {
        Some(output_format_env_var) => OutputFormat::try_from(output_format_env_var.as_str())
          .map_err(error_append!("{} in environment variable: ", ENV_VAR_DSH_CLI_OUTPUT_FORMAT))
          .map(Some),
        None => match settings.output_format.clone() {
          Some(output_format_from_settings) => Ok(Some(output_format_from_settings)),
          None => Ok(None),
        },
      },
    }
  }

  /// Gets output_format context value
  ///
  /// 1. Try specification (flag `--output-format`, environment variable
  ///    `DSH_CLI_OUTPUT_FORMAT` or settings file),
  /// 1. Try default_output_format parameter
  /// 1. If stdout is a terminal default to `OutputFormat::Table`,
  ///    else default to `OutputFormat::Json`
  pub(crate) fn output_format(&self, default_output_format: Option<OutputFormat>) -> OutputFormat {
    match self.output_format_specification {
      Some(ref output_format_from_specification) => output_format_from_specification.clone(),
      None => default_output_format.unwrap_or(if self.stdout_is_terminal { OutputFormat::Table } else { OutputFormat::Json }),
    }
  }

  /// Gets quiet context value
  ///
  /// 1. Try flag `--quiet`
  /// 1. Try if environment variable `DSH_CLI_QUIET` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_quiet(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(QUIET_ARGUMENT) || is_environment_variable_specified(ENV_VAR_DSH_CLI_QUIET, matches) || settings.quiet.unwrap_or(false)
  }

  /// Gets show_execution_time context value
  ///
  /// 1. Try flag `--show-execution-time`
  /// 1. Try if environment variable `DSH_CLI_SHOW_EXECUTION_TIME` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn get_show_execution_time(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT)
      || is_environment_variable_specified(ENV_VAR_DSH_CLI_SHOW_EXECUTION_TIME, matches)
      || settings.show_execution_time.unwrap_or(false)
  }

  /// Gets terminal width context value
  ///
  /// 1. Try flag `--terminal-width`
  /// 1. Try if environment variable `DSH_CLI_TERMINAL_WIDTH` exists
  /// 1. Try settings file
  /// 1. If stdout is a terminal use actual terminal width, else default to `None`
  fn get_terminal_width(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<usize>> {
    match matches.get_one::<usize>(TERMINAL_WIDTH_ARGUMENT) {
      Some(terminal_width_argument) => Ok(Some(terminal_width_argument.to_owned())),
      None => match environment_variable(ENV_VAR_DSH_CLI_TERMINAL_WIDTH, Some(matches))? {
        Some(terminal_width_env_var) => match terminal_width_env_var.parse::<usize>() {
          Ok(terminal_width) => {
            if terminal_width < 40 {
              Err(error!(
                "terminal width in environment variable {} must be greater than or equal to 40",
                ENV_VAR_DSH_CLI_TERMINAL_WIDTH
              ))
            } else {
              Ok(Some(terminal_width))
            }
          }
          Err(_) => Err(error!(
            "non-numerical value '{}' in environment variable {}",
            terminal_width_env_var, ENV_VAR_DSH_CLI_TERMINAL_WIDTH
          )),
        },
        None => match settings.terminal_width {
          Some(terminal_width_from_settings) => Ok(Some(terminal_width_from_settings)),
          None => {
            if stdout().is_terminal() {
              match terminal_size() {
                Some((Width(width), Height(_))) => Ok(Some(width as usize)),
                None => Ok(None),
              }
            } else {
              Ok(None)
            }
          }
        },
      },
    }
  }

  /// Gets verbosity context value
  ///
  /// 1. Try flag `--verbosity`
  /// 1. Try environment variable `DSH_CLI_VERBOSITY`
  /// 1. Try settings file
  /// 1. Default to `Verbosity::Low`
  fn get_verbosity(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Verbosity> {
    match matches.get_one::<Verbosity>(VERBOSITY_ARGUMENT) {
      Some(verbosity_argument) => Ok(verbosity_argument.to_owned()),
      None => match environment_variable(ENV_VAR_DSH_CLI_VERBOSITY, Some(matches))? {
        Some(verbosity_env_var) => Verbosity::try_from(verbosity_env_var.as_str()).map_err(error_append!("error in environment variable {}: ", ENV_VAR_DSH_CLI_VERBOSITY)),
        None => match settings.verbosity.clone() {
          Some(verbosity_from_settings) => Ok(verbosity_from_settings),
          None => Ok(Verbosity::Low),
        },
      },
    }
  }

  /// Open the provided url in the system browser
  pub(crate) fn open_url(&self, url: impl AsRef<OsStr> + Display, opening_target: impl Display) {
    if self.dry_run() {
      self.print_warning(format!("dry-run mode, opening {} canceled", opening_target));
      self.print_warning(format!("{}", url));
    } else {
      match self.browser_method() {
        BrowserMethod::Instruct => {
          self.print_explanation(format!("opening {}", opening_target));
          self.print_explanation("open url in your browser:");
          self.print(format!("{}", url));
        }
        BrowserMethod::Open => match open::that(&url) {
          Ok(()) => {
            self.print_explanation(format!("opening {}", opening_target));
          }
          Err(error) => {
            self.print_error(format!("could not open {} in your browser", opening_target));
            debug!("{}", error);
            self.print_explanation("open url in your browser:");
            self.print(format!("{}", url));
          }
        },
      }
    }
  }

  /// # Returns current time `Instant`
  pub(crate) fn now(&self) -> Instant {
    Instant::now()
  }

  /// # Prints plain text output to stdout
  ///
  /// This method is used to print the output of the `dsh` tool to the standard output device.
  /// If `quiet` is `true`, nothing will be printed.
  /// This standard output device can either be a tty, a pipe or an output file,
  /// depending on how the `dsh` tool was run from a shell or script.
  pub(crate) fn print<T: Display>(&self, output: T) {
    if !self.quiet {
      self.println(output)
    }
  }

  /// # Prints serializable output to stdout
  ///
  /// This method is used to print a serialized version of the output of the `dsh` tool
  /// to the standard output device.
  /// If `quiet` is `true`, nothing will be printed.
  /// This standard output device can either be a tty, a pipe or an output file,
  /// depending on how the `dsh` tool was run from a shell or script.
  pub(crate) fn print_serializable<T: Serialize>(&self, output: T, default_output_format: Option<OutputFormat>) {
    if !self.quiet {
      match self.output_format(default_output_format) {
        Csv => self.print_warning("csv output is not supported here, use --output-format json|toml|yaml"),
        OutputFormat::Json => match serde_json::to_string_pretty(&output) {
          Ok(json) => self.println(json),
          Err(_) => self.print_error("serializing to json failed"),
        },
        OutputFormat::JsonCompact => match serde_json::to_string(&output) {
          Ok(json) => self.println(json),
          Err(_) => self.print_error("serializing to json failed"),
        },
        OutputFormat::Plain => self.print_warning("plain output is not supported here, use --output-format json|toml|yaml"),
        OutputFormat::Quiet => (),
        OutputFormat::Table | OutputFormat::TableNoBorder => self.print_warning("table output is not supported here, use --output-format json|toml|yaml"),
        OutputFormat::Toml => match toml::ser::to_string_pretty(&output) {
          Ok(toml) => self.println(toml),
          Err(_) => self.print_error("serializing to toml failed"),
        },
        OutputFormat::TomlCompact => match toml::ser::to_string(&output) {
          Ok(toml) => self.println(toml),
          Err(_) => self.print_error("serializing to toml failed"),
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&output) {
          Ok(yaml) => self.println(yaml),
          Err(_) => self.print_error("serializing to yaml failed"),
        },
      }
    }
  }

  /// # Prints the next progress bar character to stderr
  ///
  /// If `quiet` is `true`, nothing will be printed.
  /// The prompt is only printed when stderr is a terminal.
  pub(crate) fn print_progress_step(&self) {
    if !self.quiet && self.stderr_is_terminal {
      self.eprint(".");
    }
  }

  /// # Prints a prompt to stderr
  ///
  /// This method is used to print a prompt to the standard error device.
  /// The prompt is used when input from the user is expected.
  /// If `quiet` is `true`, nothing will be printed.
  /// The prompt is only printed when stderr is a terminal,
  /// since it would make no sense for a pipe or output file.
  pub(crate) fn print_prompt<T: Display>(&self, prompt: T) {
    if !self.quiet && self.stderr_is_terminal {
      self.eprint(prompt);
    }
  }

  /// # Prints the outcome to stderr
  ///
  /// This method is used to print the outcome of the `dsh` tool to the standard error device.
  /// The outcome is not the output of the tool, but indicates whether a function was
  /// successful or not.
  /// This method is typically used when the function has side effects,
  /// like creating or deleting a resource.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_outcome<T: Display>(&self, outcome: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium | Verbosity::High => self.eprintln(outcome),
      }
    }
  }

  /// # Prints a warning to stderr
  ///
  /// This method is used to print a warning to the standard error device.
  /// The warning is not the output of the `dsh` tool, but indicates a special situation.
  /// This method is typically used when the function behaves differently
  /// then the user might expect, like when the `--dry-run` option was provided.
  /// If `--quiet` is provided or `--verbosity` is `off`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_warning<T: Display>(&self, warning: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off => (),
        Verbosity::Low | Verbosity::Medium | Verbosity::High => self.eprintln_warning(warning),
      }
    }
  }

  /// # Prints an error to stderr
  ///
  /// This method is used to print an error message to the standard error device.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_error<T: Display>(&self, error: T) {
    if !self.quiet {
      self.eprintln_error(error);
    }
  }

  /// # Prints an explanation to stderr
  ///
  /// This method is used to print an explanation about the function that is
  /// about to be executed to stderr. When the verbosity level is `High` and a client is available,
  /// also the target is printed to stderr.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_explanation<T: Display>(&self, explanation: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium | Verbosity::High => self.eprintln(explanation),
      }
    }
  }

  /// # Prints an explanation to stderr
  ///
  /// This method is used to print an explanation about the function that is
  /// about to be executed to stderr. When the verbosity level is `High` and a client is available,
  /// also the target is printed to stderr.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_allocation_status<T: Display>(&self, allocation_status: &AllocationStatus, target: T) {
    if !self.quiet {
      for notification in &allocation_status.notifications {
        // if notification.remove {
        self.eprintln_warning(format_notification_message(notification));
        // } else {
        //   self.eprintln(format_notification_message(notification));
        // }
      }
      if let Some(derived_from) = &allocation_status.derived_from {
        self.eprint(format!("derived from: {}", derived_from));
      }
      match self.verbosity {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium => {
          if allocation_status.provisioned {
            self.eprintln(format!("{} is provisioned", target));
          }
        }
        Verbosity::High => {
          if allocation_status.provisioned {
            self.eprintln(format!("{} is provisioned", target));
          } else {
            self.eprintln(format!("{} is not provisioned", target));
          }
        }
      }
    }
  }

  /// # Prints the target platform and tenant to stderr
  ///
  /// This method is used to print the target platform and tenant to stderr,
  /// when the verbosity level is `High`.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_target(&self, dsh_api_tenant: &DshApiTenant) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off | Verbosity::Low | Verbosity::Medium => (),
        Verbosity::High => self.eprintln(format!("target {}", dsh_api_tenant)),
      }
    }
  }

  /// # Prints the execution time to stderr
  ///
  /// This method computes the time elapsed since `start_instant` (in milliseconds)
  /// and prints the result to stderr. The time is only printed when the verbosity level
  /// is high enough and/or the `show-execution-time` flag has been set.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_execution_time(&self, start_instant: Instant) {
    if !self.quiet && self.show_execution_time {
      self.eprintln(format!("execution took {} milliseconds", Instant::now().duration_since(start_instant).as_millis()));
    }
  }

  pub(crate) fn read_multi_line(&self, prompt: impl Display) -> DshCliResult<String> {
    if self.stdin_is_terminal {
      self.print_prompt(prompt);
    }
    let mut multi_line = String::new();
    let stdin = stdin();
    loop {
      match stdin.read_line(&mut multi_line) {
        Ok(0) => break,
        Ok(_) => continue,
        Err(_) => return Err(error!("error reading line")),
      }
    }
    Ok(multi_line)
  }

  pub(crate) fn read_single_line(&self, prompt: impl Display) -> DshCliResult<String> {
    if self.stdin_is_terminal {
      self.print_prompt(format!("{}: ", prompt));
    }
    let _ = stdout().lock().flush();
    let mut line = String::new();
    stdin().read_line(&mut line).expect("could not read line");
    Ok(line.trim().to_string())
  }

  pub(crate) fn read_single_line_with_default(&self, prompt: impl Display, default: impl Display) -> DshCliResult<String> {
    if self.stdin_is_terminal {
      self.print_prompt(format!("{} [{}]: ", prompt, default));
    }
    let _ = stdout().lock().flush();
    let mut line = String::new();
    stdin().read_line(&mut line).expect("could not read line");
    let trimmed = line.trim();
    if trimmed.is_empty() {
      Ok(default.to_string())
    } else {
      Ok(trimmed.to_string())
    }
  }

  pub(crate) fn read_single_line_password(&self, prompt: impl Display) -> DshCliResult<String> {
    if self.stdin_is_terminal {
      match prompt_password(prompt) {
        Ok(line) => Ok(line.trim().to_string()),
        Err(_) => Err(error!("empty input")),
      }
    } else {
      self.read_single_line(prompt)
    }
  }

  /// Converts `Part` slice to string for stdout
  ///
  /// This method converts a `Part` slice to a `String`, formatted to be printed to stdout.
  /// If the output format is `Table` or `TableNoBorder` and `stdout_no_escape` is not set,
  /// the `matching_style` will be applied to the matching parts and the string will be
  /// post-fixed with an escape sequence to reset the `stdout_style`.
  pub(crate) fn parts_to_string_for_stdout(&self, parts: &[Part], default_output_format: Option<OutputFormat>) -> String {
    match self.output_format(default_output_format) {
      OutputFormat::Table | OutputFormat::TableNoBorder => {
        if self.stdout_no_escape {
          Self::parts_to_string(parts)
        } else {
          parts
            .iter()
            .map(|part| match part {
              Matching(matching_part) => format!("{}{}{:#}{}", self.matching_style, matching_part, self.matching_style, self.stdout_style),
              NonMatching(non_matching_part) => non_matching_part.to_string(),
            })
            .collect_vec()
            .join("")
        }
      }
      _ => Self::parts_to_string(parts),
    }
  }

  fn parts_to_string(parts: &[Part]) -> String {
    parts.iter().map(|part| part.to_string()).collect_vec().join("")
  }

  /// Applies styling for labels for stdout
  ///
  /// If the output format is `Table` or `TableNoBorder` and `stdout_no_escape` is not set,
  /// the `label_style` will be applied to the provided string, and it will be
  /// post-fixed with an escape sequence to set the `stdout_style`.
  pub(crate) fn apply_label_style_for_stdout<T: Display>(&self, text: T, default_output_format: Option<OutputFormat>) -> String {
    match self.output_format(default_output_format) {
      OutputFormat::Table | OutputFormat::TableNoBorder => {
        if self.stdout_no_escape {
          text.to_string()
        } else {
          format!("{}{}{:#}{}", self.label_style, text, self.label_style, self.stdout_style)
        }
      }
      _ => text.to_string(),
    }
  }

  /// Converts string slice to csv value
  ///
  /// This method converts a value to a `String`, formatted to be printed as csv.
  /// It will perform some checks to see if conversion is allowed and add quotes if necessary.
  pub(crate) fn csv_value(&self, value: &str) -> DshCliResult<String> {
    if value.contains(self.csv_separator.as_str()) {
      Err(error!("csv value contains separator character"))
    } else if value.contains("\n") {
      Err(error!("csv value contains new line"))
    } else if let Some(csv_quote) = self.csv_quote {
      if value.contains(csv_quote) {
        Err(error!("csv value contains quote character"))
      } else {
        Ok(format!("{}{}{}", csv_quote, value, csv_quote))
      }
    } else {
      Ok(value.to_string())
    }
  }

  fn _apply_stderr_style<T: Display>(&self, text: T) -> String {
    if self.stderr_no_escape {
      text.to_string()
    } else {
      format!("{}{}{:#}", self.error_style, text, self.error_style)
    }
  }

  /// Print a warning text to stderr
  ///
  /// If `stderr_no_escape` is not set, the `warning_style` will be applied to the provided string,
  /// and it will be post-fixed with an escape sequence to reset to the `stderr_style`.
  fn eprintln_warning<T: Display>(&self, text: T) {
    if self.stderr_no_escape {
      eprintln!("{}", text)
    } else {
      eprintln!("{}{}{:#}", self.warning_style, text, self.warning_style)
    }
  }

  /// Print an error text to stderr
  ///
  /// If `stderr_no_escape` is not set, the `error_style` will be applied to the provided string,
  /// and it will be post-fixed with an escape sequence to reset to the `stderr_style`.
  fn eprintln_error<T: Display>(&self, text: T) {
    if self.stderr_no_escape {
      eprintln!("{}", text)
    } else {
      eprintln!("{}{}{:#}", self.error_style, text, self.error_style)
    }
  }

  /// Print a text to stderr without newline
  ///
  /// If `stderr_no_escape` is not set, the `stderr_style` will be applied to the provided string,
  /// and it will be post-fixed with an escape sequence to reset the `stderr_style`.
  fn eprint<T: Display>(&self, text: T) {
    if self.stderr_no_escape {
      eprint!("{}", text)
    } else {
      eprint!("{}{}{:#}", self.stderr_style, text, self.stderr_style)
    }
  }

  /// Print a text to stderr
  ///
  /// If `stderr_no_escape` is not set, the `error_style` will be applied to the provided string,
  /// and it will be post-fixed with an escape sequence to set the `stderr_style`.
  fn eprintln<T: Display>(&self, text: T) {
    if self.stderr_no_escape {
      eprintln!("{}", text)
    } else {
      eprintln!("{}{}{:#}", self.stderr_style, text, self.stderr_style)
    }
  }

  /// Print a text to stdout
  ///
  /// If `stdout_no_escape` is not set, the `stdout_style` will be applied to the provided string,
  /// and it will be post-fixed with an escape sequence to reset the `stdout_style`.
  fn println<T: Display>(&self, text: T) {
    if self.stdout_no_escape {
      println!("{}", text)
    } else {
      println!("{}{}{:#}", self.stdout_style, text, self.stdout_style)
    }
  }
}

lazy_static! {
  static ref NotificationQueryProcessor: RegexQueryProcessor = RegexQueryProcessor::create(r"\$\{[a-zA-Z0-9:_-]+\}").unwrap();
}

fn format_notification_message(notification: &Notification) -> String {
  match NotificationQueryProcessor.matching_parts(&notification.message) {
    Some(parts) => parts
      .iter()
      .map(|part| match part {
        Matching(matching) => {
          let stripped = &matching[2..matching.len() - 1];
          if let Some(key) = stripped.strip_prefix("urn:") {
            match notification.args.get(key) {
              Some(value) => value.to_string(),
              None => key.to_uppercase(),
            }
          } else {
            match notification.args.get(stripped) {
              Some(value) => value.to_string(),
              None => stripped.to_uppercase(),
            }
          }
        }
        NonMatching(non_matching) => non_matching.to_string(),
      })
      .join(""),
    None => notification.message.clone(),
  }
}

#[test]
fn test_format_notification_message() {
  let mut args = std::collections::HashMap::<String, String>::new();
  args.insert("arg1".to_string(), "ARG1".to_string());
  args.insert("arg2".to_string(), "ARG2".to_string());
  let message = "abc${arg1}def${urn:arg2}ghi";
  let notification = Notification::new(args, message, true);
  assert_eq!(format_notification_message(&notification), "abcARG1defARG2ghi");
}
