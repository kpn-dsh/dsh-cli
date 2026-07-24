use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_PLATFORM};
use crate::error::DshCliError;
use crate::global_options::PLATFORM_OPTION;
use crate::settings::Settings;
use crate::{err, read_single_line, DshCliResult};
use clap::builder::PossibleValue;
use clap::{Arg, ArgAction, ArgMatches};
use dsh_api::platform::DshPlatform;
use itertools::Itertools;
use log::debug;
use std::io::{stdin, IsTerminal};
use std::str::FromStr;

/// # Get the target platform from all sources
///
/// This method will get the target platform. It will try all possible sources listed below,
/// and returns at the first match.
/// 1. Command line argument.
/// 1. Command line option `--platform` or `-p`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Parameter `default-platform` from settings file.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Panics
/// This function will panic when the target platform has not been defined as a command line argument
/// (using [`platform_name_argument`]).
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Ok<Platform>`  - Target platform.
/// * `Err<DshCliError>` - Error message.
pub(crate) fn get_target_platform_from_all_sources(matches: &ArgMatches, settings: &Settings) -> DshCliResult<DshPlatform> {
  Ok(match get_target_platform_from_command_line_argument(matches)? {
    Some(target_platform) => target_platform,
    None => get_target_platform(matches, settings)?,
  })
}

/// # Get the target platform
///
/// This method will get the target platform. It will try the potential sources listed below,
/// and returns at the first match.
/// 1. Command line option `--platform` or `-p`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Parameter `default-platform` from settings file.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Ok<Platform>`  - Target platform.
/// * `Err<DshCliError>` - Error message.
pub(crate) fn get_target_platform(matches: &ArgMatches, settings: &Settings) -> DshCliResult<DshPlatform> {
  if let Some(target_platform) = get_target_platform_from_command_line_option(matches)? {
    Ok(target_platform)
  } else if let Some(target_platform) = get_target_platform_from_environment_variable(matches)? {
    Ok(target_platform)
  } else if let Some(target_platform) = get_target_platform_from_settings(settings)? {
    Ok(target_platform)
  } else {
    get_target_platform_from_user()
  }
}

/// # Get target platform from argument, option or prompt
///
/// This method will get the target platform explicit. It will try the sources listed below and
/// returns at the first match.
/// 1. Command line argument.
/// 1. Command line option `--platform` or `-p`.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Panics
/// This function will panic when the target platform has not been defined as a command line argument
/// (using [`platform_name_argument`]).
///
/// ## Returns
/// * `Ok<Platform>`  - Platform.
/// * `Err<DshCliError>` - Error message.
pub(crate) fn get_target_platform_explicit(matches: &ArgMatches) -> DshCliResult<DshPlatform> {
  if let Some(target_platform) = get_target_platform_from_command_line_argument(matches)? {
    Ok(target_platform)
  } else if let Some(target_platform) = get_target_platform_from_command_line_option(matches)? {
    Ok(target_platform)
  } else {
    get_target_platform_from_user()
  }
}

const PLATFORM_NAME_ARGUMENT: &str = "platform-name-argument";

/// Create target platform command line argument
///
/// Creates a command line argument that allows commands to get the target platform as an argument.
/// If a target platform is required, use the function
/// [`get_target_platform_from_command_line_argument`] to get the value.
pub(crate) fn platform_name_argument() -> Arg {
  let possible_values = DshPlatform::all()
    .iter()
    .map(|platform| {
      PossibleValue::new(platform.name())
        .alias(platform.alias())
        .help(format!("{} ({})", platform.description(), platform.alias()))
    })
    .collect_vec();
  Arg::new(PLATFORM_NAME_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(possible_values)
    .value_name("PLATFORM")
    .help("Platform")
    .long_help("The name or alias of the platform.")
}

/// # Get the target platform from command line argument
///
/// This method will get the target platform from the command line argument.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Panics
/// This function will panic when the target platform has not been defined as a command line argument
/// (using [`platform_name_argument`]).
///
/// ## Returns
/// * `Ok(Some(Platform))` - The platform when it is available as command line argument and valid.
/// * `Ok(None)` - When no target platform is available on the command line.
/// * `Err<DshCliError>` - When an invalid platform name was found.
pub(crate) fn get_target_platform_from_command_line_argument(matches: &ArgMatches) -> DshCliResult<Option<DshPlatform>> {
  match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
    Some(target_platform_name) => match DshPlatform::from_str(target_platform_name) {
      Ok(target_platform) => {
        debug!("target platform '{}' (from command line argument)", target_platform);
        Ok(Some(target_platform))
      }
      Err(error) => Err(DshCliError::from(error)),
    },
    None => Ok(None),
  }
}

