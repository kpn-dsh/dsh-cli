#![doc(
  html_favicon_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
#![doc(
  html_logo_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
extern crate core;

use autocomplete::{generate_autocomplete_file, generate_autocomplete_file_argument, AutocompleteShell, AUTOCOMPLETE_ARGUMENT};
use clap::builder::styling::{AnsiColor, Color, Style};
use clap::builder::{styling, Styles};
use clap::error::{Error as ClapError, ErrorKind};
use clap::{ArgMatches, Command};
use context::Context;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::platform::DshPlatform;
use dsh_api::{crate_version, openapi_version};
use filter_flags::FilterFlagType;
use global_arguments::{
  dry_run_argument, force_argument, no_escape_argument, no_headers_argument, output_format_argument, quiet_argument, set_verbosity_argument, show_execution_time_argument,
  suppress_exit_status_argument, target_password_file_argument, target_platform_argument, target_tenant_argument, terminal_width_argument, version_argument,
  TARGET_PASSWORD_FILE_ARGUMENT, TARGET_PLATFORM_ARGUMENT, TARGET_TENANT_ARGUMENT, VERSION_ARGUMENT,
};
use homedir::my_home;
use lazy_static::lazy_static;
use log::{debug, trace};
use log_arguments::{log_level_api_argument, log_level_argument, log_level_sdk_argument};
use log_level::initialize_logger;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use settings::{get_settings, Settings};
use std::collections::HashMap;
use std::env::temp_dir;
use std::fmt::Debug;
use std::io::ErrorKind::NotFound;
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Termination};
use std::{env, fs, process};
use style::{style_from, wrap_style, DshColor, DshStyle};
use subject::Subject;
use subjects::api::API_SUBJECT;
use subjects::app::APP_SUBJECT;
use subjects::bucket::BUCKET_SUBJECT;
use subjects::certificate::CERTIFICATE_SUBJECT;
use subjects::env::ENV_SUBJECT;
use subjects::image::IMAGE_SUBJECT;
use subjects::manifest::MANIFEST_SUBJECT;
use subjects::metric::METRIC_SUBJECT;
use subjects::platform::PLATFORM_SUBJECT;
use subjects::proxy::PROXY_SUBJECT;
use subjects::secret::SECRET_SUBJECT;
use subjects::service::SERVICE_SUBJECT;
use subjects::setting::SETTING_SUBJECT;
use subjects::target::TARGET_SUBJECT;
#[cfg(feature = "manage")]
use subjects::tenant::TENANT_SUBJECT;
use subjects::token::TOKEN_SUBJECT;
use subjects::topic::TOPIC_SUBJECT;
use subjects::vhost::VHOST_SUBJECT;
use subjects::volume::VOLUME_SUBJECT;
use targets::{get_target_password_from_keyring, read_target};

mod arguments;
mod autocomplete;
mod capability;
mod capability_builder;
mod context;
mod filter_flags;
mod flags;
mod formatters;
mod global_arguments;
mod limits_flags;
mod log_arguments;
mod log_level;
mod modifier_flags;
mod settings;
mod style;
mod subject;
mod subjects;
mod targets;
mod verbosity;
mod version;

lazy_static! {
  static ref STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | styling::Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default());
}

pub(crate) const APPLICATION_NAME: &str = "dsh";

/// Short help text, shown when `-h` was provided
const ABOUT: &str = "DSH resource management api command line interface.";
const AUTHOR: &str = "KPN DSH Team, unibox@kpn.com";
/// Long help text, shown when `--help` was provided
const LONG_ABOUT: &str = "DSH resource management api command line interface\n\n\
   The DSH api command line tool enables the user to call a subset of the functions \
   in the DSH api from the command line. \
   It also supports functions that are not supported directly from the DSH api, \
   such as finding all services that use a certain resource (e.g. a secret) or showing a \
   list of all resources of a certain type (e.g. list all volumes).";
/// Will be shown after normal help text, when `-h` was provided
const AFTER_HELP: &str = "For most commands adding an 's' as a postfix will yield the same result \
   as using the 'list' subcommand, e.g. using 'dsh apps' will be the same \
   as using 'dsh app list'.";

const VERSION: &str = "0.7.3";

const ENV_VAR_PREFIX: &str = "DSH_CLI_";

// Duplicate from dsh_api crate
const ENV_VAR_PLATFORMS_FILE_NAME: &str = "DSH_API_PLATFORMS_FILE";

