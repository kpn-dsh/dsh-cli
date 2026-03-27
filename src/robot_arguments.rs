use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_PASSWORD, ENV_VAR_DSH_CLI_PASSWORD_FILE, ENV_VAR_DSH_CLI_PLATFORM, ENV_VAR_DSH_CLI_TENANT};
use crate::error::DshCliError;
use crate::global_options::{PASSWORD_FILE_OPTION, PLATFORM_OPTION, TENANTS_ALL_FLAG, TENANTS_OPTION, TENANT_OPTION};
use crate::keyring::get_secret_from_keyring;
use crate::{err, error_map, read_single_line, read_single_line_password, DshCliResult};
use clap::ArgMatches;
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::platform::DshPlatform;
use dsh_api::secret::ROBOT_SECRET;
use log::debug;
use std::fs;
use std::io::{stdin, IsTerminal};
use std::path::{Path, PathBuf};

/// # Get the robot platform
///
/// This method will get the robot platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--platform` or `-p`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Returns
/// * `Ok<Platform>`  - Robot platform.
/// * `Err<String>` - Error message.
pub(crate) fn get_robot_platform(matches: &ArgMatches) -> DshCliResult<DshPlatform> {
  match get_robot_platform_non_interactive(matches)? {
    Some(platform_non_interactive) => Ok(platform_non_interactive),
    None => {
      if stdin().is_terminal() {
        DshPlatform::try_from(read_single_line("robot platform: ")?.as_str()).map_err(error_map!("{}"))
      } else {
        err!("could not determine robot platform, please check configuration")
      }
    }
  }
}

/// # Get the robot tenant
///
/// This method will get the robot tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--tenant` or `-t`.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Returns
/// * `Ok<String>` Robot tenant name.
/// * `Err<String>` - Error message.
pub(crate) fn get_robot_tenant(matches: &ArgMatches) -> DshCliResult<String> {
  if matches.get_one::<String>(TENANTS_OPTION).is_some() {
    err!("option --tenants not allowed for robot authentication method")
  } else if matches.get_flag(TENANTS_ALL_FLAG) {
    err!("option --all-tenants not allowed for robot authentication method")
  } else {
    match get_robot_tenant_non_interactive(matches)? {
      Some(robot_tenant_name_non_interactive) => Ok(robot_tenant_name_non_interactive),
      None => {
        if stdin().is_terminal() {
          let robot_tenant_name_from_console = read_single_line("robot tenant: ")?;
          if robot_tenant_name_from_console.is_empty() {
            err!("robot tenant name cannot be empty")
          } else {
            Ok(robot_tenant_name_from_console)
          }
        } else {
          err!("could not determine robot tenant, please check configuration")
        }
      }
    }
  }
}

/// # Get the robot password
///
/// This method will get the robot password.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--password-file`, which should reference a file that
///    contains the password.
/// 1. Environment variable `DSH_CLI_PASSWORD_FILE`.
/// 1. Environment variable `DSH_CLI_PASSWORD`. Note that this environment variable must be a
///    regular environment variable and cannot be specified via the command line.
/// 1. Check entry from the keyring, if available. This can result in a pop-up where the user
///    must authenticate for the keyring.
/// 1. If stdin is a terminal, ask the user to enter the password.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
/// * `dsh_api_tenant` - Used to determine the target settings file.
///
/// ## Returns
/// * `Ok<String>` - Password.
pub(crate) fn get_robot_password(matches: &ArgMatches, dsh_api_tenant: &DshApiTenant) -> DshCliResult<String> {
  match matches.get_one::<PathBuf>(PASSWORD_FILE_OPTION) {
    Some(password_file_from_arg) => read_robot_password_file(password_file_from_arg),
    None => match environment_variable(ENV_VAR_DSH_CLI_PASSWORD_FILE, Some(matches))? {
      Some(password_file_from_env) => read_robot_password_file(password_file_from_env),
      None => match environment_variable(ENV_VAR_DSH_CLI_PASSWORD, None)? {
        Some(password_from_env_var) => {
          debug!("robot password (from environment variable '{}')", ENV_VAR_DSH_CLI_PASSWORD);
          Ok(password_from_env_var)
        }
        None => match get_secret_from_keyring(dsh_api_tenant.platform(), dsh_api_tenant.name(), ROBOT_SECRET) {
          Ok(Some(password_from_keyring)) => {
            debug!("robot password (from keyring)");
            Ok(password_from_keyring)
          }
          Ok(None) | Err(DshCliError::Canceled) => {
            if stdin().is_terminal() {
              let password = read_single_line_password(format!("robot password for tenant {}: ", dsh_api_tenant).as_str())?;
              if password.is_empty() {
                err!("password cannot be empty")
              } else {
                Ok(password)
              }
            } else {
              err!("could not determine robot password and unable to to prompt user, please check configuration")
            }
          }
          Err(error) => Err(error),
        },
      },
    },
  }
}

