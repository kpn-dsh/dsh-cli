#![doc(
  html_favicon_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
#![doc(
  html_logo_url = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPHN2ZyB2ZXJzaW9uPSIxLjEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeD0iMHB4IiB5PSIwcHgiCiAgICAgdmlld0JveD0iMCAwIDE3MS4zIDE4Mi45IiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCAxNzEuMyAxODIuOTsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgogICAgPHN0eWxlPgoJCSNrcG5fbG9nbyB7CgkJCWZpbGw6IGJsYWNrOwoJCX0KCgkJQG1lZGlhIChwcmVmZXJzLWNvbG9yLXNjaGVtZTogZGFyaykgewoJCQkja3BuX2xvZ28gewoJCQkJZmlsbDogd2hpdGU7CgkJCX0KCQl9Cgk8L3N0eWxlPgogICAgPGcgaWQ9Imtwbl9sb2dvIj4KCQk8cGF0aCBkPSJNMTYxLjcsNzIuMWMtNS40LTUuNC0xNS4zLTExLjgtMzIuMi0xMS44Yy0zLjEsMC02LjIsMC4yLTkuMSwwLjZsLTAuOSwwLjFsMC4zLDAuOWMwLjgsMi42LDEuNCw1LjUsMS44LDguNGwwLjEsMC44CgkJCWwwLjgtMC4xYzIuNC0wLjMsNC43LTAuNCw3LTAuNGMxMy40LDAsMjEsNC44LDI1LDguOGM0LjIsNC4yLDYuNSw5LjYsNi41LDE1YzAsNi45LTMuNiwxNS42LTcuMiwyNC4xYy0xLjcsNC4yLTQuOSwxMi4zLTYuNywxOS4yCgkJCWMtMy4zLDEzLjEtOC44LDM1LTIxLjksMzVjLTQuMywwLTkuNC0yLjQtMTUuNS03LjJjLTMuMywxLjktNi44LDMuNC0xMC41LDQuNmM5LjgsOC43LDE4LjEsMTIuOCwyNiwxMi44CgkJCWMyMS4yLDAsMjguMS0yNy44LDMxLjgtNDIuN2MxLjEtNC42LDMuMy0xMC44LDYuMi0xNy43YzMuOS05LjQsOC0xOS4xLDgtMjhDMTcxLjMsODYuMywxNjcuOCw3OC4yLDE2MS43LDcyLjF6Ii8+CgkJPHBhdGggZD0iTTExNiw1Mi4ybDAuOS0wLjJjMi45LTAuNSw1LjktMC44LDkuMS0xYzAuMywwLDAuNiwwLDAuOSwwQzExMi45LDE3LjcsNzcuMiwwLDU2LjcsMEMyOS42LDAsMjAsMjcuNiwyMCw1My40CgkJCWMwLDEyLDQuMSwyNC42LDcuNSwzM2wwLjMsMC44bDAuOC0wLjNjMi40LTEuMSw1LTIuMSw4LTMuMmwwLjgtMC4zTDM3LDgyLjZjLTQuMy0xMC42LTYuOC0yMS4zLTYuOC0yOS4yYzAtMTYuNSw0LTMwLDExLjEtMzcKCQkJYzQuMS00LjEsOS4xLTYuMSwxNS40LTYuMUM3Mi44LDEwLjMsMTAzLDI1LjIsMTE2LDUyLjJ6Ii8+CgkJPHBhdGggZD0iTTk0LjksMTUxLjNsLTAuNC0wLjRsLTAuNSwwLjJjLTUuNSwyLTExLjEsMi45LTE3LjIsMi45Yy0yMCwwLTQxLjgtOC45LTU1LjYtMjIuOGMtNi45LTYuOS0xMC45LTE0LjMtMTAuOS0yMC4yCgkJCWMwLTguMSwzLTE0LjEsOS40LTE5Yy0xLjItMi45LTIuNi02LjMtMy44LTkuOUM1LjIsODkuMiwwLDk4LjcsMCwxMTFjMCw4LjcsNC45LDE4LjUsMTMuOSwyNy41YzEyLjQsMTIuNSwzNS41LDI1LjgsNjIuOSwyNS44CgkJCWM4LjYsMCwxNi44LTEuNywyNC40LTVsMS4xLTAuNWwtMC44LTAuOEM5OS4xLDE1NS43LDk2LjksMTUzLjQsOTQuOSwxNTEuM3oiLz4KCQk8cGF0aCBkPSJNODMuMiw3OS45di05QzgxLDcwLjMsNzguNSw3MCw3NS45LDcwYy0xMC41LDAtMTUuNiw3LjEtMTUuNiwxNC4yYzAsNi44LDIuNSwxMy4zLDExLjksMjcuOWMzLjgtMC41LDcuNi0wLjgsMTEuNC0wLjkKCQkJYy04LjItMTUuMi0xMC4yLTIwLjYtMTAuMi0yNC41YzAtNC41LDIuNi02LjgsNy45LTYuOEM4Miw3OS44LDgyLjYsNzkuOSw4My4yLDc5Ljl6Ii8+CgkJPHBhdGggZD0iTTU0LjcsOTMuMWMtMC44LTItMS4zLTUuMy0xLjYtNy43Yy04LjMtMC4zLTE0LjYsNC41LTE0LjYsMTEuMmMwLDUuNCwyLjgsMTAuMiwxNC4yLDE5LjljMi45LTEsNi44LTIuMSwxMC4xLTIuOAoJCQljLTExLjItMTAuNS0xMy0xMy4zLTEzLTE2LjRDNTAsOTUuMSw1MS42LDkzLjYsNTQuNyw5My4xeiIvPgoJCTxwYXRoIGQ9Ik05MC45LDc5Ljl2LTljMi4xLTAuNiw0LjctMC45LDcuMy0wLjljMTAuNCwwLDE1LjYsNy4xLDE1LjYsMTQuMmMwLDYuOC0yLjUsMTMuMy0xMS45LDI3LjljLTMuOC0wLjUtNy42LTAuOC0xMS40LTAuOQoJCQljOC4yLTE1LjIsMTAuMi0yMC42LDEwLjItMjQuNWMwLTQuNS0yLjYtNi44LTcuOS02LjhDOTIsNzkuOCw5MS40LDc5LjksOTAuOSw3OS45eiIvPgoJCTxwYXRoIGQ9Ik0xMTkuMyw5My4xYzAuOC0yLDEuMy01LjMsMS42LTcuN2M4LjMtMC4zLDE0LjYsNC41LDE0LjYsMTEuMmMwLDUuNC0yLjgsMTAuMi0xNC4yLDE5LjljLTIuOS0xLTYuOC0yLjEtMTAuMS0yLjgKCQkJYzExLjItMTAuNSwxMy0xMy4zLDEzLTE2LjRDMTI0LjEsOTUuMSwxMjIuNSw5My42LDExOS4zLDkzLjF6Ii8+CgkJPHBhdGggZD0iTTg3LDEzMC4yYzguNCwwLDE3LDEuMSwyNS45LDMuOGwzLTEwYy0xMC0zLTE5LjgtNC4yLTI5LTQuMmMtOS4yLDAtMTguOSwxLjItMjksNC4ybDMsMTBDNzAsMTMxLjMsNzguNiwxMzAuMiw4NywxMzAuMnoiCgkJLz4KCQk8cmVjdCB4PSI4MC41IiB5PSI0OS4zIiB0cmFuc2Zvcm09Im1hdHJpeCgwLjcwNzIgLTAuNzA3MSAwLjcwNzEgMC43MDcyIC0xMy45OTkyIDc3Ljg3NDQpIiB3aWR0aD0iMTMuMSIKCQkJICBoZWlnaHQ9IjEzLjEiLz4KCTwvZz4KPC9zdmc+Cg=="
)]
extern crate core;

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{stdin, stdout, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::process::{ExitCode, Termination};

use crate::arguments::{
  dry_run_argument, force_argument, guid_argument, log_level_api_argument, log_level_argument, matching_style_argument, no_escape_argument, output_format_argument,
  password_file_argument, platform_argument, quiet_argument, set_verbosity_argument, show_execution_time_argument, tenant_argument, terminal_width_argument, LogLevel,
  GUID_ARGUMENT, LOG_LEVEL_API_ARGUMENT, LOG_LEVEL_ARGUMENT, PASSWORD_FILE_ARGUMENT, PLATFORM_ARGUMENT, TENANT_ARGUMENT,
};
use crate::autocomplete::{generate_autocomplete_file, generate_autocomplete_file_argument, AutocompleteShell, AUTOCOMPLETE_ARGUMENT};
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::settings::{get_password_from_keyring, read_settings, read_target, read_target_and_password, Settings};
use crate::subject::Subject;
use crate::subjects::api::API_SUBJECT;
use crate::subjects::application::APPLICATION_SUBJECT;
use crate::subjects::platform::PLATFORM_SUBJECT;
use clap::builder::{styling, Styles};
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::dsh_api_tenant::{parse_and_validate_guid, DshApiTenant};
use dsh_api::platform::DshPlatform;
use dsh_api::{api_version, crate_version, ENV_VAR_PLATFORMS_FILE_NAME};
use lazy_static::lazy_static;
use log::{debug, LevelFilter};
use rpassword::prompt_password;
use subjects::app::APP_SUBJECT;
use subjects::bucket::BUCKET_SUBJECT;
use subjects::certificate::CERTIFICATE_SUBJECT;
use subjects::env::ENV_SUBJECT;
use subjects::image::IMAGE_SUBJECT;
use subjects::manifest::MANIFEST_SUBJECT;
use subjects::metric::METRIC_SUBJECT;
use subjects::proxy::PROXY_SUBJECT;
use subjects::secret::SECRET_SUBJECT;
use subjects::setting::SETTING_SUBJECT;
// #[cfg(feature = "stream")]
// use subjects::stream::STREAM_SUBJECT;
use crate::subjects::token::TOKEN_SUBJECT;
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
mod modifier_flags;
mod settings;
mod subject;
mod subjects;

lazy_static! {
  static ref STYLES: Styles = Styles::styled()
    .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
    .placeholder(styling::AnsiColor::Cyan.on_default());
}

pub(crate) static APPLICATION_NAME: &str = "dsh";

/// Short help text, shown when `-h` was provided
static ABOUT: &str = "DSH resource management api command line interface.";
static AUTHOR: &str = "KPN DSH Team, unibox@kpn.com";
/// Long help text, shown when `--help` was provided
static LONG_ABOUT: &str = "DSH resource management api command line interface\n\n\
   The DSH api command line tool enables the user to call a subset of the functions \
   in the DSH api from the command line. \
   It also supports functions that are not supported directly from the DSH api, \
   such as finding all applications that use a certain resource (e.g. a secret) or showing a \
   list of all resources of a certain type (e.g. list all volumes).";
/// Will be shown after normal help text, when `-h` was provided
static AFTER_HELP: &str = "For most commands adding an 's' as a postfix will yield the same result \
   as using the 'list' subcommand, e.g. using 'dsh apps' will be the same \
   as using 'dsh app list'.";

static VERSION: &str = "0.4.1";

static ENV_VAR_PLATFORM: &str = "DSH_CLI_PLATFORM";
static ENV_VAR_TENANT: &str = "DSH_CLI_TENANT";
static ENV_VAR_GUID: &str = "DSH_CLI_GUID";
static ENV_VAR_PASSWORD: &str = "DSH_CLI_PASSWORD";
static ENV_VAR_PASSWORD_FILE: &str = "DSH_CLI_PASSWORD_FILE";

static ENV_VAR_LOG_LEVEL: &str = "DSH_CLI_LOG_LEVEL";
static ENV_VAR_LOG_LEVEL_API: &str = "DSH_CLI_LOG_LEVEL_API";

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
    MANIFEST_SUBJECT.as_ref(),
    METRIC_SUBJECT.as_ref(),
    PLATFORM_SUBJECT.as_ref(),
    PROXY_SUBJECT.as_ref(),
    SECRET_SUBJECT.as_ref(),
    // #[cfg(feature = "stream")]
    // STREAM_SUBJECT.as_ref(),
    TOKEN_SUBJECT.as_ref(),
    TOPIC_SUBJECT.as_ref(),
    VHOST_SUBJECT.as_ref(),
    VOLUME_SUBJECT.as_ref(),
    SETTING_SUBJECT.as_ref(),
    TARGET_SUBJECT.as_ref(),
  ];

  let mut subject_registry: HashMap<String, &(dyn Subject + Send + Sync)> = HashMap::new();
  let mut subject_list_shortcut_registry: HashMap<String, &(dyn Subject + Send + Sync)> = HashMap::new();

  let mut clap_commands: Vec<Command> = Vec::new();

  for subject in subjects {
    let (command_name, clap_command) = subject.clap_subject_command();
    subject_registry.insert(command_name.to_string(), subject);
    clap_commands.push(clap_command);
    if let Some((list_shortcut_name, clap_list_command_shortcut)) = subject.clap_list_shortcut_command() {
      subject_list_shortcut_registry.insert(list_shortcut_name.to_string(), subject);
      clap_commands.push(clap_list_command_shortcut);
    }
  }

  let mut command = create_command(&clap_commands);

  let matches = command.clone().get_matches();

  if let Some(shell) = matches.get_one::<AutocompleteShell>(AUTOCOMPLETE_ARGUMENT) {
    generate_autocomplete_file(&mut command, shell);
    return Ok(());
  }

  let settings = read_settings(None)?;

  initialize_logger(&matches, settings.as_ref())?;

  debug!("{:#?}", settings);

  match matches.subcommand() {
    Some((subject_command_name, sub_matches)) => match subject_registry.get(subject_command_name) {
      Some(subject) => {
        let requirements = subject.requirements(sub_matches);
        if requirements.needs_dsh_api_client {
          let factory = get_api_client_factory(&matches, settings.as_ref()).await?;
          let context = Context::create(&matches, &requirements, Some(factory.client().await?))?;
          subject.execute_subject_command(sub_matches, &context).await?;
        } else {
          let context = Context::create(&matches, &requirements, None)?;
          subject.execute_subject_command(sub_matches, &context).await?;
        };
      }
      None => match subject_list_shortcut_registry.get(subject_command_name) {
        Some(subject) => {
          let requirements = subject.requirements(sub_matches);
          if requirements.needs_dsh_api_client {
            let factory = get_api_client_factory(&matches, settings.as_ref()).await?;
            let context = Context::create(&matches, &requirements, Some(factory.client().await?))?;
            subject.execute_subject_list_shortcut(sub_matches, &context).await?;
          } else {
            let context = Context::create(&matches, &requirements, None)?;
            subject.execute_subject_list_shortcut(sub_matches, &context).await?;
          };
        }
        None => return Err("unexpected error, list shortcut not found".to_string()),
      },
    },
    None => return Err("unexpected error, no subcommand".to_string()),
  };
  Ok(())
}

