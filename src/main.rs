#![doc(
  html_favicon_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
#![doc(
  html_logo_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
extern crate core;

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::ErrorKind::NotFound;
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Termination};
use std::{env, fs, process};

use crate::autocomplete::{generate_autocomplete_file, generate_autocomplete_file_argument, AutocompleteShell, AUTOCOMPLETE_ARGUMENT};
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::global_arguments::{
  dry_run_argument, force_argument, matching_style_argument, no_escape_argument, no_headers_argument, output_format_argument, quiet_argument, set_verbosity_argument,
  show_execution_time_argument, target_password_file_argument, target_platform_argument, target_tenant_argument, terminal_width_argument, TARGET_PASSWORD_FILE_ARGUMENT,
  TARGET_PLATFORM_ARGUMENT, TARGET_TENANT_ARGUMENT,
};
use crate::log_arguments::{log_level_api_argument, log_level_argument, log_level_sdk_argument};
use crate::log_level::initialize_logger;
use crate::settings::{get_settings, Settings};
use crate::subject::Subject;
use crate::subjects::api::API_SUBJECT;
use crate::subjects::application::APPLICATION_SUBJECT;
use crate::subjects::platform::PLATFORM_SUBJECT;
use crate::subjects::token::TOKEN_SUBJECT;
use crate::targets::{get_target_password_from_keyring, read_target};
use clap::builder::{styling, Styles};
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::platform::DshPlatform;
use dsh_api::{crate_version, openapi_version};
use homedir::my_home;
use lazy_static::lazy_static;
use log::debug;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use subjects::app::APP_SUBJECT;
use subjects::bucket::BUCKET_SUBJECT;
use subjects::certificate::CERTIFICATE_SUBJECT;
use subjects::env::ENV_SUBJECT;
use subjects::image::IMAGE_SUBJECT;
#[cfg(feature = "appcatalog")]
use subjects::manifest::MANIFEST_SUBJECT;
use subjects::metric::METRIC_SUBJECT;
use subjects::proxy::PROXY_SUBJECT;
use subjects::secret::SECRET_SUBJECT;
use subjects::setting::SETTING_SUBJECT;
use subjects::target::TARGET_SUBJECT;
use subjects::topic::TOPIC_SUBJECT;
use subjects::vhost::VHOST_SUBJECT;
use subjects::volume::VOLUME_SUBJECT;

mod arguments;
mod autocomplete;
mod capability;
mod capability_builder;
mod context;
mod filter_flags;
mod flags;
mod formatters;
mod global_arguments;
mod log_arguments;
mod log_level;
mod modifier_flags;
mod settings;
mod subject;
mod subjects;
mod targets;
mod verbosity;

