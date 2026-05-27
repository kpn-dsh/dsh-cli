#![doc(
  html_favicon_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
#![doc(
  html_logo_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
extern crate core;

use crate::authentication::get_access_tokens;
use crate::clients::create_clients;
use crate::directory::{get_settings, init_dsh_directory};
use crate::environment_variables::{
  env_var_argument, env_var_file_argument, env_vars_argument, environment_variable, get_configured_environment_variables, print_environment_variable, print_environment_variables,
  ENV_VARS_ARGUMENT, ENV_VAR_ARGUMENT,
};
use crate::error::DshCliError;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::{Label, SubjectFormatter, Value};
use crate::global_options::{
  authentication_option, browser_option, environment_variable_option, log_level_api_option, log_level_option, no_csv_headers_flag, output_directory_option, password_file_option,
  platform_option, releases_flag, tenant_option, tenants_all_flag, tenants_option, RELEASES_FLAG, TENANTS_ALL_FLAG,
};
use crate::releases::{newer_release, newer_release_notification, releases_from_github, ReleaseAsset};
use crate::style::{apply_default_error_style, apply_default_warning_style};
use crate::subjects::aclgroup::ACL_GROUP_SUBJECT;
use crate::subjects::nodepool::NODE_POOL_SUBJECT;
use crate::subjects::task::TASK_SUBJECT;
use crate::target_platform::{get_target_platform, get_target_platform_from_all_sources, platform_name_argument};
use autocomplete::{generate_autocomplete_file, generate_autocomplete_file_argument, AutocompleteShell, AUTOCOMPLETE_ARGUMENT};
use clap::builder::styling::{AnsiColor, Color, Style};
use clap::builder::{styling, Styles};
use clap::error::{Error as ClapError, ErrorKind};
use clap::{ArgMatches, Command};
use context::Context;
use dsh_api::version::Version;
use dsh_api::{crate_version, openapi_version};
use filter_flags::FilterFlagType;
use global_options::{
  dry_run_flag, force_flag, no_escape_flag, output_format_option, quiet_flag, set_verbosity_option, show_execution_time_flag, suppress_exit_status_flag, terminal_width_option,
  version_flag, VERSION_FLAG,
};
use itertools::Itertools;
use log::{debug, error, trace};
use log_level::initialize_logger;
use serde::{Deserialize, Serialize};
use settings::Settings;
use std::collections::HashMap;
use std::env::temp_dir;
use std::error::Error;
use std::fmt::Debug;
use std::io::ErrorKind::NotFound;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{ExitCode, Termination};
use std::str::FromStr;
use std::sync::LazyLock;
use std::{fs, process};
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
use subjects::robot::ROBOT_SUBJECT;
use subjects::secret::SECRET_SUBJECT;
use subjects::service::SERVICE_SUBJECT;
use subjects::setting::SETTING_SUBJECT;
#[cfg(feature = "manage")]
use subjects::stream::STREAM_SUBJECT;
#[cfg(feature = "manage")]
use subjects::tenant::TENANT_SUBJECT;
use subjects::token::TOKEN_SUBJECT;
use subjects::topic::TOPIC_SUBJECT;
use subjects::vhost::VHOST_SUBJECT;
use subjects::volume::VOLUME_SUBJECT;

mod argument_parsers;
mod arguments;
mod authentication;
mod autocomplete;
mod capability;
mod capability_builder;
mod cipher;
mod clients;
mod code;
mod context;
mod directory;
mod environment_variables;
mod error;
mod filter_flags;
mod flags;
mod formatters;
mod global_options;
mod issues;
mod keyring;
#[cfg(feature = "manage")]
mod limits_options;
mod log_level;
mod modifier_flags;
mod monitoring;
mod proxy_bundles;
mod releases;
mod robot_arguments;
mod secret_metadata;
mod settings;
mod style;
mod subject;
mod subjects;
mod target_platform;
mod target_tenant;
mod verbosity;

static STYLES: LazyLock<Styles> = LazyLock::new(|| {
  Styles::styled()
    .header(AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | styling::Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default())
});

const APPLICATION_NAME: &str = "dsh";

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

static VERSION: LazyLock<Version> = LazyLock::new(|| Version::from_str("0.10.0").unwrap());