const ENV_VAR_CSV_QUOTE: &str = "DSH_CLI_CSV_QUOTE";
const ENV_VAR_CSV_SEPARATOR: &str = "DSH_CLI_CSV_SEPARATOR";
const ENV_VAR_DRY_RUN: &str = "DSH_CLI_DRY_RUN";
const ENV_VAR_ERROR_COLOR: &str = "DSH_CLI_ERR_COLOR";
const ENV_VAR_ERROR_STYLE: &str = "DSH_CLI_ERROR_STYLE";
const ENV_VAR_HOME_DIRECTORY: &str = "DSH_CLI_HOME";
const ENV_VAR_LOG_LEVEL: &str = "DSH_CLI_LOG_LEVEL";
const ENV_VAR_LOG_LEVEL_API: &str = "DSH_CLI_LOG_LEVEL_API";
const ENV_VAR_LOG_LEVEL_SDK: &str = "DSH_CLI_LOG_LEVEL_SDK";
const ENV_VAR_MATCHING_COLOR: &str = "DSH_CLI_MATCHING_COLOR";
const ENV_VAR_MATCHING_STYLE: &str = "DSH_CLI_MATCHING_STYLE";
const ENV_VAR_NO_COLOR: &str = "NO_COLOR";
const ENV_VAR_NO_ESCAPE: &str = "DSH_CLI_NO_ESCAPE";
const ENV_VAR_NO_HEADERS: &str = "DSH_CLI_NO_HEADERS";
const ENV_VAR_OUTPUT_FORMAT: &str = "DSH_CLI_OUTPUT_FORMAT";
const ENV_VAR_PASSWORD: &str = "DSH_CLI_PASSWORD";
const ENV_VAR_PASSWORD_FILE: &str = "DSH_CLI_PASSWORD_FILE";
const ENV_VAR_PLATFORM: &str = "DSH_CLI_PLATFORM";
const ENV_VAR_QUIET: &str = "DSH_CLI_QUIET";
const ENV_VAR_SHOW_EXECUTION_TIME: &str = "DSH_CLI_SHOW_EXECUTION_TIME";
const ENV_VAR_STDERR_COLOR: &str = "DSH_CLI_STDERR_COLOR";
const ENV_VAR_STDERR_STYLE: &str = "DSH_CLI_STDERR_STYLE";
const ENV_VAR_STDOUT_COLOR: &str = "DSH_CLI_STDOUT_COLOR";
const ENV_VAR_STDOUT_STYLE: &str = "DSH_CLI_STDOUT_STYLE";
const ENV_VAR_SUPPRESS_EXIT_STATUS: &str = "DSH_CLI_SUPPRESS_EXIT_STATUS";
const ENV_VAR_TENANT: &str = "DSH_CLI_TENANT";
const ENV_VAR_TERMINAL_WIDTH: &str = "DSH_CLI_TERMINAL_WIDTH";
const ENV_VAR_VERBOSITY: &str = "DSH_CLI_VERBOSITY";
const ENV_VAR_WARNING_COLOR: &str = "DSH_CLI_WARNING_COLOR";
const ENV_VAR_WARNING_STYLE: &str = "DSH_CLI_WARNING_STYLE";

const DEFAULT_USER_DSH_CLI_DIRECTORY: &str = ".dsh_cli";
const TARGETS_SUBDIRECTORY: &str = "targets";
const DEFAULT_DSH_CLI_SETTINGS_FILENAME: &str = "settings.toml";
const TOML_FILENAME_EXTENSION: &str = "toml";

type DshCliResult = Result<(), String>;

#[derive(Debug)]
enum DshCliExit {
  Ok,
  OkClap(ClapError),
  Err(String),
  ErrClap(ClapError),
  ErrContext(String, Box<Context>),
}

impl Termination for DshCliExit {
  fn report(self) -> ExitCode {
    match self {
      DshCliExit::Ok => ExitCode::SUCCESS,
      DshCliExit::OkClap(clap_error) => {
        let _ = clap_error.print();
        ExitCode::SUCCESS
      }
      DshCliExit::Err(msg) => {
        eprintln!("{}", wrap_style(default_error_style(), msg.trim_start_matches("error: ").trim_end_matches("\n")));
        ExitCode::FAILURE
      }
      DshCliExit::ErrClap(clap_error) => {
        let _ = clap_error.print();
        ExitCode::FAILURE
      }
      DshCliExit::ErrContext(msg, context) => {
        context.print_error(msg);
        if context.suppress_exit_status {
          context.print_warning("exit status suppressed");
          ExitCode::SUCCESS
        } else {
          ExitCode::FAILURE
        }
      }
    }
  }
}