lazy_static! {
  static ref STYLES: Styles = Styles::styled()
    .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
    .placeholder(styling::AnsiColor::Cyan.on_default());
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
   such as finding all applications that use a certain resource (e.g. a secret) or showing a \
   list of all resources of a certain type (e.g. list all volumes).";
/// Will be shown after normal help text, when `-h` was provided
const AFTER_HELP: &str = "For most commands adding an 's' as a postfix will yield the same result \
   as using the 'list' subcommand, e.g. using 'dsh apps' will be the same \
   as using 'dsh app list'.";
const USAGE: &str = "dsh [OPTIONS] [SUBJECT/COMMAND]\n       dsh --help\n       dsh secret --help\n       dsh secret list --help";

const VERSION: &str = "0.5.0";

const ENV_VAR_PREFIX: &str = "DSH_CLI_";

// Duplicate from dsh_api crate
const ENV_VAR_PLATFORMS_FILE_NAME: &str = "DSH_API_PLATFORMS_FILE";

const ENV_VAR_CSV_QUOTE: &str = "DSH_CLI_CSV_QUOTE";
const ENV_VAR_CSV_SEPARATOR: &str = "DSH_CLI_CSV_SEPARATOR";
const ENV_VAR_DRY_RUN: &str = "DSH_CLI_DRY_RUN";
const ENV_VAR_HOME_DIRECTORY: &str = "DSH_CLI_HOME";
const ENV_VAR_LOG_LEVEL: &str = "DSH_CLI_LOG_LEVEL";
const ENV_VAR_LOG_LEVEL_API: &str = "DSH_CLI_LOG_LEVEL_API";
const ENV_VAR_LOG_LEVEL_SDK: &str = "DSH_CLI_LOG_LEVEL_SDK";
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
const ENV_VAR_TENANT: &str = "DSH_CLI_TENANT";
const ENV_VAR_TERMINAL_WIDTH: &str = "DSH_CLI_TERMINAL_WIDTH";
const ENV_VAR_VERBOSITY: &str = "DSH_CLI_VERBOSITY";

const DEFAULT_USER_DSH_CLI_DIRECTORY: &str = ".dsh_cli";
const TARGETS_SUBDIRECTORY: &str = "targets";
const DEFAULT_DSH_CLI_SETTINGS_FILENAME: &str = "settings.toml";
const TOML_FILENAME_EXTENSION: &str = "toml";

type DshCliResult = Result<(), String>;

#[derive(Debug)]
enum DshCliExit {
  Success,
  Error(String),
}

impl Termination for DshCliExit {
  fn report(self) -> ExitCode {
    match self {
      DshCliExit::Success => ExitCode::SUCCESS,
      DshCliExit::Error(msg) => {
        eprintln!("{}", msg);
        ExitCode::FAILURE
      }
    }
  }
}

#[tokio::main]
async fn main() -> DshCliExit {
  match inner_main().await {
    Ok(_) => DshCliExit::Success,
    Err(msg) => DshCliExit::Error(msg),
  }
}

async fn inner_main() -> DshCliResult {
  let _ = ctrlc::set_handler(move || {
    eprintln!("interrupted");
    process::exit(0);
  });

  let subjects: Vec<&(dyn Subject + Send + Sync)> = vec![
    API_SUBJECT.as_ref(),
    APP_SUBJECT.as_ref(),
    APPLICATION_SUBJECT.as_ref(),
    BUCKET_SUBJECT.as_ref(),
    CERTIFICATE_SUBJECT.as_ref(),
    ENV_SUBJECT.as_ref(),
    IMAGE_SUBJECT.as_ref(),
    #[cfg(feature = "appcatalog")]
    MANIFEST_SUBJECT.as_ref(),
    METRIC_SUBJECT.as_ref(),
    PLATFORM_SUBJECT.as_ref(),
    PROXY_SUBJECT.as_ref(),
    SECRET_SUBJECT.as_ref(),
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

  let mut command = create_command(&subject_commands);

  let matches = command.clone().get_matches();

  if let Some(shell) = matches.get_one::<AutocompleteShell>(AUTOCOMPLETE_ARGUMENT) {
    generate_autocomplete_file(&mut command, shell);
    return Ok(());
  }

  let (settings, settings_log) = get_settings(None)?;

  initialize_logger(&matches, &settings)?;

  debug!("{}", settings_log);

  match matches.subcommand() {
    Some((subject_command_name, sub_matches)) => match subject_registry.get(subject_command_name) {
      Some(subject) => {
        let requirements = subject.requirements(sub_matches);
        let context = Context::create(&matches, &requirements, &settings).await?;
        subject.execute_subject_command(sub_matches, &context).await?;
      }
      None => match subject_list_shortcut_registry.get(subject_command_name) {
        Some(subject_list_shortcut) => {
          let requirements = subject_list_shortcut.requirements(sub_matches);
          let context = Context::create(&matches, &requirements, &settings).await?;
          subject_list_shortcut.execute_subject_list_shortcut(sub_matches, &context).await?;
        }
        None => return Err("unexpected error, list shortcut not found".to_string()),
      },
    },
    None => return Err("unexpected error, no command provided".to_string()),
  };
  Ok(())
}

fn create_command(clap_commands: &Vec<Command>) -> Command {
  let long_about = match enabled_features() {
    Some(enabled_features) => format!("{} Enabled features: {}.", LONG_ABOUT, enabled_features.join(", ")),
    None => LONG_ABOUT.to_string(),
  };
  Command::new(APPLICATION_NAME)
    .about(ABOUT)
    .author(AUTHOR)
    .long_about(long_about)
    .override_usage(USAGE)
    .after_help(AFTER_HELP)
    .args(vec![
      target_platform_argument(),
      target_tenant_argument(),
      target_password_file_argument(),
      output_format_argument(), // TODO Should this one be at this level?
      set_verbosity_argument(),
      dry_run_argument(),
      force_argument(),
      matching_style_argument(), // TODO Should this one be at this level?
      no_escape_argument(),      // TODO Should this one be at this level?
      no_headers_argument(),     // TODO Should this one be at this level?
      quiet_argument(),
      log_level_argument(),
      log_level_api_argument(),
      log_level_sdk_argument(),
      show_execution_time_argument(), // TODO Should this one be at this level?
      terminal_width_argument(),      // TODO Should this one be at this level?
      generate_autocomplete_file_argument(),
    ])
    .subcommand_value_name("SUBJECT/COMMAND")
    .subcommand_help_heading("Subjects/commands")
    .arg_required_else_help(true)
    .max_term_width(120)
    .hide_possible_values(false)
    .styles(STYLES.clone())
    .subcommands(clap_commands)
    .version(VERSION)
    .long_version(format!(
      "version: {}\ndsh-api library version: {}\ndsh openapi version: {}",
      VERSION,
      crate_version(),
      openapi_version()
    ))
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

pub(crate) fn _include_app_application(matches: &ArgMatches) -> (bool, bool) {
  match (matches.get_flag(FilterFlagType::App.id()), matches.get_flag(FilterFlagType::Application.id())) {
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
///
/// ## Parameters
/// * `settings` - contents of the settings file or default settings
///
/// ## Returns
/// `Ok(Option<Platform>)` - containing the [`DshPlatform`]
/// `Ok(None)` - when no implicit source is available
/// `Err<String>` - when an invalid platform name was found
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

/// # Get the target platform
///
/// This method will get the target platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--platform`.
/// 1. Environment variable `DSH_CLI_PLATFORM`.
/// 1. Parameter `default-platform` from settings file, if available.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// An `Ok<Platform>` containing the [`DshPlatform`], or an `Err<String>`.
fn get_target_platform(matches: &ArgMatches, settings: &Settings) -> Result<DshPlatform, String> {
  match matches.get_one::<String>(TARGET_PLATFORM_ARGUMENT) {
    Some(platform_from_argument) => {
      debug!("target platform '{}' (argument)", platform_from_argument);
      Ok(DshPlatform::try_from(platform_from_argument.as_str()).unwrap())
    }
    None => match get_target_platform_implicit(settings)? {
      Some(platform_name_from_implicit_source) => Ok(platform_name_from_implicit_source),
      None => {
        if stdin().is_terminal() {
          DshPlatform::try_from(read_single_line("target platform: ")?.as_str())
        } else {
          Err("could not determine target platform, please check configuration".to_string())
        }
      }
    },
  }
}

/// # Get the target tenant from implicit sources
///
/// This method will get try to find the target tenant from the implicit sources listed below,
/// and returns at the first match.
/// 1. Environment variable `DSH_CLI_TENANT`.
/// 1. Parameter `default-tenant` from settings file, if available.
///
/// ## Parameters
/// * `settings` - contents of the settings file or default settings
///
/// ## Returns
/// `Some<String>` - containing the tenant name
/// `None` - when no implicit source is available
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
/// An `Ok<String>` containing the tenant name, or an `Err<String>`.
fn get_target_tenant(matches: &ArgMatches, settings: &Settings) -> Result<String, String> {
  match matches.get_one::<String>(TARGET_TENANT_ARGUMENT) {
    Some(tenant_name_from_argument) => {
      debug!("target tenant '{}' (argument)", tenant_name_from_argument);
      Ok(tenant_name_from_argument.to_string())
    }
    None => match get_target_tenant_implicit(settings) {
      Some(tenant_name_from_implicit_source) => Ok(tenant_name_from_implicit_source),
      None => {
        if stdin().is_terminal() {
          let tenant_name = read_single_line("target tenant: ")?;
          if tenant_name.is_empty() {
            Err("target tenant name cannot be empty".to_string())
          } else {
            Ok(tenant_name)
          }
        } else {
          Err("could not determine target tenant, please check configuration".to_string())
        }
      }
    },
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

/// # Returns the dsh application directory
///
/// This function returns the application directory.
/// If it doesn't already exist the directory (and possibly its parent directories)
/// will be created.
///
/// To determine the directory, first the environment variable DSH_CLI_HOME will be checked.
/// If this variable is not defined, `${HOME}/.dsh_cli` will be used as the application directory.
fn dsh_directory() -> Result<PathBuf, String> {
  let dsh_directory = match env::var(ENV_VAR_HOME_DIRECTORY) {
    Ok(dsh_directory) => PathBuf::new().join(dsh_directory),
    Err(_) => match my_home() {
      Ok(Some(user_home_directory)) => user_home_directory.join(DEFAULT_USER_DSH_CLI_DIRECTORY),
      _ => {
        let message = format!("could not determine dsh cli directory name (check environment variable {})", ENV_VAR_HOME_DIRECTORY);
        log::error!("{}", &message);
        return Err(message);
      }
    },
  };
  match fs::create_dir_all(&dsh_directory) {
    Ok(_) => match fs::create_dir_all(dsh_directory.join(TARGETS_SUBDIRECTORY)) {
      Ok(_) => Ok(dsh_directory),
      Err(io_error) => {
        let message = format!(
          "could not create dsh targets directory '{}' ({})",
          dsh_directory.join(TARGETS_SUBDIRECTORY).to_string_lossy(),
          io_error
        );
        log::error!("{}", &message);
        Err(message)
      }
    },
    Err(io_error) => {
      let message = format!("could not create dsh directory '{}' ({})", dsh_directory.to_string_lossy(), io_error);
      log::error!("{}", &message);
      Err(message)
    }
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

fn enabled_features() -> Option<Vec<&'static str>> {
  #[allow(unused_mut)]
  let mut enabled_features = vec![];
  #[cfg(feature = "appcatalog")]
  enabled_features.push("appcatalog");
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

#[test]
fn test_open_api_version() {
  assert_eq!(openapi_version(), "1.9.0");
}

#[test]
fn test_dsh_api_version() {
  assert_eq!(crate_version(), "0.5.0");
}
