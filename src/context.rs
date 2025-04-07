use crate::formatters::OutputFormat;
use crate::global_arguments::{
  DRY_RUN_ARGUMENT, FORCE_ARGUMENT, NO_ESCAPE_ARGUMENT, NO_HEADERS_ARGUMENT, OUTPUT_FORMAT_ARGUMENT, QUIET_ARGUMENT, SHOW_EXECUTION_TIME_ARGUMENT, SUPPRESS_EXIT_STATUS_ARGUMENT,
  TERMINAL_WIDTH_ARGUMENT, VERBOSITY_ARGUMENT,
};
use crate::settings::Settings;
use crate::style::{style_from, wrap_style, DshColor, DshStyle};
use crate::verbosity::Verbosity;
use crate::{
  ENV_VAR_CSV_QUOTE, ENV_VAR_CSV_SEPARATOR, ENV_VAR_DRY_RUN, ENV_VAR_ERROR_COLOR, ENV_VAR_ERROR_STYLE, ENV_VAR_MATCHING_COLOR, ENV_VAR_MATCHING_STYLE, ENV_VAR_NO_COLOR,
  ENV_VAR_NO_ESCAPE, ENV_VAR_NO_HEADERS, ENV_VAR_OUTPUT_FORMAT, ENV_VAR_QUIET, ENV_VAR_SHOW_EXECUTION_TIME, ENV_VAR_STDERR_COLOR, ENV_VAR_STDERR_STYLE, ENV_VAR_STDOUT_COLOR,
  ENV_VAR_STDOUT_STYLE, ENV_VAR_SUPPRESS_EXIT_STATUS, ENV_VAR_TERMINAL_WIDTH, ENV_VAR_VERBOSITY, ENV_VAR_WARNING_COLOR, ENV_VAR_WARNING_STYLE,
};
use clap::builder::styling::Style;
use clap::ArgMatches;
use dsh_api::query_processor::Part;
use dsh_api::query_processor::Part::{Matching, NonMatching};
use log::debug;
use rpassword::prompt_password;
use serde::Serialize;
use std::io::{stderr, stdin, stdout, IsTerminal, Write};
use std::time::Instant;
use terminal_size::{terminal_size, Height, Width};

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) csv_quote: Option<char>,
  pub(crate) csv_separator: String,
  pub(crate) dry_run: bool,
  pub(crate) error_color: DshColor,
  pub(crate) error_style: DshStyle,
  pub(crate) force: bool,
  pub(crate) matching_color: DshColor,
  pub(crate) matching_style: DshStyle,
  pub(crate) no_escape: bool,
  output_format_specification: Option<OutputFormat>,
  pub(crate) quiet: bool,
  pub(crate) settings: Settings,
  pub(crate) show_execution_time: bool,
  pub(crate) show_headers: bool,
  pub(crate) stderr_color: DshColor,
  pub(crate) _stderr_escape: bool,
  pub(crate) stderr_style: DshStyle,
  pub(crate) stdin_is_terminal: bool,
  pub(crate) stdout_color: DshColor,
  pub(crate) _stdout_escape: bool,
  pub(crate) stdout_style: DshStyle,
  pub(crate) terminal_width: Option<usize>,
  pub(crate) verbosity: Verbosity,
  pub(crate) warning_color: DshColor,
  pub(crate) warning_style: DshStyle,
  pub(crate) suppress_exit_status: bool,
}