fn initialize_logger(matches: &ArgMatches, settings: Option<&Settings>) -> Result<(), String> {
  let log_level_dsh: LogLevel = match matches.get_one::<LogLevel>(LOG_LEVEL_ARGUMENT) {
    Some(log_level_from_argument) => log_level_from_argument.clone(),
    None => match std::env::var(ENV_VAR_LOG_LEVEL) {
      Ok(log_level_from_env_var) => LogLevel::try_from(log_level_from_env_var.as_str())?,
      Err(_) => settings.and_then(|settings| settings.log_level.clone()).unwrap_or_else(|| LogLevel::Error),
    },
  };
  let log_level_dsh_api: LogLevel = match matches.get_one::<LogLevel>(LOG_LEVEL_API_ARGUMENT) {
    Some(log_level_api_from_argument) => log_level_api_from_argument.clone(),
    None => match std::env::var(ENV_VAR_LOG_LEVEL_API) {
      Ok(log_level_api_from_env_var) => LogLevel::try_from(log_level_api_from_env_var.as_str())?,
      Err(_) => settings.and_then(|settings| settings.log_level_api.clone()).unwrap_or_else(|| LogLevel::Error),
    },
  };
  env_logger::builder()
    .filter_module("dsh", LevelFilter::from(log_level_dsh))
    .filter_module("dsh_api", LevelFilter::from(log_level_dsh_api))
    .init();
  Ok(())
}