#[tokio::main]
async fn main() -> DshCliExit {
  inner_main().await
}

async fn inner_main() -> DshCliExit {
  let _ = ctrlc::set_handler(move || {
    eprintln!("{}", wrap_style(default_warning_style(), "interrupted"));
    process::exit(0);
  });

  let subjects: Vec<&(dyn Subject + Send + Sync)> = vec![
    API_SUBJECT.as_ref(),
    APP_SUBJECT.as_ref(),
    BUCKET_SUBJECT.as_ref(),
    CERTIFICATE_SUBJECT.as_ref(),
    ENV_SUBJECT.as_ref(),
    IMAGE_SUBJECT.as_ref(),
    MANIFEST_SUBJECT.as_ref(),
    METRIC_SUBJECT.as_ref(),
    PLATFORM_SUBJECT.as_ref(),
    PROXY_SUBJECT.as_ref(),
    SECRET_SUBJECT.as_ref(),
    SERVICE_SUBJECT.as_ref(),
    #[cfg(feature = "manage")]
    TENANT_SUBJECT.as_ref(),
    TOKEN_SUBJECT.as_ref(),
    TOPIC_SUBJECT.as_ref(),
    VHOST_SUBJECT.as_ref(),
    VOLUME_SUBJECT.as_ref(),
    SETTING_SUBJECT.as_ref(),
    TARGET_SUBJECT.as_ref(),
  ];

  let mut subject_registry: HashMap<String, &(dyn Subject + Send + Sync)> = HashMap::new();
  let mut subject_list_shortcut_registry: HashMap<String, &(dyn Subject + Send + Sync)> = HashMap::new();

  let mut subject_commands: Vec<Command> = Vec::new();

  for subject in subjects {
    let (command_name, subject_command) = subject.subject_command();
    subject_registry.insert(command_name.to_string(), subject);
    subject_commands.push(subject_command);
    if let Some((list_shortcut_name, clap_list_command_shortcut)) = subject.subject_list_shortcut_command() {
      subject_list_shortcut_registry.insert(list_shortcut_name.to_string(), subject);
      subject_commands.push(clap_list_command_shortcut);
    }
  }

  let (settings, settings_log) = match get_settings(None) {
    Ok((setting, settings_log)) => (setting, settings_log),
    Err(msg) => return DshCliExit::Err(msg),
  };

  let mut command = create_command(&subject_commands, &settings);

  let matches = match command.clone().try_get_matches() {
    Ok(matches) => matches,
    Err(clap_error) => match clap_error.kind() {
      ErrorKind::DisplayHelp => return DshCliExit::OkClap(clap_error),
      ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => return DshCliExit::ErrClap(clap_error),
      _ => return DshCliExit::Err(clap_error.to_string()),
    },
  };

  if let Some(shell) = matches.get_one::<AutocompleteShell>(AUTOCOMPLETE_ARGUMENT) {
    generate_autocomplete_file(&mut command, shell);
    return DshCliExit::Ok;
  }

  if matches.get_flag(VERSION_ARGUMENT) {
    println!(
      "version: {}\ndsh-api library version: {}\ndsh openapi version: {}",
      VERSION,
      crate_version(),
      openapi_version()
    );
    return DshCliExit::Ok;
  }

  match initialize_logger(&matches, &settings) {
    Ok(_) => debug!("{}", settings_log),
    Err(msg) => return DshCliExit::Err(msg),
  }

  let context = match Context::create(&matches, settings) {
    Ok(context) => {
      trace!("{:#?}", context);
      context
    }
    Err(msg) => return DshCliExit::Err(msg),
  };

  match matches.subcommand() {
    Some((subject_command_name, sub_matches)) => match subject_registry.get(subject_command_name) {
      Some(subject) => {
        let requirements = subject.requirements(sub_matches);
        debug!("{:?}", requirements);
        if requirements.needs_dsh_api_client() {
          let client = match create_client(&matches, &context.settings).await {
            Ok(client) => client,
            Err(error) => return DshCliExit::ErrContext(error, Box::new(context)),
          };
          match subject.execute_subject_command_with_client(sub_matches, &client, &context).await {
            Ok(_) => {}
            Err(msg) => {
              return DshCliExit::ErrContext(msg, Box::new(context));
            }
          }
        } else {
          match subject.execute_subject_command_without_client(sub_matches, &context).await {
            Ok(_) => {}
            Err(msg) => {
              return DshCliExit::ErrContext(msg, Box::new(context));
            }
          }
        }
      }
      None => match subject_list_shortcut_registry.get(subject_command_name) {
        Some(subject_list_shortcut) => {
          let requirements = subject_list_shortcut.requirements_list_shortcut(sub_matches);
          debug!("{:?}", requirements);
          if requirements.needs_dsh_api_client() {
            let client = match create_client(&matches, &context.settings).await {
              Ok(client) => client,
              Err(error) => return DshCliExit::ErrContext(error, Box::new(context)),
            };
            match subject_list_shortcut
              .execute_subject_list_shortcut_with_client(sub_matches, &client, &context)
              .await
            {
              Ok(_) => {}
              Err(msg) => {
                return DshCliExit::ErrContext(msg, Box::new(context));
              }
            }
          } else {
            match subject_list_shortcut.execute_subject_list_shortcut_without_client(sub_matches, &context).await {
              Ok(_) => {}
              Err(msg) => {
                return DshCliExit::ErrContext(msg, Box::new(context));
              }
            }
          }
        }
        None => return DshCliExit::Err("unexpected error, list shortcut not found".to_string()),
      },
    },
    None => return DshCliExit::Err("unexpected error, no command provided".to_string()),
  }
  DshCliExit::Ok
}

