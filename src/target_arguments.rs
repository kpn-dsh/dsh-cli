use crate::arguments::{PLATFORM_NAME_ARGUMENT, TENANT_NAME_ARGUMENT};
use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_PLATFORM, ENV_VAR_DSH_CLI_TENANT};
use crate::error::DshCliError;
use crate::global_arguments::{TARGET_PLATFORM_ARGUMENT, TARGET_TENANT_ARGUMENT};
use crate::settings::Settings;
use crate::{err, error_map, read_single_line, DshCliResult};
use clap::ArgMatches;
use dsh_api::platform::DshPlatform;
use log::debug;
use std::io::{stdin, IsTerminal};

/// # Get the target platform
///
/// This method will get the target platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--platform` or `-p`.
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
/// * `Err<String>` - Error message.
pub(crate) fn get_target_platform(matches: &ArgMatches, settings: &Settings) -> DshCliResult<DshPlatform> {
  match get_target_platform_non_interactive(matches, settings)? {
    Some(platforms_non_interactive) => Ok(platforms_non_interactive),
    None => {
      if stdin().is_terminal() {
        DshPlatform::try_from(read_single_line("target platform: ")?.as_str()).map_err(error_map!("{}"))
      } else {
        err!("could not determine target platform, please check configuration")
      }
    }
  }
}

/// # Get the target tenant without user interaction
///
/// This method will get the target tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--tenant`.
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
  match matches.get_one::<String>(TARGET_TENANT_ARGUMENT) {
    Some(target_tenant_name_from_argument) => {
      debug!("target tenant '{}' (argument)", target_tenant_name_from_argument);
      Ok(Some(target_tenant_name_from_argument.clone()))
    }
    None => get_target_tenant_implicit(matches, settings),
  }
}

/// # Get the target tenant
///
/// This method will get the target tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--tenant`.
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
  match get_target_tenant_non_interactive(matches, settings)? {
    Some(tenant_names_non_interactive) => Ok(tenant_names_non_interactive),
    None => {
      if stdin().is_terminal() {
        let tenant_name_from_console = read_single_line("target tenant: ")?;
        if tenant_name_from_console.is_empty() {
          err!("target tenant name cannot be empty")
        } else {
          Ok(tenant_name_from_console)
        }
      } else {
        err!("could not determine target tenant, please check configuration")
      }
    }
  }
}

/// # Get the target platform and tenant name
///
/// This method will get the target platform and tenant name.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Ok<(DshPlatform, String)>` - Target platform and tenant name.
/// * `Err<String>` - Error message.
pub(crate) fn get_platform_and_tenant(matches: &ArgMatches, settings: &Settings) -> Result<(DshPlatform, String), DshCliError> {
  let platform = match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
    Some(platform_name_from_argument) => DshPlatform::try_from(platform_name_from_argument.as_str())?,
    None => get_target_platform(matches, settings)?,
  };
  let tenant = get_target_tenant(matches, settings)?;
  Ok((platform, tenant))
}

/// # Get platform from argument or prompt
///
/// This method will get the target platform. It will try the sources listed below and returns
/// at the first match.
/// 1. Command line argument `--platform`.
/// 1. Ask the user to enter the value.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Returns
/// * `Ok<Platform>`  - Platform.
/// * `Err<DshCliError>` - Error message.
pub(crate) fn get_platform_argument_or_prompt(matches: &ArgMatches) -> DshCliResult<DshPlatform> {
  match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
    Some(dsh_platform) => Ok(DshPlatform::try_from(dsh_platform.as_str())?),
    None => Ok(DshPlatform::try_from(read_single_line("enter platform: ")?.as_str())?),
  }
}

/// # Get tenant from argument or prompt
///
/// This method will get the tenant name. It will try the sources listed below and returns
/// at the first match.
/// 1. Command line argument `--platform`.
/// 1. Ask the user to enter the value.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Returns
/// * `Ok<String>`  - Tenant name.
/// * `Err<DshCliError>` - Error message.
pub(crate) fn get_tenant_argument_or_prompt(matches: &ArgMatches) -> DshCliResult<String> {
  match matches.get_one::<String>(TENANT_NAME_ARGUMENT) {
    Some(tenant_argument) => Ok(tenant_argument.to_string()),
    None => Ok(read_single_line("enter tenant: ")?),
  }
}

/// # Get the target platform from implicit sources
///
/// This method will get try to find the target platform from the implicit sources listed below,
/// and returns at the first match.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Parameter `default-platform` from settings file, if available.
/// 1. Else return with `None`.
///
/// ## Parameters
/// * `settings` - Contents of the settings file or default settings.
///
/// ## Returns
/// * `Ok(Some<Platform>)` - Target platforms.
/// * `Ok(None)` - When no implicit platform is available.
/// * `Err<String>` - When an invalid platform name was found.
fn get_target_platform_implicit(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<DshPlatform>> {
  match environment_variable(ENV_VAR_DSH_CLI_PLATFORM, Some(matches))? {
    Some(platform_name_from_env_var) => {
      debug!(
        "target platform '{}' (environment variable '{}')",
        platform_name_from_env_var, ENV_VAR_DSH_CLI_PLATFORM
      );
      DshPlatform::try_from(platform_name_from_env_var.as_str()).map_err(error_map!("{}")).map(Some)
    }
    None => match settings.default_platform.clone() {
      Some(default_platform_name_from_settings) => {
        debug!("default target platform '{}' (settings)", default_platform_name_from_settings);
        DshPlatform::try_from(default_platform_name_from_settings.as_str())
          .map_err(error_map!("{}"))
          .map(Some)
      }
      None => Ok(None),
    },
  }
}

/// # Get the target platform without user interaction
///
/// This method will get the target platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--platform`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Parameter `default-platform` from settings file, if available.
/// 1. Else return with `None`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
/// * `settings` - Contents of the settings file.
///
/// ## Returns
/// * `Ok(Option<Platform>)` - The platform.
/// * `Ok(None)` - When no implicit target platform is available.
/// * `Err<String>` - When an invalid platform name was found.
fn get_target_platform_non_interactive(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<DshPlatform>> {
  match matches.get_one::<String>(TARGET_PLATFORM_ARGUMENT) {
    Some(target_platform_name_from_argument) => {
      debug!("target platform '{}' (argument)", target_platform_name_from_argument);
      DshPlatform::try_from(target_platform_name_from_argument.as_str())
        .map_err(error_map!("{}"))
        .map(Some)
    }
    None => get_target_platform_implicit(matches, settings),
  }
}

/// # Get the target tenant from implicit sources
///
/// This method will get try to find the target tenant from the implicit sources listed below,
/// and returns at the first match.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. Parameter `default-tenant` from settings file, if available.
/// 1. Else return with `None`.
///
/// ## Parameters
/// * `settings` - Contents of the settings file or default settings.
///
/// ## Returns
/// * `Some<String>` - Tenant name.
/// * `None` - When no implicit tenant name is available.
fn get_target_tenant_implicit(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<String>> {
  match environment_variable(ENV_VAR_DSH_CLI_TENANT, Some(matches))? {
    Some(tenant_name_from_env_var) => {
      debug!("target tenant '{}' (environment variable '{}')", tenant_name_from_env_var, ENV_VAR_DSH_CLI_TENANT);
      Ok(Some(tenant_name_from_env_var))
    }
    None => match settings.default_tenant.clone() {
      Some(default_tenant_name_from_settings) => {
        debug!("default target tenant '{}' (settings)", default_tenant_name_from_settings);
        Ok(Some(default_tenant_name_from_settings))
      }
      None => Ok(None),
    },
  }
}
