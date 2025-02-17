use crate::formatters::OutputFormat;
use crate::global_arguments::{
  DRY_RUN_ARGUMENT, FORCE_ARGUMENT, MATCHING_STYLE_ARGUMENT, NO_ESCAPE_ARGUMENT, NO_HEADERS_ARGUMENT, OUTPUT_FORMAT_ARGUMENT, QUIET_ARGUMENT, SHOW_EXECUTION_TIME_ARGUMENT,
  TERMINAL_WIDTH_ARGUMENT, VERBOSITY_ARGUMENT,
};
use crate::settings::Settings;
use crate::subject::Requirements;
use crate::verbosity::Verbosity;
use crate::{
  get_target_password, get_target_platform, get_target_platform_implicit, get_target_tenant, get_target_tenant_implicit, ENV_VAR_CSV_QUOTE, ENV_VAR_CSV_SEPARATOR, ENV_VAR_DRY_RUN,
  ENV_VAR_MATCHING_STYLE, ENV_VAR_NO_COLOR, ENV_VAR_NO_ESCAPE, ENV_VAR_NO_HEADERS, ENV_VAR_OUTPUT_FORMAT, ENV_VAR_QUIET, ENV_VAR_SHOW_EXECUTION_TIME, ENV_VAR_TERMINAL_WIDTH,
  ENV_VAR_VERBOSITY,
};
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::platform::DshPlatform;
use dsh_api::query_processor::Part;
use dsh_api::query_processor::Part::{Matching, NonMatching};
use log::debug;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::io::{stderr, stdin, stdout, IsTerminal, Write};
use std::time::Instant;
use terminal_size::{terminal_size, Height, Width};

#[derive(Debug)]
pub(crate) struct Context {
  pub(crate) csv_quote: Option<char>,
  pub(crate) csv_separator: String,
  pub(crate) dry_run: bool,
  pub(crate) dsh_api_client: Option<DshApiClient>,
  pub(crate) force: bool,
  pub(crate) matching_style: Option<MatchingStyle>,
  pub(crate) no_escape: bool,
  pub(crate) output_format: OutputFormat,
  pub(crate) target_platform: Option<DshPlatform>,
  pub(crate) quiet: bool,
  pub(crate) target_tenant_name: Option<String>,
  pub(crate) terminal_width: Option<usize>,
  pub(crate) show_execution_time: bool,
  pub(crate) show_headers: bool,
  pub(crate) _stderr_escape: bool,
  pub(crate) stdin_is_terminal: bool,
  pub(crate) _stdout_escape: bool,
  pub(crate) verbosity: Verbosity,
}