fn create_command(clap_commands: &Vec<Command>, settings: &Settings) -> Command {
  let long_about = match enabled_features() {
    Some(enabled_features) => format!("{} Enabled features: {}.", LONG_ABOUT, enabled_features.join(", ")),
    None => LONG_ABOUT.to_string(),
  };
  let mut command = Command::new(APPLICATION_NAME)
    .about(ABOUT)
    .author(AUTHOR)
    .long_about(long_about)
    .disable_help_subcommand(true)
    .args(vec![
      target_platform_argument(),
      target_tenant_argument(),
      target_password_file_argument(),
      dry_run_argument(),
      force_argument(),
      log_level_argument(),
      log_level_api_argument(),
      log_level_sdk_argument(),
      no_escape_argument(),
      no_headers_argument(),
      output_format_argument(),
      quiet_argument(),
      set_verbosity_argument(),
      show_execution_time_argument(),
      suppress_exit_status_argument(),
      terminal_width_argument(),
      generate_autocomplete_file_argument(),
      version_argument(),
    ])
    .subcommand_value_name("SUBJECT/COMMAND")
    .subcommand_help_heading("Subjects/commands")
    .arg_required_else_help(true)
    .max_term_width(120)
    .hide_possible_values(false)
    .styles(STYLES.clone())
    .subcommands(clap_commands)
    .disable_version_flag(true);
  let mut default_settings: Vec<(&str, String)> = vec![];
  if let Some(default_platform) = &settings.default_platform {
    let platform = DshPlatform::try_from(default_platform.as_str()).unwrap();
    default_settings.push(("default platform", format!("{} / {}", platform.name(), platform.alias())));
  }
  if let Some(default_tenant) = &settings.default_tenant {
    default_settings.push(("default tenant", default_tenant.to_string()));
  }
  if let Some(ref file_name) = settings.file_name {
    default_settings.push(("settings file", file_name.to_string()));
  }
  if let Some(dry_run) = settings.dry_run {
    default_settings.push(("dry run mode", if dry_run { "enabled".to_string() } else { "disabled".to_string() }));
  }

  let mut environment_variables: Vec<(&str, String)> = vec![];
  let env_vars = get_environment_variables();
  if !env_vars.is_empty() {
    for (env_var, value) in &env_vars {
      if env_var == ENV_VAR_PASSWORD {
        environment_variables.push((env_var, "********".to_string()));
      } else {
        environment_variables.push((env_var, value.to_string()));
      }
    }
  }

  if default_settings.is_empty() {
    if environment_variables.is_empty() {
      command = command.after_long_help(AFTER_HELP);
    } else {
      let environment_variables_table = to_table("Environment variables:", environment_variables);
      command = command.after_help(&environment_variables_table);
      command = command.after_long_help(format!("{}\n\n{}", environment_variables_table, AFTER_HELP));
    }
  } else {
    let settings_table = to_table("Settings:", default_settings);
    if environment_variables.is_empty() {
      command = command.after_help(&settings_table);
      command = command.after_long_help(format!("{}\n\n{}", settings_table, AFTER_HELP));
    } else {
      let environment_variables_table = to_table("Environment variables:", environment_variables);
      command = command.after_help(format!("{}\n\n{}", settings_table, environment_variables_table));
      command = command.after_long_help(format!("{}\n\n{}\n\n{}", settings_table, environment_variables_table, AFTER_HELP));
    }
  }
  command
}

