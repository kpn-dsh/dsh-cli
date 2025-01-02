use crate::arguments::{
  Verbosity, DRY_RUN_ARGUMENT, MATCHING_STYLE_ARGUMENT, NO_ESCAPE_ARGUMENT, OUTPUT_FORMAT_ARGUMENT, QUIET_ARGUMENT, SHOW_EXECUTION_TIME_ARGUMENT, TERMINAL_WIDTH_ARGUMENT,
  VERBOSITY_ARGUMENT,
};
use crate::formatters::OutputFormat;
use crate::settings::{read_settings, Settings};
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::query_processor::Part;
use dsh_api::query_processor::Part::{Matching, NonMatching};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::io::{stderr, stdin, stdout, IsTerminal, Write};
use std::time::Instant;
use terminal_size::{terminal_size, Height, Width};

static ENV_VAR_CSV_QUOTE: &str = "DSH_CLI_CSV_QUOTE";
static ENV_VAR_CSV_SEPARATOR: &str = "DSH_CLI_CSV_SEPARATOR";
static ENV_VAR_DRY_RUN: &str = "DSH_CLI_DRY_RUN";
static ENV_VAR_MATCHING_STYLE: &str = "DSH_CLI_MATCHING_STYLE";
static ENV_VAR_NO_COLOR: &str = "NO_COLOR";
static ENV_VAR_NO_ESCAPE: &str = "DSH_CLI_NO_ESCAPE";
static ENV_VAR_OUTPUT_FORMAT: &str = "DSH_CLI_OUTPUT_FORMAT";
static ENV_VAR_QUIET: &str = "DSH_CLI_QUIET";
static ENV_VAR_SHOW_EXECUTION_TIME: &str = "DSH_CLI_SHOW_EXECUTION_TIME";
static ENV_VAR_TERMINAL_WIDTH: &str = "DSH_CLI_TERMINAL_WIDTH";
static ENV_VAR_VERBOSITY: &str = "DSH_CLI_VERBOSITY";

#[derive(Debug)]
pub(crate) struct Context<'a> {
  pub(crate) csv_quote: Option<char>,
  pub(crate) csv_separator: String,
  pub(crate) dry_run: bool,
  pub(crate) dsh_api_client: Option<DshApiClient<'a>>,
  pub(crate) force: bool,
  pub(crate) matching_style: Option<MatchingStyle>,
  pub(crate) no_escape: bool,
  pub(crate) output_format: OutputFormat,
  pub(crate) quiet: bool,
  pub(crate) terminal_width: Option<usize>,
  pub(crate) show_execution_time: bool,
  pub(crate) show_labels: bool,
  pub(crate) _stderr_escape: bool,
  pub(crate) _stdin_terminal: bool,
  pub(crate) _stdout_escape: bool,
  pub(crate) verbosity: Verbosity,
}