impl Context {
  pub(crate) async fn create(matches: &ArgMatches, requirements: &Requirements, settings: &Settings) -> Result<Context, String> {
    let stdin_is_terminal = stdin().is_terminal();
    let csv_quote = Self::csv_quote(settings)?;
    let csv_separator = Self::csv_separator(settings)?;
    if let Some(quote) = csv_quote {
      if csv_separator.contains(quote) {
        return Err("csv separator string cannot contain quote character".to_string());
      }
    }
    let dry_run = Self::dry_run(matches, settings);
    let target_platform = if requirements.needs_platform() { Some(get_target_platform(matches, settings)?) } else { get_target_platform_implicit(settings)? };
    let target_tenant_name = if requirements.needs_tenant_name() { Some(get_target_tenant(matches, settings)?) } else { get_target_tenant_implicit(settings) };
    let dsh_api_client = if requirements.needs_dsh_api_client() {
      let dsh_api_tenant = DshApiTenant::new(target_tenant_name.clone().unwrap(), target_platform.clone().unwrap());
      let password = get_target_password(matches, &dsh_api_tenant)?;
      let dsh_api_client_factory = DshApiClientFactory::create(dsh_api_tenant, password)?;
      let client = dsh_api_client_factory.client().await?;
      debug!("api client created");
      Some(client)
    } else {
      debug!("no api client required");
      None
    };
    let no_escape = Self::no_escape(matches, settings);
    let (matching_style, stderr_escape, stdout_escape) =
      if no_escape { (None, false, false) } else { (Self::matching_style(matches, settings)?, stderr().is_terminal(), stdout().is_terminal()) };
    let quiet = Self::quiet(matches, settings);
    let force = Self::force(matches, settings);
    let (output_format, show_execution_time, verbosity) = if quiet {
      (OutputFormat::Quiet, false, Verbosity::Off)
    } else {
      (
        Self::output_format(matches, settings, requirements.default_output_format())?,
        Self::show_execution_time(matches, settings),
        Self::verbosity(matches, settings)?,
      )
    };
    let show_headers = !Self::no_headers(matches, settings);
    let terminal_width = Self::terminal_width(matches, settings)?;
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
      target_platform,
      quiet,
      show_execution_time,
      show_headers,
      target_tenant_name,
      terminal_width,
      _stderr_escape: stderr_escape,
      stdin_is_terminal,
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

  /// Gets matching_style context value
  ///
  /// 1. Try flag `--matching-style`
  /// 1. Try environment variable `DSH_CLI_MATCHING_STYLE`
  /// 1. Try settings file
  /// 1. Default to `None`
  fn matching_style(matches: &ArgMatches, settings: &Settings) -> Result<Option<MatchingStyle>, String> {
    match matches.get_one::<MatchingStyle>(MATCHING_STYLE_ARGUMENT) {
      Some(matching_style_argument) => Ok(Some(matching_style_argument.to_owned())),
      None => match std::env::var(ENV_VAR_MATCHING_STYLE) {
        Ok(matching_style_env_var) => MatchingStyle::try_from(matching_style_env_var.as_str()).map(Some),
        Err(_) => Ok(settings.matching_style.clone()),
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

  /// Gets output_format context value
  ///
  /// 1. Try flag `--output-format`
  /// 1. Try environment variable `DSH_CLI_OUTPUT_FORMAT`
  /// 1. Try settings file
  /// 1. Try default_output_format parameter
  /// 1. If stdout is a terminal default to `OutputFormat::Table`,
  ///    else default to `OutputFormat::Json`
  fn output_format(matches: &ArgMatches, settings: &Settings, default_output_format: Option<OutputFormat>) -> Result<OutputFormat, String> {
    match matches.get_one::<OutputFormat>(OUTPUT_FORMAT_ARGUMENT) {
      Some(output_format_argument) => Ok(output_format_argument.to_owned()),
      None => match std::env::var(ENV_VAR_OUTPUT_FORMAT) {
        Ok(output_format_env_var) => OutputFormat::try_from(output_format_env_var.as_str()).map_err(|error| format!("{} in environment variable {}", error, ENV_VAR_OUTPUT_FORMAT)),
        Err(_) => match settings.output_format.clone() {
          Some(output_format_from_settings) => Ok(output_format_from_settings),
          None => match default_output_format {
            Some(output_format_from_default) => Ok(output_format_from_default),
            None => Ok(if stdout().is_terminal() { OutputFormat::Table } else { OutputFormat::Json }),
          },
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

  /// # Prints serializable output to stdout
  ///
  /// This method is used to print a serialized version of the output of the tool
  /// to the standard output device.
  /// If `quiet` is `true`, nothing will be printed.
  /// This standard output device can either be a tty, a pipe or an output file,
  /// depending on how the tool was run from a shell or script.
  pub(crate) fn print_serializable<T: Serialize>(&self, output: T) {
    if !self.quiet {
      match self.output_format {
        OutputFormat::Json => match serde_json::to_string_pretty(&output) {
          Ok(json) => println!("{}", json),
          Err(_) => self.print_error("serializing to json failed"),
        },
        OutputFormat::JsonCompact => match serde_json::to_string(&output) {
          Ok(json) => println!("{}", json),
          Err(_) => self.print_error("serializing to json failed"),
        },
        OutputFormat::Toml => match toml::to_string_pretty(&output) {
          Ok(json) => println!("{}", json),
          Err(_) => self.print_error("serializing to toml failed"),
        },
        OutputFormat::TomlCompact => match toml::to_string(&output) {
          Ok(json) => println!("{}", json),
          Err(_) => self.print_error("serializing to toml failed"),
        },
        OutputFormat::Yaml => match serde_yaml::to_string(&output) {
          Ok(json) => println!("{}", json),
          Err(_) => self.print_error("serializing to yaml failed"),
        },
        OutputFormat::Csv => self.print_warning("csv output is not supported here, use flag --output-format json|toml|yaml"),
        OutputFormat::Plain => self.print_warning("plain output is not supported here, use flag --output-format json|toml|yaml"),
        OutputFormat::Quiet => (),
        OutputFormat::Table | OutputFormat::TableNoBorder => self.print_warning("table output is not supported here, use flag --output-format json|toml|yaml"),
      }
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

  // TODO Needs better testing
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
  /// Matches will be displayed reversed
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