impl Context {
  pub(crate) fn create(matches: &ArgMatches, settings: Settings) -> Result<Context, String> {
    let stdin_is_terminal = stdin().is_terminal();
    let csv_quote = Self::csv_quote(&settings)?;
    let csv_separator = Self::csv_separator(&settings)?;
    if let Some(quote) = csv_quote {
      if csv_separator.contains(quote) {
        return Err("csv separator string cannot contain quote character".to_string());
      }
    }
    let dry_run = Self::dry_run(matches, &settings);
    let no_escape = Self::no_escape(matches, &settings);
    let (stderr_escape, stdout_escape) = if no_escape { (false, false) } else { (stderr().is_terminal(), stdout().is_terminal()) };

    let error_color = Self::dsh_color(no_escape, ENV_VAR_ERROR_COLOR, &settings.error_color, DshColor::Red)?;
    let error_style = Self::dsh_style(no_escape, ENV_VAR_ERROR_STYLE, &settings.error_style, DshStyle::Bold)?;
    let matching_color = Self::dsh_color(no_escape, ENV_VAR_MATCHING_COLOR, &settings.matching_color, DshColor::Green)?;
    let matching_style = Self::dsh_style(no_escape, ENV_VAR_MATCHING_STYLE, &settings.matching_style, DshStyle::Bold)?;
    let stderr_color = Self::dsh_color(no_escape, ENV_VAR_STDERR_COLOR, &settings.stderr_color, DshColor::Normal)?;
    let stderr_style = Self::dsh_style(no_escape, ENV_VAR_STDERR_STYLE, &settings.stderr_style, DshStyle::Dim)?;
    let stdout_color = Self::dsh_color(no_escape, ENV_VAR_STDOUT_COLOR, &settings.stdout_color, DshColor::Normal)?;
    let stdout_style = Self::dsh_style(no_escape, ENV_VAR_STDOUT_STYLE, &settings.stdout_style, DshStyle::Normal)?;
    let warning_color = Self::dsh_color(no_escape, ENV_VAR_WARNING_COLOR, &settings.warning_color, DshColor::Blue)?;
    let warning_style = Self::dsh_style(no_escape, ENV_VAR_WARNING_STYLE, &settings.warning_style, DshStyle::Bold)?;

    let quiet = Self::quiet(matches, &settings);
    let force = Self::force(matches, &settings);
    let suppress_exit_status = Self::suppress_exit_status(matches, &settings);
    let (output_format_specification, show_execution_time, verbosity) = if quiet {
      (Some(OutputFormat::Quiet), false, Verbosity::Off)
    } else {
      (
        Self::output_format_specification(matches, &settings)?,
        Self::show_execution_time(matches, &settings),
        Self::verbosity(matches, &settings)?,
      )
    };
    let show_headers = !Self::no_headers(matches, &settings);
    let terminal_width = Self::terminal_width(matches, &settings)?;
    if dry_run && verbosity >= Verbosity::Medium {
      eprintln!("dry-run mode enabled");
    }
    Ok(Context {
      csv_quote,
      csv_separator,
      dry_run,
      error_color,
      error_style,
      force,
      matching_color,
      matching_style,
      no_escape,
      output_format_specification,
      quiet,
      settings,
      show_execution_time,
      show_headers,
      stderr_color,
      _stderr_escape: stderr_escape,
      stderr_style,
      stdin_is_terminal,
      stdout_color,
      _stdout_escape: stdout_escape,
      stdout_style,
      suppress_exit_status,
      terminal_width,
      verbosity,
      warning_color,
      warning_style,
    })
  }

  /// Ask for confirmation
  ///
  /// 1. If `force` is enabled, confirmation is always `true`.
  /// 1. Else, if run from a terminal the user will be prompted for confirmation.
  /// 1. When not run from a terminal confirmation is always false.
  pub(crate) fn confirmed(&self, prompt: impl AsRef<str>) -> Result<bool, String> {
    if self.force {
      Ok(true)
    } else if stdin().is_terminal() {
      eprint!("{} [y/N]", prompt.as_ref());
      let _ = stdout().lock().flush();
      let mut line = String::new();
      stdin().read_line(&mut line).expect("could not read line");
      Ok(line.trim().to_lowercase() == "y")
    } else {
      Ok(false)
    }
  }

  /// Gets csv quote context value
  ///
  /// 1. Try environment variable `DSH_CLI_CSV_QUOTE`
  /// 1. Try settings file
  /// 1. Default to `None`
  fn csv_quote(settings: &Settings) -> Result<Option<char>, String> {
    match std::env::var(ENV_VAR_CSV_QUOTE) {
      Ok(csv_quote_env_var) => {
        if csv_quote_env_var.len() == 1 {
          Ok(csv_quote_env_var.chars().next())
        } else {
          Err("csv quote must one character".to_string())
        }
      }
      Err(_) => Ok(settings.csv_quote),
    }
  }