pub(crate) fn read_single_line(prompt: impl AsRef<str>) -> Result<String, String> {
  print!("{}", prompt.as_ref());
  let _ = stdout().lock().flush();
  let mut line = String::new();
  stdin().read_line(&mut line).expect("could not read line");
  Ok(line.trim().to_string())
}

pub(crate) fn read_single_line_password(prompt: impl AsRef<str>) -> Result<String, String> {
  match prompt_password(prompt.as_ref()) {
    Ok(line) => Ok(line.trim().to_string()),
    Err(_) => Err("empty input".to_string()),
  }
}

pub(crate) fn _include_app_service(matches: &ArgMatches) -> (bool, bool) {
  match (matches.get_flag(FilterFlagType::App.id()), matches.get_flag(FilterFlagType::Service.id())) {
    (false, false) => (true, true),
    (false, true) => (false, true),
    (true, false) => (true, false),
    (true, true) => (true, true),
  }
}

pub(crate) fn include_started_stopped(matches: &ArgMatches) -> (bool, bool) {
  match (matches.get_flag(FilterFlagType::Started.id()), matches.get_flag(FilterFlagType::Stopped.id())) {
    (false, false) => (true, true),
    (false, true) => (false, true),
    (true, false) => (true, false),
    (true, true) => (true, true),
  }
}

