use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_TENANT};
use crate::global_options::TENANT_OPTION;
use crate::settings::Settings;
use crate::{err, read_single_line, DshCliResult};
use clap::{builder, Arg, ArgAction, ArgMatches};
use log::debug;
use std::io::{stdin, IsTerminal};

/// # Get the target tenant
///
/// This method will get the target tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--tenant` or `-t`.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. Parameter `default-tenant` from settings file, if available.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Ok<String>` Tenant name.
/// * `Err<String>` - Error message.
pub(crate) fn get_target_tenant(matches: &ArgMatches, settings: &Settings) -> DshCliResult<String> {
  if let Some(target_tenant) = get_target_tenant_from_command_line_option(matches) {
    Ok(target_tenant)
  } else if let Some(target_tenant) = get_target_tenant_from_environment_variable(matches)? {
    Ok(target_tenant)
  } else if let Some(target_tenant) = get_target_tenant_from_settings(settings) {
    Ok(target_tenant)
  } else {
    get_target_tenant_from_user()
  }
}

/// # Get target tenant from argument, option or prompt
///
/// This method will get the tenant name explicit. It will try the sources listed below and
/// returns at the first match.
/// 1. Command line argument.
/// 1. Command line option `--tenant` or `-t`.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Panics
/// This function will panic when the tenant name has not been defined as a command line argument
/// (using [`tenant_name_argument`]).
///
/// ## Returns
/// * `Ok<String>`  - Tenant name.
/// * `Err<DshCliError>` - Error message.
pub(crate) fn get_target_tenant_explicit(matches: &ArgMatches) -> DshCliResult<String> {
  if let Some(target_tenant) = get_target_tenant_from_command_line_argument(matches) {
    Ok(target_tenant)
  } else if let Some(target_tenant) = get_target_tenant_from_command_line_option(matches) {
    Ok(target_tenant)
  } else {
    get_target_tenant_from_user()
  }
}

/// # Get the target tenant without asking the user
///
/// This method will get the target tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--tenant` or `-t`.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. Parameter `default-tenant` from settings file, if available.
/// 1. Else return with `None`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Some<String>` - Tenant name.
/// * `None` - When no tenant name is available without asking the user.
pub(crate) fn get_target_tenant_non_interactive(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<String>> {
  if let Some(target_tenant) = get_target_tenant_from_command_line_option(matches) {
    Ok(Some(target_tenant))
  } else if let Some(target_tenant) = get_target_tenant_from_environment_variable(matches)? {
    Ok(Some(target_tenant))
  } else if let Some(target_tenant) = get_target_tenant_from_settings(settings) {
    Ok(Some(target_tenant))
  } else {
    Ok(None)
  }
}

const TENANT_NAME_ARGUMENT: &str = "tenant-name-argument";

/// # Get the target tenant from command line argument
///
/// This method will get the target tenant from the command line argument.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Panics
/// This function will panic when the tenant name has not been defined as a command line argument
/// (using [`tenant_name_argument`]).
///
/// ## Returns
/// * `Some(String)` - The target tenant when it is available as command line argument.
/// * `None` - When no target tenant is available as an argument on the command line.
pub(crate) fn get_target_tenant_from_command_line_argument(matches: &ArgMatches) -> Option<String> {
  match matches.get_one::<String>(TENANT_NAME_ARGUMENT) {
    Some(target_tenant) => {
      debug!("target tenant '{}' (from command line argument)", target_tenant);
      Some(target_tenant.clone())
    }
    None => None,
  }
}

/// Create target tenant command line argument
///
/// Creates a command line argument that allows commands to get the target tenant as an argument.
/// If a target tenant is required, use the function
/// [`get_target_tenant_from_command_line_argument`] to get the value.
pub(crate) fn tenant_name_argument() -> Arg {
  Arg::new(TENANT_NAME_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Tenant name")
    .long_help("The name of the tenant.")
}

/// # Get the target tenant from command line option
///
/// This method will get the target tenant from the command line option `--tenant` or `-t`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Returns
/// * `Some(String)` - The tenant name when it is available as a command line option.
/// * `None` - When no tenant name is available on the command line.
fn get_target_tenant_from_command_line_option(matches: &ArgMatches) -> Option<String> {
  match matches.get_one::<String>(TENANT_OPTION) {
    Some(target_tenant) => {
      debug!("target tenant '{}' (from command line option '--tenant')", target_tenant);
      Some(target_tenant.clone())
    }
    None => None,
  }
}

/// # Get the target tenant from environment variable
///
/// This method will get the target tenant from the environment variable `DSH_CLI_TENANT`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Returns
/// * `Ok(Some(String))` - The target tenant when it is specified in the environment variable.
/// * `Ok(None)` - When the environment variable is not set.
/// * `Err<DshCliError>` - When the environment variable could not be read.
fn get_target_tenant_from_environment_variable(matches: &ArgMatches) -> DshCliResult<Option<String>> {
  match environment_variable(ENV_VAR_DSH_CLI_TENANT, Some(matches))? {
    Some(target_tenant) => {
      debug!("target tenant '{}' (from environment variable '{}')", target_tenant, ENV_VAR_DSH_CLI_TENANT);
      Ok(Some(target_tenant))
    }
    None => Ok(None),
  }
}

/// # Get the target tenant from settings
///
/// This method will get the target tenant from the setting `default-tenant`.
///
/// ## Parameters
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Some(String)` - The tenant when it is specified in the settings.
/// * `None` - When the environment variable is not defined in the settings.
fn get_target_tenant_from_settings(settings: &Settings) -> Option<String> {
  match settings.default_tenant.clone() {
    Some(target_tenant) => {
      debug!("target tenant '{}' (from settings file)", target_tenant);
      Some(target_tenant)
    }
    None => None,
  }
}

/// # Get the target tenant from the user
///
/// This method will ask the user to enter the target tenant.
///
/// ## Returns
/// * `Ok(String)` - The user entered the tenant.
/// * `Err<DshCliError>` - When the tenant could not be provided by the user.
fn get_target_tenant_from_user() -> DshCliResult<String> {
  if stdin().is_terminal() {
    let target_tenant = read_single_line("target tenant: ")?;
    if target_tenant.is_empty() {
      err!("target tenant name cannot be empty")
    } else {
      debug!("target tenant '{}' (entered by user)", target_tenant);
      Ok(target_tenant)
    }
  } else {
    err!("could not get target tenant from terminal, please check configuration")
  }
}