  /// Gets csv separator context value
  ///
  /// 1. Try environment variable `DSH_CLI_CSV_SEPARATOR`
  /// 1. Try settings file
  /// 1. Default to `","` (comma)
  fn csv_separator(settings: &Settings) -> Result<String, String> {
    match std::env::var(ENV_VAR_CSV_SEPARATOR) {
      Ok(csv_separator_env_var) => {
        if !csv_separator_env_var.is_empty() {
          Ok(csv_separator_env_var)
        } else {
          Err("seperator cannot be empty".to_string())
        }
      }
      Err(_) => match settings.csv_separator.clone() {
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
  fn dry_run(matches: &ArgMatches, settings: &Settings) -> bool {
    if matches.get_flag(DRY_RUN_ARGUMENT) {
      debug!("dry run mode enabled (argument)");
      true
    } else if std::env::var(ENV_VAR_DRY_RUN).is_ok() {
      debug!("dry run mode enabled (environment variable '{}')", ENV_VAR_DRY_RUN);
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
  fn force(matches: &ArgMatches, settings: &Settings) -> bool {
    if matches.get_flag(FORCE_ARGUMENT) {
      debug!("force mode enabled (argument)");
      true
    } else if std::env::var(ENV_VAR_DRY_RUN).is_ok() {
      debug!("force mode enabled (environment variable '{}')", ENV_VAR_DRY_RUN);
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
  fn suppress_exit_status(matches: &ArgMatches, settings: &Settings) -> bool {
    if matches.get_flag(SUPPRESS_EXIT_STATUS_ARGUMENT) {
      debug!("suppress exit status enabled (argument)");
      true
    } else if std::env::var(ENV_VAR_SUPPRESS_EXIT_STATUS).is_ok() {
      debug!("suppress exit status enabled (environment variable '{}')", ENV_VAR_SUPPRESS_EXIT_STATUS);
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
  fn dsh_color(no_escape: bool, env_var: &str, settings_color: &Option<DshColor>, default_color: DshColor) -> Result<DshColor, String> {
    if no_escape {
      Ok(DshColor::Normal)
    } else {
      match std::env::var(env_var) {
        Ok(color_from_env_var) => DshColor::try_from(color_from_env_var.as_str()),
        Err(_) => match settings_color {
          Some(ref color_from_settings) => Ok(color_from_settings.clone()),
          None => Ok(default_color),
        },
      }
    }
  }

  /// Gets dsh style context value
  ///
  /// 1. Try environment variable `env_var`
  /// 1. Try settings file value
  /// 1. Default to `default_style`
  fn dsh_style(no_escape: bool, env_var: &str, settings_style: &Option<DshStyle>, default_style: DshStyle) -> Result<DshStyle, String> {
    if no_escape {
      Ok(DshStyle::Normal)
    } else {
      match std::env::var(env_var) {
        Ok(style_from_env_var) => DshStyle::try_from(style_from_env_var.as_str()),
        Err(_) => match settings_style {
          Some(ref style_from_settings) => Ok(style_from_settings.clone()),
          None => Ok(default_style),
        },
      }
    }
  }

  /// Gets no escape context value
  ///
  /// 1. Try flag `--no-color` or `--no-ansi`
  /// 1. Try if environment variable `NO_COLOR` exists
  /// 1. Try if environment variable `DSH_CLI_NO_ESCAPE` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn no_escape(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(NO_ESCAPE_ARGUMENT) || std::env::var(ENV_VAR_NO_COLOR).is_ok() || std::env::var(ENV_VAR_NO_ESCAPE).is_ok() || settings.no_escape.unwrap_or(false)
  }

  /// Gets no headers context value
  ///
  /// 1. Try flag `--no-headers`
  /// 1. Try if environment variable `DSH_CLI_NO_HEADERS` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn no_headers(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(NO_HEADERS_ARGUMENT) || std::env::var(ENV_VAR_NO_HEADERS).is_ok() || settings.no_headers.unwrap_or(false)
  }

  /// Gets output format specification
  ///
  /// 1. Try flag `--output-format`
  /// 1. Try environment variable `DSH_CLI_OUTPUT_FORMAT`
  /// 1. Try settings file
  /// 1. Else default to `None`
  fn output_format_specification(matches: &ArgMatches, settings: &Settings) -> Result<Option<OutputFormat>, String> {
    match matches.get_one::<OutputFormat>(OUTPUT_FORMAT_ARGUMENT) {
      Some(output_format_argument) => Ok(Some(output_format_argument.to_owned())),
      None => match std::env::var(ENV_VAR_OUTPUT_FORMAT) {
        Ok(output_format_env_var) => OutputFormat::try_from(output_format_env_var.as_str())
          .map_err(|error| format!("{} in environment variable {}", error, ENV_VAR_OUTPUT_FORMAT))
          .map(Some),
        Err(_) => match settings.output_format.clone() {
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
      None => default_output_format.unwrap_or_else(|| if stdout().is_terminal() { OutputFormat::Table } else { OutputFormat::Json }),
    }
  }

  /// Gets quiet context value
  ///
  /// 1. Try flag `--quiet`
  /// 1. Try if environment variable `DSH_CLI_QUIET` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn quiet(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(QUIET_ARGUMENT) || std::env::var(ENV_VAR_QUIET).is_ok() || settings.quiet.unwrap_or(false)
  }

  /// Gets show_execution_time context value
  ///
  /// 1. Try flag `--show-execution-time`
  /// 1. Try if environment variable `DSH_CLI_SHOW_EXECUTION_TIME` exists
  /// 1. Try settings file
  /// 1. Default to `false`
  fn show_execution_time(matches: &ArgMatches, settings: &Settings) -> bool {
    matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT) || std::env::var(ENV_VAR_SHOW_EXECUTION_TIME).is_ok() || settings.show_execution_time.unwrap_or(false)
  }

  /// Gets terminal width context value
  ///
  /// 1. Try flag `--terminal-width`
  /// 1. Try if environment variable `DSH_CLI_TERMINAL_WIDTH` exists
  /// 1. Try settings file
  /// 1. If stdout is a terminal use actual terminal width, else default to `None`
  fn terminal_width(matches: &ArgMatches, settings: &Settings) -> Result<Option<usize>, String> {
    match matches.get_one::<usize>(TERMINAL_WIDTH_ARGUMENT) {
      Some(terminal_width_argument) => Ok(Some(terminal_width_argument.to_owned())),
      None => match std::env::var(ENV_VAR_TERMINAL_WIDTH) {
        Ok(terminal_width_env_var) => match terminal_width_env_var.parse::<usize>() {
          Ok(terminal_width) => {
            if terminal_width < 40 {
              Err(format!(
                "terminal width in environment variable {} must be greater than or equal to 40",
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
        Err(_) => match settings.terminal_width {
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
  fn verbosity(matches: &ArgMatches, settings: &Settings) -> Result<Verbosity, String> {
    match matches.get_one::<Verbosity>(VERBOSITY_ARGUMENT) {
      Some(verbosity_argument) => Ok(verbosity_argument.to_owned()),
      None => match std::env::var(ENV_VAR_VERBOSITY) {
        Ok(verbosity_env_var) => Verbosity::try_from(verbosity_env_var.as_str()).map_err(|error| format!("{} in environment variable {}", error, ENV_VAR_VERBOSITY)),
        Err(_) => match settings.verbosity.clone() {
          Some(verbosity_from_settings) => Ok(verbosity_from_settings),
          None => Ok(Verbosity::Low),
        },
      },
    }
  }

  /// # Returns current time `Instant`
  pub(crate) fn now(&self) -> Instant {
    Instant::now()
  }

  /// # Prints the output to stdout
  ///
  /// This method is used to print the output of the `dsh` tool to the standard output device.
  /// If `quiet` is `true`, nothing will be printed.
  /// This standard output device can either be a tty, a pipe or an output file,
  /// depending on how the `dsh` tool was run from a shell or script.
  pub(crate) fn print<T: AsRef<str>>(&self, output: T) {
    if !self.quiet {
      println!("{}", wrap_style(self.stdout_style(), output));
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
        OutputFormat::Csv => self.print_warning("csv output is not supported here, use --output-format json|toml|yaml"),
        OutputFormat::Json => match serde_json::to_string_pretty(&output) {
          Ok(json) => println!("{}", wrap_style(self.stdout_style(), json)),
          Err(_) => self.print_error("serializing to json failed"),
        },
        OutputFormat::JsonCompact => match serde_json::to_string(&output) {
          Ok(json) => println!("{}", wrap_style(self.stdout_style(), json)),
          Err(_) => self.print_error("serializing to json failed"),
        },
        OutputFormat::Plain => self.print_warning("plain output is not supported here, use --output-format json|toml|yaml"),
        OutputFormat::Quiet => (),
        OutputFormat::Table | OutputFormat::TableNoBorder => self.print_warning("table output is not supported here, use --output-format json|toml|yaml"),
        OutputFormat::Toml => match toml::to_string_pretty(&output) {
          Ok(json) => println!("{}", wrap_style(self.stdout_style(), json)),
          Err(_) => self.print_error("serializing to toml failed"),
        },
        OutputFormat::TomlCompact => match toml::to_string(&output) {
          Ok(json) => println!("{}", wrap_style(self.stdout_style(), json)),
          Err(_) => self.print_error("serializing to toml failed"),
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&output) {
          Ok(json) => println!("{}", wrap_style(self.stdout_style(), json)),
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
    if !self.quiet && stderr().is_terminal() {
      eprint!("{}", wrap_style(self.stderr_style(), "."));
    }
  }

  /// # Prints a prompt to stderr
  ///
  /// This method is used to print a prompt to the standard error device.
  /// The prompt is used when input from the user is expected.
  /// If `quiet` is `true`, nothing will be printed.
  /// The prompt is only printed when stderr is a terminal,
  /// since it would make no sense for a pipe or output file.
  pub(crate) fn print_prompt<T: AsRef<str>>(&self, prompt: T) {
    if !self.quiet && stderr().is_terminal() {
      eprintln!("{}", wrap_style(self.stderr_style(), prompt));
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
  pub(crate) fn print_outcome<T: AsRef<str>>(&self, outcome: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium | Verbosity::High => eprintln!("{}", wrap_style(self.stderr_style(), outcome)),
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
  pub(crate) fn print_warning<T: AsRef<str>>(&self, warning: T) {
    if !self.quiet {
      match self.verbosity {
        Verbosity::Off => (),
        Verbosity::Low | Verbosity::Medium | Verbosity::High => eprintln!("{}", wrap_style(self.warning_style(), warning)),
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
      eprintln!("{}", wrap_style(self.error_style(), error));
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
        Verbosity::Medium => eprintln!("{}", wrap_style(self.stderr_style(), explanation)),
        Verbosity::High => {
          // if let Some(ref client) = self.dsh_api_client {
          //   eprintln!("{}", wrap_style(self.stderr_style(), format!("{}", client.tenant())));
          // }
          eprintln!("{}", wrap_style(self.stderr_style(), explanation));
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
      eprintln!(
        "{}",
        wrap_style(
          self.stderr_style(),
          format!("execution took {} milliseconds", Instant::now().duration_since(start_instant).as_millis())
        )
      );
    }
  }

  pub(crate) fn read_multi_line(&self, prompt: impl AsRef<str>) -> Result<String, String> {
    if stdin().is_terminal() {
      self.print_prompt(prompt.as_ref());
    }
    let mut multi_line = String::new();
    let stdin = stdin();
    loop {
      match stdin.read_line(&mut multi_line) {
        Ok(0) => break,
        Ok(_) => continue,
        Err(_) => return Err("error reading line".to_string()),
      }
    }
    Ok(multi_line)
  }

  pub(crate) fn read_single_line(&self, prompt: impl AsRef<str>) -> Result<String, String> {
    if stdin().is_terminal() {
      self.print_prompt(prompt.as_ref());
    }
    let _ = stdout().lock().flush();
    let mut line = String::new();
    stdin().read_line(&mut line).expect("could not read line");
    Ok(line.trim().to_string())
  }

  pub(crate) fn read_single_line_password(&self, prompt: impl AsRef<str>) -> Result<String, String> {
    if stdin().is_terminal() {
      match prompt_password(prompt.as_ref()) {
        Ok(line) => Ok(line.trim().to_string()),
        Err(_) => Err("empty input".to_string()),
      }
    } else {
      self.read_single_line(prompt)
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
  /// `self.matching_style` has a value, the `escape` parameter is `true`
  /// and `no_escape` is `false`.
  /// Else the result will be a plain `String`.
  fn parts_to_string(&self, parts: &[Part], escape: bool) -> String {
    if escape && !self.no_escape {
      parts
        .iter()
        .map(|part| match part {
          Matching(p) => {
            let style = self.matching_style();
            format!("{style}{p}{style:#}")
          }
          // wrap_style_color(&self.matching_style, &self.matching_color, p.as_str()),
          NonMatching(p) => p.to_string(),
        })
        .collect::<Vec<_>>()
        .join("")
    } else {
      parts
        .iter()
        .map(|part| match part {
          Matching(p) => p.to_string(),
          NonMatching(p) => p.to_string(),
        })
        .collect::<Vec<_>>()
        .join("")
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

  fn error_style(&self) -> Style {
    style_from(&self.error_style, &self.error_color)
  }

  fn matching_style(&self) -> Style {
    style_from(&self.matching_style, &self.matching_color)
  }

  fn stderr_style(&self) -> Style {
    style_from(&self.stderr_style, &self.stderr_color)
  }

  fn stdout_style(&self) -> Style {
    style_from(&self.stdout_style, &self.stdout_color)
  }

  fn warning_style(&self) -> Style {
    style_from(&self.warning_style, &self.warning_color)
  }
}