fn create_command(clap_commands: &Vec<Command>) -> Command {
  Command::new(APPLICATION_NAME)
    .about(ABOUT)
    .author(AUTHOR)
    .long_about(LONG_ABOUT)
    .after_help(AFTER_HELP)
    .args(vec![
      platform_argument(),
      tenant_argument(),
      guid_argument(),
      password_file_argument(),
      output_format_argument(), // TODO Should this one be at this level?
      set_verbosity_argument(),
      dry_run_argument(),
      force_argument(),
      matching_style_argument(), // TODO Should this one be at this level?
      no_escape_argument(),      // TODO Should this one be at this level?
      quiet_argument(),
      log_level_argument(),
      log_level_api_argument(),
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
      "version: {}\ndsh-api library version: {}\ndsh rest api version: {}",
      VERSION,
      crate_version(),
      api_version()
    ))
}

async fn get_api_client_factory(matches: &ArgMatches, settings: Option<&Settings>) -> Result<DshApiClientFactory, String> {
  let dsh_api_tenant: DshApiTenant = get_dsh_api_tenant(matches, settings)?;
  let password = get_password(matches, &dsh_api_tenant)?;
  Ok(DshApiClientFactory::create(dsh_api_tenant, password)?)
}

fn get_dsh_api_tenant(matches: &ArgMatches, settings: Option<&Settings>) -> Result<DshApiTenant, String> {
  let platform = get_platform(matches, settings)?;
  let tenant_name = get_tenant_name(matches, settings)?;
  let guid = get_guid(matches, &platform, &tenant_name)?;
  Ok(DshApiTenant::new(tenant_name, guid, platform))
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
fn get_platform(matches: &ArgMatches, settings: Option<&Settings>) -> Result<DshPlatform, String> {
  match matches.get_one::<String>(PLATFORM_ARGUMENT) {
    Some(platform_from_argument) => Ok(DshPlatform::try_from(platform_from_argument.as_str()).unwrap()),
    None => match std::env::var(ENV_VAR_PLATFORM) {
      Ok(platform_name_from_env_var) => DshPlatform::try_from(platform_name_from_env_var.as_str()),
      Err(_) => match settings.and_then(|settings| settings.default_platform.clone()) {
        Some(default_platform_name_from_settings) => DshPlatform::try_from(default_platform_name_from_settings.as_str()),
        None => {
          if stdin().is_terminal() {
            DshPlatform::try_from(read_single_line("platform name: ")?.as_str())
          } else {
            Err("could not determine platform, please check configuration".to_string())
          }
        }
      },
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
fn get_tenant_name(matches: &ArgMatches, settings: Option<&Settings>) -> Result<String, String> {
  match matches.get_one::<String>(TENANT_ARGUMENT) {
    Some(tenant_name_from_argument) => Ok(tenant_name_from_argument.to_string()),
    None => match std::env::var(ENV_VAR_TENANT) {
      Ok(tenant_name_from_env_var) => Ok(tenant_name_from_env_var),
      Err(_) => match settings.and_then(|settings| settings.default_tenant.clone()) {
        Some(tenant_name_from_settings) => Ok(tenant_name_from_settings),
        None => {
          if stdin().is_terminal() {
            read_single_line("tenant: ")
          } else {
            Err("could not determine tenant, please check configuration".to_string())
          }
        }
      },
    },
  }
}

/// # Get the target guid
///
/// This method will get the target group and user id.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--guid`.
/// 1. Environment variable `DSH_CLI_GUID`.
/// 1. Parameter `group-user-id` from the target settings file, if available.
/// 1. If stdin is a terminal, ask the user to enter the value.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `platform` - used to determine the target settings file
/// * `tenant_name` - used to determine the environment variable or the target settings file
///
/// ## Returns
/// An `Ok<u16>` containing the group and user id, or an `Err<String>`.
fn get_guid(matches: &ArgMatches, platform: &DshPlatform, tenant_name: &str) -> Result<u16, String> {
  match matches.get_one::<String>(GUID_ARGUMENT) {
    Some(guid_from_argument) => Ok(parse_and_validate_guid(guid_from_argument.clone())?),
    None => match std::env::var(ENV_VAR_GUID) {
      Ok(guid_from_env_var) => Ok(parse_and_validate_guid(guid_from_env_var)?),
      Err(_) => match read_target(platform, tenant_name)? {
        Some(target) => Ok(target.group_user_id),
        None => {
          if stdin().is_terminal() {
            read_single_line(format!("group and user id for tenant {}: ", tenant_name).as_str()).and_then(|guid| parse_and_validate_guid(guid).map_err(|e| e.to_string()))
          } else {
            Err("could not determine group and user id and unable to prompt user, please check configuration".to_string())
          }
        }
      },
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
/// 1. Entry `dsh.[platform].[tenant_name]` from the keychain, if available.
///    This can result in a pop-up where the user must authenticate for the keychain.
/// 1. Parameter `password` from the target settings file, if available.
/// 1. If stdin is a terminal, ask the user to enter the password.
/// 1. Else return with an error.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `dsh_api_tenant` - used to determine the target settings file
///
/// ## Returns
/// An `Ok<String>` containing the password, or an `Err<String>`.
fn get_password(matches: &ArgMatches, dsh_api_tenant: &DshApiTenant) -> Result<String, String> {
  match matches.get_one::<PathBuf>(PASSWORD_FILE_ARGUMENT) {
    Some(password_file_from_arg) => read_password_file(password_file_from_arg),
    None => match std::env::var(ENV_VAR_PASSWORD_FILE) {
      Ok(password_file_from_env) => read_password_file(password_file_from_env),
      Err(_) => match std::env::var(ENV_VAR_PASSWORD) {
        Ok(password_from_env_var) => Ok(password_from_env_var),
        Err(_) => match get_password_from_keyring(dsh_api_tenant.platform(), dsh_api_tenant.name())? {
          Some(password_from_keyring) => Ok(password_from_keyring),
          None => match read_target_and_password(dsh_api_tenant.platform(), dsh_api_tenant.name())?.and_then(|target| target.password) {
            Some(password_from_target) => Ok(password_from_target),
            None => {
              if stdin().is_terminal() {
                read_single_line_password(format!("password for tenant {}: ", dsh_api_tenant).as_str())
              } else {
                Err("could not determine password and unable to to prompt user, please check configuration".to_string())
              }
            }
          },
        },
      },
    },
  }
}

fn read_password_file<T: AsRef<Path>>(password_file: T) -> Result<String, String> {
  match std::fs::read_to_string(&password_file) {
    Ok(password_string) => {
      let trimmed_password = password_string.trim();
      if trimmed_password.is_empty() {
        Err(format!("password file '{}' is empty", password_file.as_ref().to_string_lossy()))
      } else {
        Ok(trimmed_password.to_string())
      }
    }
    Err(_) => Err(format!("password file '{}' could not be read", password_file.as_ref().to_string_lossy())),
  }
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
  for (env_var, value) in std::env::vars() {
    if env_var.starts_with("DSH_CLI_") {
      environment_variables.push((env_var, value));
    }
  }
  if let Ok(platforms_file) = std::env::var(ENV_VAR_PLATFORMS_FILE_NAME) {
    environment_variables.push((ENV_VAR_PLATFORMS_FILE_NAME.to_string(), platforms_file));
  }
  environment_variables.sort_by(|(env_var_a, _), (env_var_b, _)| env_var_a.cmp(env_var_b));
  environment_variables
}

#[test]
fn test_open_api_version() {
  assert_eq!(api_version(), "1.9.0");
}

#[test]
fn test_dsh_api_version() {
  assert_eq!(crate_version(), "0.4.1");
}