impl Context<'_> {
  pub(crate) fn create<'a>(matches: &'a ArgMatches, dsh_api_client: Option<DshApiClient<'a>>) -> Result<Context<'a>, String> {
    let settings: Option<Settings> = read_settings(None)?;
    let csv_quote = Self::csv_quote(settings.as_ref())?;
    let csv_separator = Self::csv_separator(settings.as_ref())?;
    if let Some(quote) = csv_quote {
      if csv_separator.contains(quote) {
        return Err("csv separator string cannot contain quote character".to_string());
      }
    }
    let dry_run = Self::dry_run(matches, settings.as_ref());
    let no_escape = Self::no_escape(matches, settings.as_ref());
    let (matching_style, stderr_escape, stdout_escape) =
      if no_escape { (None, false, false) } else { (Self::matching_style(matches, settings.as_ref())?, stderr().is_terminal(), stdout().is_terminal()) };
    let quiet = Self::quiet(matches, settings.as_ref());
    let force = Self::force(matches, settings.as_ref());
    let (output_format, show_execution_time, verbosity) = if quiet {
      (OutputFormat::Quiet, false, Verbosity::Off)
    } else {
      (
        Self::output_format(matches, settings.as_ref())?,
        Self::show_execution_time(matches, settings.as_ref()),
        Self::verbosity(matches, settings.as_ref())?,
      )
    };
    let show_labels = true;
    let terminal_width = Self::terminal_width(matches, settings.as_ref())?;
    if dry_run && verbosity >= Verbosity::Medium {
      eprintln!("dry-run mode enabled");
    }
    Ok(Context {
      csv_quote,
      csv_separator,
      dry_run,
      dsh_api_client,
      force,
      matching_style,
      no_escape,
      output_format,
      quiet,
      show_execution_time,
      show_labels,
      terminal_width,
      _stderr_escape: stderr_escape,
      _stdin_terminal: stdin().is_terminal(),
      _stdout_escape: stdout_escape,
      verbosity,
    })
  }

  pub(crate) fn confirmed(&self, prompt: impl AsRef<str>) -> Result<bool, String> {
    if self.force {
      Ok(true)
    } else {
      print!("{}", prompt.as_ref());
      let _ = stdout().lock().flush();
      let mut line = String::new();
      stdin().read_line(&mut line).expect("could not read line");
      line = line.trim().to_string();
      Ok(line == *"yes")
    }
  }

  /// Gets csv quote context value
  ///
  /// 1. Try environment variable `DSH_CLI_CSV_QUOTE`
  /// 1. Try settings file
  /// 1. Default to `None`
  fn csv_quote(settings: Option<&Settings>) -> Result<Option<char>, String> {
    match std::env::var(ENV_VAR_CSV_QUOTE) {
      Ok(csv_quote_env_var) => {
        if csv_quote_env_var.len() == 1 {
          Ok(csv_quote_env_var.chars().next())
        } else {
          Err("csv quote must one character".to_string())
        }
      }
      Err(_) => Ok(settings.and_then(|settings| settings.csv_quote)),
    }
  }

  /// Gets csv separator context value
  ///
  /// 1. Try environment variable `DSH_CLI_CSV_SEPARATOR`
  /// 1. Try settings file
  /// 1. Default to `","` (comma)
  fn csv_separator(settings: Option<&Settings>) -> Result<String, String> {
    match std::env::var(ENV_VAR_CSV_SEPARATOR) {
      Ok(csv_separator_env_var) => {
        if !csv_separator_env_var.is_empty() {
          Ok(csv_separator_env_var)
        } else {
          Err("seperator cannot be empty".to_string())
        }
      }
      Err(_) => match settings.and_then(|settings| settings.csv_separator.clone()) {
        Some(csv_separator_setting) => {
          if !csv_separator_setting.is_empty() {
            Ok(csv_separator_setting)
          } else {
            Err("seperator cannot be empty".to_string())
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
  fn dry_run(matches: &ArgMatches, settings: Option<&Settings>) -> bool {
    matches.get_flag(DRY_RUN_ARGUMENT) || std::env::var(ENV_VAR_DRY_RUN).is_ok() || settings.is_some_and(|settings| settings.dry_run.unwrap_or(false))
  }

  /// Gets force context value
  ///
  /// 1. Try flag `--force`
  /// 1. Default to `false`
  fn force(matches: &ArgMatches, _settings: Option<&Settings>) -> bool {
    matches.get_flag(DRY_RUN_ARGUMENT)
  }

  /// Gets matching_style context value
  ///
  /// 1. Try flag `--matching-style`
  /// 1. Try environment variable `DSH_CLI_MATCHING_STYLE`
  /// 1. Try settings file
  /// 1. Default to `None`
  fn matching_style(matches: &ArgMatches, settings: Option<&Settings>) -> Result<Option<MatchingStyle>, String> {
    match matches.get_one::<MatchingStyle>(MATCHING_STYLE_ARGUMENT) {
      Some(matching_style_argument) => Ok(Some(matching_style_argument.to_owned())),
      None => match std::env::var(ENV_VAR_MATCHING_STYLE) {
        Ok(matching_style_env_var) => MatchingStyle::try_from(matching_style_env_var.as_str()).map(Some),
        Err(_) => Ok(settings.and_then(|settings| settings.matching_style.clone())),
      },
    }
  }

  /// Gets no_escape context value
  ///
  /// 1. Try flag `--no-color` or `--no-ansi`
  /// 1. Try if environment variable `NO_COLOR` exists
  /// 1. Try if environment variable `DSH_CLI_NO_ESCAPE` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn no_escape(matches: &ArgMatches, settings: Option<&Settings>) -> bool {
    matches.get_flag(NO_ESCAPE_ARGUMENT)
      || std::env::var(ENV_VAR_NO_COLOR).is_ok()
      || std::env::var(ENV_VAR_NO_ESCAPE).is_ok()
      || settings.is_some_and(|settings| settings.no_escape.unwrap_or(false))
  }

  /// Gets output_format context value
  ///
  /// 1. Try flag `--output-format`
  /// 1. Try environment variable `DSH_CLI_OUTPUT_FORMAT`
  /// 1. Try settings file
  /// 1. If stdout is a terminal default to `OutputFormat::Table`,
  ///    else default to `OutputFormat::Json`
  fn output_format(matches: &ArgMatches, settings: Option<&Settings>) -> Result<OutputFormat, String> {
    match matches.get_one::<OutputFormat>(OUTPUT_FORMAT_ARGUMENT) {
      Some(output_format_argument) => Ok(output_format_argument.to_owned()),
      None => match std::env::var(ENV_VAR_OUTPUT_FORMAT) {
        Ok(output_format_env_var) => OutputFormat::try_from(output_format_env_var.as_str()).map_err(|error| format!("{} in environment variable {}", error, ENV_VAR_OUTPUT_FORMAT)),
        Err(_) => match settings.and_then(|settings| settings.output_format.clone()) {
          Some(output_format_from_settings) => Ok(output_format_from_settings),
          None => Ok(if stdout().is_terminal() { OutputFormat::Table } else { OutputFormat::Json }),
        },
      },
    }
  }

  /// Gets quiet context value
  ///
  /// 1. Try flag `--quiet`
  /// 1. Try if environment variable `DSH_CLI_QUIET` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn quiet(matches: &ArgMatches, settings: Option<&Settings>) -> bool {
    matches.get_flag(QUIET_ARGUMENT) || std::env::var(ENV_VAR_QUIET).is_ok() || settings.is_some_and(|settings| settings.quiet.unwrap_or(false))
  }

  /// Gets show_execution_time context value
  ///
  /// 1. Try flag `--show-execution-time`
  /// 1. Try if environment variable `DSH_CLI_SHOW_EXECUTION_TIME` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn show_execution_time(matches: &ArgMatches, settings: Option<&Settings>) -> bool {
    matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT)
      || std::env::var(ENV_VAR_SHOW_EXECUTION_TIME).is_ok()
      || settings.is_some_and(|settings| settings.show_execution_time.unwrap_or(false))
  }

  /// Gets terminal width context value
  ///
  /// 1. Try flag `--terminal-width`
  /// 1. Try if environment variable `DSH_CLI_TERMINAL_WIDTH` exists
  /// 1. Try settings file
  /// 1. If stdout is a terminal use actual terminal width, else default to `None`
  fn terminal_width(matches: &ArgMatches, settings: Option<&Settings>) -> Result<Option<usize>, String> {
    match matches.get_one::<usize>(TERMINAL_WIDTH_ARGUMENT) {
      Some(terminal_width_argument) => Ok(Some(terminal_width_argument.to_owned())),
      None => match std::env::var(ENV_VAR_TERMINAL_WIDTH) {
        Ok(terminal_width_env_var) => match terminal_width_env_var.parse::<usize>() {
          Ok(terminal_width) => {
            if terminal_width < 40 {
              Err(format!(
                "terminal width must be greater than 40 in environment variable {} must be greater or equal than 40",
                ENV_VAR_TERMINAL_WIDTH
              ))
            } else {
              Ok(Some(terminal_width))
            }
          }
          Err(_) => Err(format!(
            "non-numerical value '{}' in environment variable {}",
            terminal_width_env_var, ENV_VAR_TERMINAL_WIDTH
          )),
        },
        Err(_) => match settings.and_then(|settings| settings.terminal_width) {
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
  fn verbosity(matches: &ArgMatches, settings: Option<&Settings>) -> Result<Verbosity, String> {
    match matches.get_one::<Verbosity>(VERBOSITY_ARGUMENT) {
      Some(verbosity_argument) => Ok(verbosity_argument.to_owned()),
      None => match std::env::var(ENV_VAR_VERBOSITY) {
        Ok(verbosity_env_var) => Verbosity::try_from(verbosity_env_var.as_str()).map_err(|error| format!("{} in environment variable {}", error, ENV_VAR_VERBOSITY)),
        Err(_) => match settings.and_then(|settings| settings.verbosity.clone()) {
          Some(verbosity_from_settings) => Ok(verbosity_from_settings),
          None => Ok(Verbosity::Low),
        },
      },
    }
  }

  /// # Prints the output to stdout
  ///
  /// This method is used to print the output of the tool to the standard output device.
  /// If `quiet` is `true`, nothing will be printed.
  /// This standard output device can either be a tty, a pipe or an output file,
  /// depending on how the tool was run from a shell or script.
  pub(crate) fn print<T: AsRef<str>>(&self, output: T) {
    if !self.quiet {
      println!("{}", output.as_ref());
    }
  }

  /// # Prints a prompt to stderr
  ///
  /// This method is used to print a prompt to the standard error device.
  /// The prompt is used when input from the user is expected.
  /// If `quiet` is `true`, nothing will be printed.
  /// The prompt is only printed when stderr is a tty,
  /// since it would make no sense for a pipe or output file.
  pub(crate) fn print_prompt<T: AsRef<str>>(&self, prompt: T) {
    if !self.quiet && stderr().is_terminal() {
      eprintln!("{}", prompt.as_ref());
    }
  }

  /// # Prints the outcome to stderr
  ///
  /// This method is used to print the outcome of the tool to the standard error device.
  /// The outcome is not the output of the tool, but indicates whether a function was
  /// successful or not.
  /// This method is typically used when the function has side effects,
  /// like creating or deleting a resource.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_outcome<T: AsRef<str>>(&self, outcome: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium | Verbosity::High => eprintln!("{}", outcome.as_ref()),
      }
    }
  }

  /// # Prints a warning to stderr
  ///
  /// This method is used to print a warning to the standard error device.
  /// The warning is not the output of the tool, but indicates a special situation.
  /// This method is typically used when the function behaves differently
  /// then the user might expect, like when the `--dry-run` option was provided.
  /// If `--quiet` is provided or `--verbosity` is `off`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_warning<T: AsRef<str>>(&self, warning: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off => (),
        Verbosity::Low | Verbosity::Medium | Verbosity::High => eprintln!("{}", warning.as_ref()),
      }
    }
  }

  /// # Prints an error to stderr
  ///
  /// This method is used to print an error message to the standard error device.
  /// If `quiet` is `true`, nothing will be printed.
  /// The standard error device is almost always a tty, but can in special cases also be
  /// a pipe or an output file.
  pub(crate) fn print_error<T: AsRef<str>>(&self, error: T) {
    if !self.quiet {
      eprintln!("{}", error.as_ref());
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
  pub(crate) fn print_explanation<T: AsRef<str>>(&self, explanation: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium => eprintln!("{}", explanation.as_ref()),
        Verbosity::High => {
          if let Some(ref client) = self.dsh_api_client {
            eprintln!("target {}", client.tenant());
          }
          eprintln!("{}", explanation.as_ref())
        }
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
    if !self.quiet && (self.show_execution_time || self.verbosity == Verbosity::High) {
      eprintln!("execution took {} milliseconds", Instant::now().duration_since(start_instant).as_millis());
    }
  }

  /// Converts `Part` slice to string for stderr
  ///
  /// This method converts a `Part` slice to a `String`, formatted to be printed to stderr.
  /// Ansi escape characters will be used only when `self.matching_parts_style` has a value
  /// and `self.stderr_escape` is `true`. Else the result will be a plain `String`.
  pub(crate) fn _parts_to_string_stderr(&self, parts: &[Part]) -> String {
    self.parts_to_string(parts, self._stderr_escape)
  }

  /// Converts `Part` slice to string for stdout
  ///
  /// This method converts a `Part` slice to a `String`, formatted to be printed to stdout.
  /// Ansi escape characters will be used only when `self.matching_parts_style` has a value
  /// and `self.stdout_escape` is `true`. Else the result will be a plain `String`.
  pub(crate) fn parts_to_string_stdout(&self, parts: &[Part]) -> String {
    self.parts_to_string(parts, self._stdout_escape)
  }

  /// Converts `Part` slice to string
  ///
  /// This method converts a `Part` slice to a `String`, formatted to be printed to
  /// stderr or stdout. Ansi escape characters will be used only when
  /// `self.matching_parts_style` has a value, the `escape` parameter is `true`
  /// and `no_escape` is `false`.
  /// Else the result will be a plain `String`.
  fn parts_to_string(&self, parts: &[Part], escape: bool) -> String {
    match (&self.matching_style, escape, self.no_escape) {
      (Some(style), true, false) => parts
        .iter()
        .map(|part| match part {
          Matching(p) => wrap_style(style.clone(), p.as_str()),
          NonMatching(p) => p.to_string(),
        })
        .collect::<Vec<_>>()
        .join(""),
      _ => parts
        .iter()
        .map(|part| match part {
          Matching(p) => p.to_string(),
          NonMatching(p) => p.to_string(),
        })
        .collect::<Vec<_>>()
        .join(""),
    }
  }

  pub(crate) fn _show_settings(&self) -> bool {
    match self.verbosity {
      Verbosity::Off | Verbosity::Low => false,
      Verbosity::Medium | Verbosity::High => true,
    }
  }

  /// Converts string slice to csv value
  ///
  /// This method converts a value to a `String`, formatted to be printed as csv.
  /// It will perform some checks to see if conversion is allowed and add quotes if necessary.
  pub(crate) fn csv_value(&self, value: &str) -> Result<String, String> {
    if value.contains(self.csv_separator.as_str()) {
      Err("csv value contains separator character".to_string())
    } else if value.contains("\n") {
      Err("csv value contains new line".to_string())
    } else if let Some(csv_quote) = self.csv_quote {
      if value.contains(csv_quote) {
        Err("csv value contains quote character".to_string())
      } else {
        Ok(format!("{}{}{}", csv_quote, value, csv_quote))
      }
    } else {
      Ok(value.to_string())
    }
  }
}

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum MatchingStyle {
  /// Matches will be displayed in normal font
  #[serde(rename = "normal")]
  Normal = 0,
  /// Matches will be displayed bold
  #[serde(rename = "bold")]
  Bold = 1,
  /// Matches will be displayed dimmed
  #[serde(rename = "dim")]
  Dim = 2,
  /// Matches will be displayed in italics
  #[serde(rename = "italic")]
  Italic = 3,
  /// Matches will be displayed underlined
  #[serde(rename = "underlined")]
  Underlined = 4,
  /// Mathces will be displayed reversed
  #[serde(rename = "reverse")]
  Reverse = 7,
}

impl Display for MatchingStyle {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MatchingStyle::Normal => write!(f, "normal"),
      MatchingStyle::Bold => write!(f, "bold"),
      MatchingStyle::Dim => write!(f, "dim"),
      MatchingStyle::Italic => write!(f, "italic"),
      MatchingStyle::Underlined => write!(f, "underlined"),
      MatchingStyle::Reverse => write!(f, "reverse"),
    }
  }
}

impl TryFrom<&str> for MatchingStyle {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "normal" => Ok(Self::Normal),
      "bold" => Ok(Self::Bold),
      "dim" => Ok(Self::Dim),
      "italic" => Ok(Self::Italic),
      "underlined" => Ok(Self::Underlined),
      "reverse" => Ok(Self::Reverse),
      _ => Err(format!("invalid matching style '{}'", value)),
    }
  }
}

pub fn wrap_style(style: MatchingStyle, string: &str) -> String {
  if style == MatchingStyle::Normal {
    string.to_string()
  } else {
    format!("\x1b[{}m{}\x1b[0m", style as usize, string)
  }
}