pub(crate) fn get_environment_variables() -> Vec<(String, String)> {
  let mut environment_variables: Vec<(String, String)> = vec![];
  for (env_var, value) in env::vars() {
    if env_var.starts_with(ENV_VAR_PREFIX) {
      environment_variables.push((env_var, value));
    }
  }
  if let Ok(platforms_file) = env::var(ENV_VAR_PLATFORMS_FILE_NAME) {
    environment_variables.push((ENV_VAR_PLATFORMS_FILE_NAME.to_string(), platforms_file));
  }
  if env::var(ENV_VAR_NO_COLOR).is_ok() {
    environment_variables.push((ENV_VAR_NO_COLOR.to_string(), "set".to_string()));
  }
  environment_variables.sort_by(|(env_var_a, _), (env_var_b, _)| env_var_a.cmp(env_var_b));
  environment_variables
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
/// * `settings` - contents of the settings file or default settings
///
/// ## Returns
/// `Ok(Some<Platform>)` - target platforms
/// `Ok(None)` - when no implicit platform is available
/// `Err<String>` - when invalid platform name was found
fn get_target_platform_implicit(settings: &Settings) -> Result<Option<DshPlatform>, String> {
  match env::var(ENV_VAR_PLATFORM) {
    Ok(platform_name_from_env_var) => {
      debug!("target platform '{}' (environment variable '{}')", platform_name_from_env_var, ENV_VAR_PASSWORD);
      DshPlatform::try_from(platform_name_from_env_var.as_str()).map(Some)
    }
    Err(_) => match settings.default_platform.clone() {
      Some(default_platform_name_from_settings) => {
        debug!("default target platform '{}' (settings)", default_platform_name_from_settings);
        DshPlatform::try_from(default_platform_name_from_settings.as_str()).map(Some)
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
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// `Ok(Option<Platform>)` - containing the platforms
/// `Ok(None)` - when no implicit target platform is available
/// `Err<String>` - when an invalid platform name was found
fn get_target_platform_non_interactive(matches: &ArgMatches, settings: &Settings) -> Result<Option<DshPlatform>, String> {
  match matches.get_one::<String>(TARGET_PLATFORM_ARGUMENT) {
    Some(target_platform_name_from_argument) => {
      debug!("target platform '{}' (argument)", target_platform_name_from_argument);
      DshPlatform::try_from(target_platform_name_from_argument.as_str()).map(Some)
    }
    None => get_target_platform_implicit(settings),
  }
}

/// # Get the target platform
///
/// This method will get the target platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--platform`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Parameter `default-platform` from settings file.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// `Ok<Platform>`  - target platform
/// `Err<String>` - error message
fn get_target_platform(matches: &ArgMatches, settings: &Settings) -> Result<DshPlatform, String> {
  match get_target_platform_non_interactive(matches, settings)? {
    Some(platforms_non_interactive) => Ok(platforms_non_interactive),
    None => {
      if stdin().is_terminal() {
        DshPlatform::try_from(read_single_line("target platform: ")?.as_str())
      } else {
        Err("could not determine target platform, please check configuration".to_string())
      }
    }
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
/// * `settings` - contents of the settings file or default settings
///
/// ## Returns
/// `Some<String>` - containing the tenant name
/// `None` - when no implicit tenant name is available
fn get_target_tenant_implicit(settings: &Settings) -> Option<String> {
  match env::var(ENV_VAR_TENANT) {
    Ok(tenant_name_from_env_var) => {
      debug!("target tenant '{}' (environment variable '{}')", tenant_name_from_env_var, ENV_VAR_TENANT);
      Some(tenant_name_from_env_var)
    }
    Err(_) => match settings.default_tenant.clone() {
      Some(default_tenant_name_from_settings) => {
        debug!("default target tenant '{}' (settings)", default_tenant_name_from_settings);
        Some(default_tenant_name_from_settings)
      }
      None => None,
    },
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
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// `Some<String>` - tenant name
/// `None` - when no tenant name is available without asking the user
fn get_target_tenant_non_interactive(matches: &ArgMatches, settings: &Settings) -> Option<String> {
  match matches.get_one::<String>(TARGET_TENANT_ARGUMENT) {
    Some(target_tenant_name_from_argument) => {
      debug!("target tenant '{}' (argument)", target_tenant_name_from_argument);
      Some(target_tenant_name_from_argument.clone())
    }
    None => get_target_tenant_implicit(settings),
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
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// An `Ok<String>` tenant name
/// An `Err<String>` error message
fn get_target_tenant(matches: &ArgMatches, settings: &Settings) -> Result<String, String> {
  match get_target_tenant_non_interactive(matches, settings) {
    Some(tenant_names_non_interactive) => Ok(tenant_names_non_interactive),
    None => {
      if stdin().is_terminal() {
        let tenant_name_from_console = read_single_line("target tenant: ")?;
        if tenant_name_from_console.is_empty() {
          Err("target tenant name cannot be empty".to_string())
        } else {
          Ok(tenant_name_from_console)
        }
      } else {
        Err("could not determine target tenant, please check configuration".to_string())
      }
    }
  }
}

/// # Get the target password
///
/// This method will get the target password.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--password-file`, which should reference a file that
///    contains the password.
/// 1. Environment variable `DSH_CLI_PASSWORD_FILE`.
/// 1. Environment variable `DSH_CLI_PASSWORD`.
/// 1. If target file `[platform].[tenant_name].toml` exists,
///    check entry `dsh.[platform].[tenant_name]` from the keychain, if available.
///    This can result in a pop-up where the user must authenticate for the keychain.
/// 1. If stdin is a terminal, ask the user to enter the password.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `dsh_api_tenant` - used to determine the target settings file
///
/// ## Returns
/// An `Ok<String>` containing the password, or an `Err<String>`.
fn get_target_password(matches: &ArgMatches, dsh_api_tenant: &DshApiTenant) -> Result<String, String> {
  match matches.get_one::<PathBuf>(TARGET_PASSWORD_FILE_ARGUMENT) {
    Some(password_file_from_arg) => read_target_password_file(password_file_from_arg),
    None => match env::var(ENV_VAR_PASSWORD_FILE) {
      Ok(password_file_from_env) => read_target_password_file(password_file_from_env),
      Err(_) => match env::var(ENV_VAR_PASSWORD) {
        Ok(password_from_env_var) => {
          debug!("target password (environment variable '{}')", ENV_VAR_PASSWORD);
          Ok(password_from_env_var)
        }
        Err(_) => match (
          read_target(dsh_api_tenant.platform(), dsh_api_tenant.name())?,
          get_target_password_from_keyring(dsh_api_tenant.platform(), dsh_api_tenant.name())?,
        ) {
          (Some(_), Some(password_from_keyring)) => {
            debug!("target exists, password read (keyring)");
            Ok(password_from_keyring)
          }
          _ => {
            if stdin().is_terminal() {
              read_single_line_password(format!("password for tenant {}: ", dsh_api_tenant).as_str())
            } else {
              Err("could not determine password and unable to to prompt user, please check configuration".to_string())
            }
          }
        },
      },
    },
  }
}

fn read_target_password_file<T: AsRef<Path>>(password_file: T) -> Result<String, String> {
  match fs::read_to_string(&password_file) {
    Ok(password_string) => {
      let trimmed_password = password_string.trim();
      if trimmed_password.is_empty() {
        Err(format!("target password file '{}' is empty", password_file.as_ref().to_string_lossy()))
      } else {
        debug!("target password (file '{}')", password_file.as_ref().to_string_lossy());
        Ok(trimmed_password.to_string())
      }
    }
    Err(_) => Err(format!("target password file '{}' could not be read", password_file.as_ref().to_string_lossy())),
  }
}

/// # Returns the `dsh` tool directory
///
/// This function returns the `dsh` tool's directory.
/// If it doesn't already exist the directory (and possibly its parent directories)
/// will be created.
///
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. If environment variable `DSH_CLI_HOME` exists:
///    * If it is empty, return `Ok(None)`
///    * Else return the specified directory.
/// 1. If environment variable `HOME` exists, return its value concatenated with `/.dsh_cli`.
fn dsh_directory() -> Result<Option<PathBuf>, String> {
  let dsh_directory: Option<PathBuf> = match env::var(ENV_VAR_HOME_DIRECTORY) {
    Ok(dsh_directory_from_env_var) => {
      if dsh_directory_from_env_var.is_empty() {
        None
      } else {
        Some(PathBuf::new().join(dsh_directory_from_env_var))
      }
    }
    Err(_) => match my_home() {
      Ok(Some(user_home_directory)) => Some(user_home_directory.join(DEFAULT_USER_DSH_CLI_DIRECTORY)),
      _ => {
        let message = format!("could not determine dsh cli directory name (check environment variable {})", ENV_VAR_HOME_DIRECTORY);
        log::error!("{}", &message);
        return Err(message);
      }
    },
  };
  match dsh_directory {
    Some(directory) => match fs::create_dir_all(&directory) {
      Ok(_) => match fs::create_dir_all(directory.join(TARGETS_SUBDIRECTORY)) {
        Ok(_) => Ok(Some(directory)),
        Err(io_error) => {
          let message = format!(
            "could not create dsh targets directory '{}' ({})",
            directory.join(TARGETS_SUBDIRECTORY).to_string_lossy(),
            io_error
          );
          log::error!("{}", &message);
          Err(message)
        }
      },
      Err(io_error) => {
        let message = format!("could not create dsh directory '{}' ({})", directory.to_string_lossy(), io_error);
        log::error!("{}", &message);
        Err(message)
      }
    },
    None => Ok(None),
  }
}

fn read_and_deserialize_from_toml_file<T>(toml_file: impl AsRef<Path>) -> Result<Option<T>, String>
where
  T: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(&toml_file) {
    Ok(toml_string) => match toml::from_str::<T>(&toml_string) {
      Ok(deserialized_toml) => Ok(Some(deserialized_toml)),
      Err(de_error) => {
        let message = format!("could not deserialize file '{}' ({})", toml_file.as_ref().to_string_lossy(), de_error.message());
        log::error!("{}", &message);
        Err(message)
      }
    },
    Err(io_error) => match io_error.kind() {
      NotFound => Ok(None),
      _ => {
        let message = format!("could not read file '{}'", toml_file.as_ref().to_string_lossy());
        log::error!("{}", &message);
        Err(message)
      }
    },
  }
}

fn serialize_and_write_to_toml_file<T>(toml_file: impl AsRef<Path>, data: &T) -> Result<(), String>
where
  T: Serialize,
{
  match toml::to_string(data) {
    Ok(toml_string) => match fs::write(&toml_file, toml_string) {
      Ok(_) => Ok(()),
      Err(io_error) => {
        let message = format!("could not write file '{}' ({})", toml_file.as_ref().to_string_lossy(), io_error);
        log::error!("{}", &message);
        Err(message)
      }
    },
    Err(ser_error) => {
      let message = format!("could not serialize data ({})", ser_error);
      log::error!("{}", &message);
      Err(message)
    }
  }
}

/// Manually edit a configuration file
///
/// Will serialize the provided `configuration` to a temporary file
/// and open that file in the default system editor.
/// When the editor closes, the temporary file will be serialized again and returned.
async fn edit_configuration<C>(configuration: &C, temporary_configuration_file_name: &str) -> Result<Option<C>, String>
where
  C: for<'de> Deserialize<'de> + Serialize,
{
  match env::var("EDITOR") {
    Ok(editor_from_env_var) => {
      let editor = editor_from_env_var.split(" ").collect::<Vec<_>>();
      let editor_command = editor.first().ok_or("".to_string())?;
      let editor_args = editor.iter().skip(1).collect::<Vec<_>>();
      debug!("editor: {} {:?}", editor_command, editor_args);
      let mut temporary_configuration_file_path = temp_dir();
      temporary_configuration_file_path.push(temporary_configuration_file_name);
      debug!("temporary configuration file: {}", temporary_configuration_file_path.to_string_lossy());
      let original_configuration = serde_json::to_string_pretty::<C>(configuration).unwrap();
      tokio::fs::write(&temporary_configuration_file_path, &original_configuration)
        .await
        .map_err(|error| format!("cannot write temporary configuration file ({})", error))?;
      std::process::Command::new(editor_command)
        .args(editor_args)
        .arg(&temporary_configuration_file_path)
        .status()
        .map_err(|error| format!("couldn't edit temporary configuration file ({})", error))?;
      let updated_configuration = tokio::fs::read_to_string(&temporary_configuration_file_path)
        .await
        .map_err(|error| format!("couldn't read temporary configuration file ({})", error))?;
      if original_configuration == updated_configuration {
        Ok(None)
      } else {
        Ok(Some(
          serde_json::from_str::<C>(&updated_configuration).map_err(|error| format!("could not parse temporary configuration file ({})", error))?,
        ))
      }
    }
    Err(_) => Err("environment variable 'EDITOR' is not set".to_string()),
  }
}

async fn create_client(matches: &ArgMatches, settings: &Settings) -> Result<DshApiClient, String> {
  let target_platform = get_target_platform(matches, settings)?;
  let target_tenant_name = get_target_tenant(matches, settings)?;
  debug!("create client for target '{}@{}'", target_tenant_name, target_platform);
  let dsh_api_tenant = DshApiTenant::new(target_tenant_name.clone(), target_platform.clone());
  let password = get_target_password(matches, &dsh_api_tenant)?;
  let dsh_api_client_factory = DshApiClientFactory::create(dsh_api_tenant, password)?;
  let dsh_api_client = dsh_api_client_factory.client().await?;
  debug!("api client created");
  Ok(dsh_api_client)
}

// Method will panic if rows vector is empty
fn to_table(header: &str, rows: Vec<(&str, String)>) -> String {
  let bold_green = Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green)));
  let bold_blue = Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Blue)));
  let key_value_length_pairs: Vec<(&str, &str, usize)> = rows.iter().map(|(key, value)| (*key, value.as_ref(), key.len())).collect::<Vec<_>>();
  let first_column_width = &key_value_length_pairs.iter().map(|(_, _, len)| len).max().unwrap().clone();
  format!(
    "{bold_green}{}{bold_green:#}\n{}",
    header,
    key_value_length_pairs
      .into_iter()
      .map(|(key, value, len)| format!("  {bold_blue}{}{bold_blue:#}{}  {}", key, " ".repeat(first_column_width - len), value))
      .collect::<Vec<_>>()
      .join("\n")
  )
}

fn enabled_features() -> Option<Vec<&'static str>> {
  #[allow(unused_mut)]
  let mut enabled_features = vec![];
  #[cfg(feature = "manage")]
  enabled_features.push("manage");
  #[cfg(feature = "robot")]
  enabled_features.push("robot");
  if enabled_features.is_empty() {
    None
  } else {
    Some(enabled_features)
  }
}

fn default_error_style() -> Style {
  style_from(&DshStyle::Bold, &DshColor::Red)
}

fn default_warning_style() -> Style {
  style_from(&DshStyle::Bold, &DshColor::Blue)
}

#[test]
fn test_open_api_version() {
  assert_eq!(openapi_version(), "1.9.0");
}

#[test]
fn test_dsh_api_version() {
  assert_eq!(crate_version(), "0.7.0");
}