const COMMAND_OPTIONS_HEADING: &str = "Command options";
const OUTPUT_OPTIONS_HEADING: &str = "Output options";
const TOOL_OPTIONS_HEADING: &str = "Tool options";

type DshCliResult<T> = Result<T, DshCliError>;

#[derive(Debug)]
enum DshCliExit {
  CliErr(DshCliError),
  CliErrContext(DshCliError, Box<Context>),
  Err(String),
  ErrClap(ClapError),
  ErrContext(String, Box<Context>),
  ErrHelp(ClapError),
  Ok,
  OkClap(ClapError),
}

impl From<DshCliError> for DshCliExit {
  fn from(dsh_cli_error: DshCliError) -> Self {
    Self::Err(dsh_cli_error.into())
  }
}

impl Termination for DshCliExit {
  fn report(self) -> ExitCode {
    match self {
      DshCliExit::CliErr(dsh_cli_error) => {
        eprintln!(
          "{}",
          apply_default_error_style(dsh_cli_error.to_string().trim_start_matches("error: ").trim_end_matches("\n"))
        );
        ExitCode::FAILURE
      }
      DshCliExit::CliErrContext(dsh_cli_error, context) => {
        context.print_error(dsh_cli_error);
        if context.suppress_exit_status() {
          context.print_warning("exit status suppressed");
          ExitCode::SUCCESS
        } else {
          ExitCode::FAILURE
        }
      }
      DshCliExit::Err(msg) => {
        eprintln!("{}", apply_default_error_style(msg.trim_start_matches("error: ").trim_end_matches("\n")));
        ExitCode::FAILURE
      }
      DshCliExit::ErrClap(clap_error) => {
        match clap_error.source() {
          Some(source) => {
            eprintln!(
              "{}",
              apply_default_error_style(source.to_string().trim_start_matches("error: ").trim_end_matches("\n"))
            );
          }
          None => {
            eprintln!(
              "{}",
              apply_default_error_style(clap_error.to_string().trim_start_matches("error: ").trim_end_matches("\n"))
            );
          }
        }
        ExitCode::FAILURE
      }
      DshCliExit::ErrContext(msg, context) => {
        context.print_error(msg);
        if context.suppress_exit_status() {
          context.print_warning("exit status suppressed");
          ExitCode::SUCCESS
        } else {
          ExitCode::FAILURE
        }
      }
      DshCliExit::ErrHelp(clap_error) => {
        let _ = clap_error.print();
        ExitCode::FAILURE
      }
      DshCliExit::Ok => ExitCode::SUCCESS,
      DshCliExit::OkClap(clap_error) => {
        let _ = clap_error.print();
        ExitCode::SUCCESS
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
    eprintln!("{}", apply_default_warning_style("interrupted"));
    process::exit(0);
  });

  if let Err(error) = init_dsh_directory() {
    return DshCliExit::CliErr(error);
  }

  let subjects: Vec<&(dyn Subject + Send + Sync)> = vec![
    ACL_GROUP_SUBJECT.as_ref(),
    API_SUBJECT.as_ref(),
    APP_SUBJECT.as_ref(),
    BUCKET_SUBJECT.as_ref(),
    CERTIFICATE_SUBJECT.as_ref(),
    ENV_SUBJECT.as_ref(),
    IMAGE_SUBJECT.as_ref(),
    MANIFEST_SUBJECT.as_ref(),
    METRIC_SUBJECT.as_ref(),
    NODE_POOL_SUBJECT.as_ref(),
    PLATFORM_SUBJECT.as_ref(),
    PROXY_SUBJECT.as_ref(),
    ROBOT_SUBJECT.as_ref(),
    SECRET_SUBJECT.as_ref(),
    SERVICE_SUBJECT.as_ref(),
    #[cfg(feature = "manage")]
    STREAM_SUBJECT.as_ref(),
    TASK_SUBJECT.as_ref(),
    #[cfg(feature = "manage")]
    TENANT_SUBJECT.as_ref(),
    TOKEN_SUBJECT.as_ref(),
    TOPIC_SUBJECT.as_ref(),
    VHOST_SUBJECT.as_ref(),
    VOLUME_SUBJECT.as_ref(),
    SETTING_SUBJECT.as_ref(),
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

  let (settings, settings_log) = match get_settings() {
    Ok((setting, settings_log)) => (setting, settings_log),
    Err(error) => return DshCliExit::CliErr(error),
  };

  subject_commands.push(login_command());
  subject_commands.push(logout_command());

  subject_commands.sort_by(|subject_command_a, subject_command_b| subject_command_a.get_name().cmp(subject_command_b.get_name()));

  let mut command = create_command(&subject_commands, &settings).await;

  let matches = match command.clone().try_get_matches() {
    Ok(matches) => matches,
    Err(clap_error) => {
      return match clap_error.kind() {
        ErrorKind::DisplayHelp => DshCliExit::OkClap(clap_error),
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => DshCliExit::ErrHelp(clap_error),
        _ => DshCliExit::ErrClap(clap_error),
      }
    }
  };

  if let Some(shell) = matches.get_one::<AutocompleteShell>(AUTOCOMPLETE_ARGUMENT) {
    generate_autocomplete_file(&mut command, shell);
    return DshCliExit::Ok;
  }

  match initialize_logger(&matches, &settings) {
    Ok(_) => debug!("{}", settings_log),
    Err(error) => return DshCliExit::CliErr(error),
  }

  let context = match Context::create(&matches, settings) {
    Ok(context) => {
      trace!("{:#?}", context);
      context
    }
    Err(error) => return DshCliExit::CliErr(error),
  };

  if matches.get_flag(VERSION_FLAG) {
    match newer_release(&VERSION).await {
      Ok(Some(newer_release)) => {
        context.println(format!(
          "version: {}\ndsh-api library version: {}\ndsh openapi version: {}",
          *VERSION,
          crate_version(),
          openapi_version()
        ));
        context.print_warning(format!(
          "newer release {} available at {}",
          newer_release.version.map(|version| version.to_string()).unwrap_or("".to_string()),
          newer_release.html_url
        ));
      }
      Ok(None) => {
        context.println(format!(
          "version: {} (latest)\ndsh-api library version: {}\ndsh openapi version: {}",
          *VERSION,
          crate_version(),
          openapi_version()
        ));
      }
      Err(error) => return DshCliExit::CliErr(error),
    }
    return DshCliExit::Ok;
  }

  if matches.get_flag(RELEASES_FLAG) {
    match releases_from_github().await {
      Ok(mut releases) => {
        releases.reverse();
        context.print_explanation("list all releases");
        let mut formatter = ListFormatter::new(&RELEASE_LABELS, &context);
        for release in releases {
          for asset in release.assets {
            formatter.push_target_id_value_owned(release.name.clone(), asset.clone());
          }
        }
        _ = formatter.print(None);
        return DshCliExit::Ok;
      }
      Err(error) => return DshCliExit::CliErr(error),
    }
  }

  if let Some(newer_release) = newer_release_notification(&VERSION).await {
    context.print_warning(format!(
      "newer release {} available at {}",
      newer_release.version.map(|version| version.to_string()).unwrap_or("".to_string()),
      newer_release.html_url
    ));
  }

  if let Some(env_var_name) = matches.get_one::<String>(ENV_VAR_ARGUMENT) {
    print_environment_variable(env_var_name, &context);
    return DshCliExit::Ok;
  }

  if matches.get_flag(ENV_VARS_ARGUMENT) {
    print_environment_variables(&context);
    return DshCliExit::Ok;
  }

  match matches.subcommand() {
    Some(("login", arg_matches)) => {
      return match get_target_platform_from_all_sources(arg_matches, context.settings()) {
        Ok(platform) => match authentication::login(platform, context).await {
          Ok(()) => DshCliExit::Ok,
          Err(error) => DshCliExit::from(error),
        },
        Err(error) => DshCliExit::CliErr(error),
      }
    }
    Some(("logout", arg_matches)) => {
      return match get_target_platform_from_all_sources(arg_matches, context.settings()) {
        Ok(platform) => match authentication::logout(platform, context).await {
          Ok(()) => DshCliExit::Ok,
          Err(error) => DshCliExit::from(error),
        },
        Err(error) => DshCliExit::CliErr(error),
      }
    }
    Some((subject_command_name, sub_matches)) => match subject_registry.get(subject_command_name) {
      Some(subject) => {
        let requirements = subject.requirements(sub_matches);
        debug!("{}", requirements);
        if requirements.needs_dsh_api_client() {
          let clients = match create_clients(&matches, &requirements, &context).await {
            Ok(Some(clients)) => clients,
            Ok(None) => return DshCliExit::ErrContext("user is not authenticated".to_string(), Box::new(context)),
            Err(error) => return DshCliExit::CliErrContext(error, Box::new(context)),
          };
          for client in &clients {
            if matches.get_flag(TENANTS_ALL_FLAG) {
              context.println(format!("# {}", client.tenant()))
            }
            match subject.execute_subject_command_with_client(sub_matches, client, &context).await {
              Ok(_) => {}
              Err(error) => {
                return DshCliExit::CliErrContext(error, Box::new(context));
              }
            }
          }
        } else {
          match subject.execute_subject_command_without_client(sub_matches, &context).await {
            Ok(_) => {}
            Err(error) => {
              return DshCliExit::CliErrContext(error, Box::new(context));
            }
          }
        }
      }
      None => match subject_list_shortcut_registry.get(subject_command_name) {
        Some(subject_list_shortcut) => {
          let requirements = subject_list_shortcut.requirements_list_shortcut(sub_matches);
          debug!("{}", requirements);
          if requirements.needs_dsh_api_client() {
            let clients = match create_clients(&matches, &requirements, &context).await {
              Ok(Some(clients)) => clients,
              Ok(None) => return DshCliExit::ErrContext("user is not authenticated".to_string(), Box::new(context)),
              Err(error) => return DshCliExit::CliErrContext(error, Box::new(context)),
            };
            for client in &clients {
              if matches.get_flag(TENANTS_ALL_FLAG) {
                context.println(format!("# {}", client.tenant()))
              }
              match subject_list_shortcut.execute_subject_list_shortcut_with_client(sub_matches, client, &context).await {
                Ok(_) => {}
                Err(error) => {
                  return DshCliExit::CliErrContext(error, Box::new(context));
                }
              }
            }
          } else {
            match subject_list_shortcut.execute_subject_list_shortcut_without_client(sub_matches, &context).await {
              Ok(_) => {}
              Err(error) => {
                return DshCliExit::CliErrContext(error, Box::new(context));
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

fn login_command() -> Command {
  Command::new("login")
    .about("Login via single-sign-on")
    .long_about(
      "Login via single-sign-on. You will be directed to the login page for the currently \
        selected platform where you must sign in using your credentials (using two-factor \
        authentication). The platform is either the configured platform, or can be specified \
        via the '--platform' argument.",
    )
    .arg(platform_name_argument())
}

fn logout_command() -> Command {
  Command::new("logout")
    .about("Logout from single-sign-on")
    .long_about(
      "Logout from single-sign-on. You will be directed to the logout page for the currently \
        selected platform where you must confirm.",
    )
    .arg(platform_name_argument())
}

async fn create_command(clap_commands: &Vec<Command>, settings: &Settings) -> Command {
  let long_about = match enabled_features() {
    Some(enabled_features) => format!("{} Enabled features: {}.", LONG_ABOUT, enabled_features.join(", ")),
    None => LONG_ABOUT.to_string(),
  };
  let mut command = Command::new(APPLICATION_NAME)
    .about(ABOUT)
    .author(AUTHOR)
    .long_about(long_about)
    .disable_help_subcommand(true)
    .subcommands(clap_commands)
    .args(vec![
      // Main options
      authentication_option(),
      browser_option(),
      dry_run_flag(),
      environment_variable_option(),
      force_flag(),
      platform_option(),
      tenant_option(),
      tenants_all_flag(),
      tenants_option(),
      // Output options
      no_csv_headers_flag(),
      no_escape_flag(),
      output_directory_option(),
      output_format_option(),
      quiet_flag(),
      set_verbosity_option(),
      show_execution_time_flag(),
      terminal_width_option(),
      // Tool options
      env_var_argument(),
      env_var_file_argument(),
      env_vars_argument(),
      generate_autocomplete_file_argument(),
      log_level_option(),
      log_level_api_option(),
      password_file_option(),
      releases_flag(),
      suppress_exit_status_flag(),
      version_flag(),
    ])
    .subcommand_value_name("SUBJECT/COMMAND")
    .subcommand_help_heading("Subjects/commands")
    .arg_required_else_help(true)
    .max_term_width(120)
    .hide_possible_values(false)
    .styles(STYLES.clone())
    .disable_version_flag(true);

  let mut after_help: Vec<String> = vec![];

  let environment_variables: Vec<(&str, String)> = get_configured_environment_variables();
  if !environment_variables.is_empty() {
    let environment_variables_table = to_help_items("Environment variables:", environment_variables);
    after_help.push(environment_variables_table);
  }
  let non_empty_settings = settings.non_empty_attributes().unwrap_or_else(|error| unreachable!("{}", error));
  if !non_empty_settings.is_empty() {
    let settings_table = to_help_items("Settings:", non_empty_settings.iter().map(|(a, b)| (a.as_str(), b.to_string())).collect_vec());
    after_help.push(settings_table);
  }

  if let Ok(access_tokens) = get_access_tokens().await {
    if !access_tokens.is_empty() {
      let access_tokens: Vec<(&str, String)> = access_tokens
        .iter()
        .flat_map(|(platform, access_token)| {
          access_token.tenant_permissions.as_ref().map(|tenant_permissions| {
            (platform.name(), {
              tenant_permissions
                .iter()
                .map(|permission| permission.tenant.to_string())
                .collect_vec()
                .chunks(6)
                .collect_vec()
                .iter()
                .map(|tenants| tenants.join(", "))
                .collect_vec()
                .iter()
                .join(",\n")
            })
          })
        })
        .collect_vec();
      let authorizations = to_help_items("Authorizations:", access_tokens);
      after_help.push(authorizations);
    }
  }
  command = command.after_help(after_help.join("\n\n"));
  after_help.push(AFTER_HELP.to_string());
  command = command.after_long_help(after_help.join("\n\n"));

  command
}

fn read_single_line(prompt: impl AsRef<str>) -> DshCliResult<String> {
  print!("{}", prompt.as_ref());
  let _ = stdout().lock().flush();
  let mut line = String::new();
  stdin().read_line(&mut line).expect("could not read line");
  Ok(line.trim().to_string())
}

fn include_started_stopped(matches: &ArgMatches) -> (bool, bool) {
  match (matches.get_flag(FilterFlagType::Started.id()), matches.get_flag(FilterFlagType::Stopped.id())) {
    (false, false) => (true, true),
    (false, true) => (false, true),
    (true, false) => (true, false),
    (true, true) => (true, true),
  }
}

fn read_and_deserialize_from_toml_file<T>(toml_file: impl AsRef<Path>) -> DshCliResult<Option<T>>
where
  T: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(&toml_file) {
    Ok(toml_string) => match toml::from_str::<T>(&toml_string) {
      Ok(deserialized_toml) => Ok(Some(deserialized_toml)),
      Err(de_error) => {
        let message = format!("could not deserialize file '{}' ({})", toml_file.as_ref().display(), de_error.message());
        error!("{}", &message);
        Err(DshCliError::from(message))
      }
    },
    Err(io_error) => match io_error.kind() {
      NotFound => Ok(None),
      _ => {
        let message = format!("could not read file '{}'", toml_file.as_ref().display());
        error!("{}", &message);
        Err(DshCliError::from(message))
      }
    },
  }
}

fn serialize_and_write_to_toml_file<T>(toml_file: impl AsRef<Path>, data: &T) -> DshCliResult<()>
where
  T: Serialize,
{
  match toml::to_string(data) {
    Ok(toml_string) => match fs::write(&toml_file, toml_string) {
      Ok(_) => Ok(()),
      Err(io_error) => {
        let message = format!("could not write file '{}' ({})", toml_file.as_ref().display(), io_error);
        error!("{}", &message);
        Err(DshCliError::from(message))
      }
    },
    Err(ser_error) => {
      let message = format!("could not serialize data ({})", ser_error);
      error!("{}", &message);
      Err(DshCliError::from(message))
    }
  }
}

/// Manually edit a configuration file
///
/// Will serialize the provided `configuration` to a temporary file
/// and open that file in the default system editor.
/// When the editor closes, the temporary file will be serialized again and returned.
async fn edit_configuration<C>(configuration: &C, temporary_configuration_file_name: &str, matches: &ArgMatches) -> DshCliResult<Option<C>>
where
  C: for<'de> Deserialize<'de> + Serialize,
{
  match environment_variable("EDITOR", Some(matches))? {
    Some(editor_from_env_var) => {
      let editor = editor_from_env_var.split(" ").collect_vec();
      let editor_command = editor.first().ok_or("".to_string())?;
      let editor_args = editor.iter().skip(1).collect_vec();
      debug!("editor: {} {:?}", editor_command, editor_args);
      let mut temporary_configuration_file_path = temp_dir();
      temporary_configuration_file_path.push(temporary_configuration_file_name);
      debug!("temporary configuration file: {}", temporary_configuration_file_path.display());
      let original_configuration = serde_json::to_string_pretty::<C>(configuration)?;
      tokio::fs::write(&temporary_configuration_file_path, &original_configuration)
        .await
        .map_err(error_map!("cannot write temporary configuration file ({})"))?;
      process::Command::new(editor_command)
        .args(editor_args)
        .arg(&temporary_configuration_file_path)
        .status()
        .map_err(error_map!("couldn't edit temporary configuration file: {}"))?;
      let updated_configuration = tokio::fs::read_to_string(&temporary_configuration_file_path)
        .await
        .map_err(error_map!("couldn't read temporary configuration file ({})"))?;
      if original_configuration == updated_configuration {
        Ok(None)
      } else {
        Ok(Some(
          serde_json::from_str::<C>(&updated_configuration).map_err(error_map!("could not parse temporary configuration file ({})"))?,
        ))
      }
    }
    None => err!("environment variable 'EDITOR' is not set"),
  }
}

// Method will panic if rows vector is empty
fn to_help_items(header: &str, rows: Vec<(&str, String)>) -> String {
  let bold_green = Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green)));
  let bold_blue = Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Blue)));
  let key_value_length_pairs: Vec<(&str, &str, usize)> = rows.iter().map(|(key, value)| (*key, value.as_ref(), key.len())).collect_vec();
  let first_column_width = &key_value_length_pairs.iter().map(|(_, _, len)| len).max().unwrap_or_else(|| unreachable!()).clone();
  let mut pairs = vec![];
  for (key, value, len) in key_value_length_pairs {
    let values = value.split("\n").collect_vec().iter().map(|s| s.to_string()).collect_vec();
    pairs.push(format!(
      "  {bold_blue}{}{bold_blue:#}{}  {}",
      key,
      " ".repeat(first_column_width - len),
      values.first().map(|s| s.to_string()).unwrap_or_default()
    ));
    for v in values[1..].iter() {
      pairs.push(format!("  {}  {}", " ".repeat(*first_column_width), v));
    }
  }
  format!("{bold_green}{}{bold_green:#}\n{}", header, pairs.join("\n"))
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

#[derive(Eq, Hash, PartialEq, Serialize)]
enum ReleaseLabel {
  Release,
  Asset,
  NumberOfDownloads,
  CreatedAt,
  Size,
}

impl Label for ReleaseLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Release => "release",
      Self::Asset => "asset",
      Self::NumberOfDownloads => "downloads",
      Self::CreatedAt => "created at",
      Self::Size => "size",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Release)
  }
}

impl SubjectFormatter<ReleaseLabel> for ReleaseAsset {
  fn value(&self, label: &ReleaseLabel, target_id: &str) -> Value {
    match label {
      ReleaseLabel::Release => Value::target(target_id),
      ReleaseLabel::Asset => Value::plain(&self.name),
      ReleaseLabel::NumberOfDownloads => Value::plain(self.download_count),
      ReleaseLabel::CreatedAt => Value::plain(&self.created_at),
      ReleaseLabel::Size => Value::plain(self.size),
    }
  }
}

pub(crate) static RELEASE_LABELS: [ReleaseLabel; 5] = [ReleaseLabel::Release, ReleaseLabel::Asset, ReleaseLabel::CreatedAt, ReleaseLabel::Size, ReleaseLabel::NumberOfDownloads];

#[test]
fn test_open_api_version() {
  assert_eq!(openapi_version(), &Version::new(1, 11, 1, None));
}

#[test]
fn test_dsh_api_version() {
  assert_eq!(crate_version(), &Version::new(0, 9, 0, None));
}