/// # Get the robot platform from implicit sources
///
/// This method will get try to find the robot platform from the implicit sources listed below,
/// and returns at the first match.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Else return with `None`.
///
/// ## Returns
/// * `Ok(Some<Platform>)` - Robot platforms.
/// * `Ok(None)` - When no implicit robot platform is available.
/// * `Err<String>` - When an invalid robot platform name was found.
fn get_robot_platform_implicit(matches: &ArgMatches) -> DshCliResult<Option<DshPlatform>> {
  match environment_variable(ENV_VAR_DSH_CLI_PLATFORM, Some(matches))? {
    Some(robot_platform_name_from_env_var) => {
      debug!(
        "robot platform '{}' (from environment variable '{}')",
        robot_platform_name_from_env_var, ENV_VAR_DSH_CLI_PLATFORM
      );
      DshPlatform::try_from(robot_platform_name_from_env_var.as_str()).map_err(error_map!("{}")).map(Some)
    }
    None => Ok(None),
  }
}

/// # Get the robot platform without user interaction
///
/// This method will get the robot platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--platform` or `-p`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Else return with `None`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments,
///
/// ## Returns
/// * `Ok(Option<Platform>)` - The robot platform.
/// * `Ok(None)` - When no implicit robot platform is available.
/// * `Err<String>` - When an invalid robot platform name was found.
fn get_robot_platform_non_interactive(matches: &ArgMatches) -> DshCliResult<Option<DshPlatform>> {
  match matches.get_one::<String>(PLATFORM_OPTION) {
    Some(robot_platform_name_from_argument) => {
      debug!("robot platform '{}' (from command line argument '--platform')", robot_platform_name_from_argument);
      DshPlatform::try_from(robot_platform_name_from_argument.as_str())
        .map_err(error_map!("{}"))
        .map(Some)
    }
    None => get_robot_platform_implicit(matches),
  }
}

/// # Get the robot tenant from implicit sources
///
/// This method will get try to find the robot tenant from the implicit sources listed below,
/// and returns at the first match.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. Else return with `None`.
///
/// ## Returns
/// * `Some<String>` - Tenant name.
/// * `None` - When no implicit tenant name is available.
fn get_robot_tenant_implicit(matches: &ArgMatches) -> DshCliResult<Option<String>> {
  match environment_variable(ENV_VAR_DSH_CLI_TENANT, Some(matches))? {
    Some(robot_name_from_env_var) => {
      debug!(
        "robot tenant '{}' (from environment variable '{}')",
        robot_name_from_env_var, ENV_VAR_DSH_CLI_TENANT
      );
      Ok(Some(robot_name_from_env_var))
    }
    None => Ok(None),
  }
}

/// # Get the robot tenant without user interaction
///
/// This method will get the robot tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line option `--tenant` or `-t`.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. Else return with `None`.
///
/// ## Parameters
/// * `matches` - Parsed clap command line arguments.
///
/// ## Returns
/// * `Some<String>` - Robot tenant name.
/// * `None` - When no robot tenant name is available without asking the user.
fn get_robot_tenant_non_interactive(matches: &ArgMatches) -> DshCliResult<Option<String>> {
  match matches.get_one::<String>(TENANT_OPTION) {
    Some(robot_tenant_name_from_argument) => {
      debug!("robot tenant '{}' (from command line argument '--tenant')", robot_tenant_name_from_argument);
      Ok(Some(robot_tenant_name_from_argument.clone()))
    }
    None => get_robot_tenant_implicit(matches),
  }
}

fn read_robot_password_file<T: AsRef<Path>>(password_file: T) -> DshCliResult<String> {
  match fs::read_to_string(&password_file) {
    Ok(password_string) => {
      let trimmed_password = password_string.trim();
      if trimmed_password.is_empty() {
        err!("target password file '{}' is empty", password_file.as_ref().display())
      } else {
        debug!("target password (from password file '{}')", password_file.as_ref().display());
        Ok(trimmed_password.to_string())
      }
    }
    Err(_) => err!("target password file '{}' could not be read", password_file.as_ref().display()),
  }
}