/// # Get the target platform from command line option
///
/// This method will get the target platform from the command line option `--platform` or `-p`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Returns
/// * `Ok(Some(Platform))` - The platform when it is available on the command line and valid.
/// * `Ok(None)` - When no target platform is available on the command line.
/// * `Err<DshCliError>` - When an invalid platform name was found.
fn get_target_platform_from_command_line_option(matches: &ArgMatches) -> DshCliResult<Option<DshPlatform>> {
  match matches.get_one::<String>(PLATFORM_OPTION) {
    Some(target_platform_name) => match DshPlatform::from_str(target_platform_name) {
      Ok(target_platform) => {
        debug!("target platform '{}' (from command line option '--platform')", target_platform);
        Ok(Some(target_platform))
      }
      Err(error) => Err(DshCliError::from(error)),
    },
    None => Ok(None),
  }
}

/// # Get the target platform from environment variable
///
/// This method will get the target platform from the environment variable `DSH_CLI_PLATFORM`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Returns
/// * `Ok(Some(Platform))` - The platform when it is specified in the environment variable
///   and valid.
/// * `Ok(None)` - When the environment variable is not set.
/// * `Err<DshCliError>` - When an invalid platform name was specified.
fn get_target_platform_from_environment_variable(matches: &ArgMatches) -> DshCliResult<Option<DshPlatform>> {
  match environment_variable(ENV_VAR_DSH_CLI_PLATFORM, Some(matches))? {
    Some(target_platform_name) => match DshPlatform::from_str(&target_platform_name) {
      Ok(target_platform) => {
        debug!("target platform '{}' (from environment variable '{}')", target_platform, ENV_VAR_DSH_CLI_PLATFORM);
        Ok(Some(target_platform))
      }
      Err(error) => Err(DshCliError::from(error)),
    },
    None => Ok(None),
  }
}

/// # Get the target platform from settings
///
/// This method will get the target platform from the setting `default-platform`.
///
/// ## Parameters
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Ok(Some(Platform))` - The platform when it is specified in the settings and valid.
/// * `Ok(None)` - When the environment variable is not defined in the settings.
/// * `Err<DshCliError>` - When an invalid platform name was specified.
fn get_target_platform_from_settings(settings: &Settings) -> DshCliResult<Option<DshPlatform>> {
  match settings.default_platform.clone() {
    Some(target_platform_name) => match DshPlatform::from_str(&target_platform_name) {
      Ok(target_platform) => {
        debug!("target platform '{}' (from settings file)", target_platform);
        Ok(Some(target_platform))
      }
      Err(error) => Err(DshCliError::from(error)),
    },
    None => Ok(None),
  }
}

/// # Get the target platform from the user
///
/// This method will ask the user to enter the target platform.
///
/// ## Returns
/// * `Ok(Platform)` - The user entered a valid platform.
/// * `Err<DshCliError>` - When an invalid platform name was provided.
fn get_target_platform_from_user() -> DshCliResult<DshPlatform> {
  if stdin().is_terminal() {
    let target_platform_name = read_single_line("target platform: ")?;
    if target_platform_name.is_empty() {
      err!("target platform name cannot be empty")
    } else {
      match DshPlatform::from_str(&target_platform_name) {
        Ok(target_platform) => {
          debug!("target platform '{}' (entered by user)", target_platform);
          Ok(target_platform)
        }
        Err(error) => Err(DshCliError::from(error)),
      }
    }
  } else {
    err!("could not get target platform from terminal, please check configuration")
  }
}
