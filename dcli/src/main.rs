#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{stdin, stdout, Write};
use std::process;
use std::process::{ExitCode, Termination};
use std::time::Instant;

use crate::app::APP_SUBJECT;
use crate::application::APPLICATION_SUBJECT;
use crate::arguments::{
  guid_argument, hide_border_argument, password_argument, platform_argument, set_verbosity_argument, show_execution_time_argument, tenant_argument, PlatformArgument,
  GUID_ARGUMENT, PASSWORD_ARGUMENT, PLATFORM_ARGUMENT, TENANT_ARGUMENT,
};
use crate::autocomplete::{generate_autocomplete_file, generate_autocomplete_file_argument, AutocompleteShell, AUTOCOMPLETE_ARGUMENT};
use crate::bucket::BUCKET_SUBJECT;
use crate::certificate::CERTIFICATE_SUBJECT;
use crate::context::get_dcli_context;
use crate::env::ENV_SUBJECT;
use crate::filter_flags::FilterFlagType;
use crate::image::IMAGE_SUBJECT;
use crate::manifest::MANIFEST_SUBJECT;
use crate::metric::METRIC_SUBJECT;
use crate::proxy::PROXY_SUBJECT;
use crate::secret::SECRET_SUBJECT;
use crate::setting::SETTING_SUBJECT;
use crate::settings::{get_password_from_keyring, read_settings, read_target, Settings};
#[cfg(feature = "stream")]
use crate::stream::STREAM_SUBJECT;
use crate::subject::{clap_list_shortcut_command, clap_subject_command, Subject};
use crate::target::TARGET_SUBJECT;
use crate::topic::TOPIC_SUBJECT;
use crate::vhost::VHOST_SUBJECT;
use crate::volume::VOLUME_SUBJECT;
use clap::builder::styling;
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::dsh_api_tenant::{parse_and_validate_guid, DshApiTenant};
use dsh_api::platform::DshPlatform;
use dsh_api::{guid_environment_variable, secret_environment_variable, DshApiError, PLATFORM_ENVIRONMENT_VARIABLE, TENANT_ENVIRONMENT_VARIABLE};
use termion::input::TermRead;

mod app;
mod application;
mod arguments;
mod autocomplete;
mod bucket;
mod capability;
mod capability_builder;
mod certificate;
mod context;
mod env;
mod filter_flags;
mod flags;
mod formatters;
mod image;
mod manifest;
mod metric;
mod modifier_flags;
mod proxy;
mod secret;
mod setting;
mod settings;
#[cfg(feature = "stream")]
mod stream;
mod subject;
mod target;
mod topic;
mod vhost;
mod volume;

pub(crate) static APPLICATION_NAME: &str = "dcli";

/// Short help text, shown when `-h` was provided
static ABOUT: &str = "DSH resource management api command line interface.";
/// Long help text, shown when `--help` was provided
static LONG_ABOUT: &str = "DSH resource management api command line interface\n\n\
   The DSH api command line tool enables the user to call a subset of the functions \
   in the DSH api from the command line. \
   It also supports functions that are not supported directly from the DSH api, \
   such as finding all applications that use a certain resource (e.g. a secret) or showing a \
   list of all resources of a certain type (e.g. list all volumes).";
/// Will be shown after normal help text, when `-h` was provided
static AFTER_HELP: &str = "For most commands adding an 's' as a postfix will yield the same result \
   as using the 'list' subcommand, e.g. using 'dcli apps' will be the same \
   as using 'dcli app list'.";
static LONG_VERSION: &str = "version: 0.2.0\ndsh-api library version: 0.2.0\ndsh rest api version: 1.8.0";

type DcliResult = Result<bool, String>;

// pub(crate) struct DcliContext<'a> {
//   pub(crate) output_format: OutputFormat,
//   pub(crate) verbosity: Verbosity,
//   pub(crate) hide_border: bool,
//   pub(crate) show_execution_time: bool,
//   pub(crate) dsh_api_client: Option<DshApiClient<'a>>,
// }

// impl DcliContext<'_> {
//   pub(crate) fn show_capability_explanation(&self) -> bool {
//     match self.verbosity {
//       Verbosity::Low => false,
//       Verbosity::Medium | Verbosity::High => true,
//     }
//   }
//
//   pub(crate) fn show_execution_time(&self) -> bool {
//     match self.verbosity {
//       Verbosity::Low | Verbosity::Medium => self.show_execution_time,
//       Verbosity::High => true,
//     }
//   }
//
//   pub(crate) fn _show_settings(&self) -> bool {
//     match self.verbosity {
//       Verbosity::Low => false,
//       Verbosity::Medium | Verbosity::High => true,
//     }
//   }
// }

#[derive(Debug)]
enum DcliExit {
  Success,
  Error(String),
}

impl Termination for DcliExit {
  fn report(self) -> ExitCode {
    match self {
      DcliExit::Success => ExitCode::SUCCESS,
      DcliExit::Error(msg) => {
        println!("{}", msg);
        ExitCode::FAILURE
      }
    }
  }
}

#[tokio::main]
async fn main() -> DcliExit {
  match inner_main().await {
    Ok(_) => DcliExit::Success,
    Err(msg) => DcliExit::Error(msg),
  }
}

async fn inner_main() -> DcliResult {
  env_logger::init();
  let _ = ctrlc::set_handler(move || {
    println!("interrupted");
    process::exit(0);
  });

  let settings = read_settings(None)?;

  let styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
    .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
    .placeholder(styling::AnsiColor::Cyan.on_default());

  let subjects: Vec<&(dyn Subject + Send + Sync)> = vec![
    APP_SUBJECT.as_ref(),
    APPLICATION_SUBJECT.as_ref(),
    BUCKET_SUBJECT.as_ref(),
    CERTIFICATE_SUBJECT.as_ref(),
    ENV_SUBJECT.as_ref(),
    IMAGE_SUBJECT.as_ref(),
    MANIFEST_SUBJECT.as_ref(),
    METRIC_SUBJECT.as_ref(),
    PROXY_SUBJECT.as_ref(),
    SECRET_SUBJECT.as_ref(),
    #[cfg(feature = "stream")]
    STREAM_SUBJECT.as_ref(),
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
    let (command_name, clap_command) = clap_subject_command(subject);
    subject_registry.insert(command_name.to_string(), subject);
    clap_commands.push(clap_command);
    if let Some((list_shortcut_name, clap_list_command_shortcut)) = clap_list_shortcut_command(subject) {
      subject_list_shortcut_registry.insert(list_shortcut_name.to_string(), subject);
      clap_commands.push(clap_list_command_shortcut);
    }
  }

  let mut command = Command::new(APPLICATION_NAME)
    .about(ABOUT)
    // .author(AUTHOR)
    // .display_name(APPLICATION_NAME)
    .long_about(LONG_ABOUT)
    .after_help(AFTER_HELP)
    // .after_long_help(AFTER_LONG_HELP)
    // .before_help("BEFORE_HELP")
    // .before_long_help("BEFORE_LONG_HELP")
    .args(vec![
      platform_argument(),
      tenant_argument(),
      guid_argument(),
      password_argument(),
      set_verbosity_argument(),
      hide_border_argument(),
      show_execution_time_argument(),
      generate_autocomplete_file_argument(),
    ])
    .subcommand_value_name("SUBJECT/COMMAND")
    .subcommand_help_heading("Subjects/commands")
    .arg_required_else_help(true)
    .max_term_width(120)
    .hide_possible_values(false)
    .styles(styles)
    .subcommands(clap_commands)
    .version("0.2.0")
    .long_version(LONG_VERSION);

  let matches = command.clone().get_matches();

  if let Some(shell) = matches.get_one::<AutocompleteShell>(AUTOCOMPLETE_ARGUMENT) {
    generate_autocomplete_file(&mut command, shell);
    return Ok(false);
  }

  let start_instant = Instant::now();

  match matches.subcommand() {
    Some((subject_command_name, sub_matches)) => match subject_registry.get(subject_command_name) {
      Some(subject) => {
        if subject.requires_dsh_api_client() {
          let factory = get_api_client_factory(&matches, settings.as_ref()).await?;
          let context = get_dcli_context(&matches, Some(factory.client().await?))?;
          let suppress_show_execution_time = subject.execute_subject_command(sub_matches, &context).await?;
          if !suppress_show_execution_time {
            context.print_execution_time(Instant::now().duration_since(start_instant).as_millis());
          }
        } else {
          let context = get_dcli_context(&matches, None)?;
          let suppress_show_execution_time = subject.execute_subject_command(sub_matches, &context).await?;
          if !suppress_show_execution_time {
            context.print_execution_time(Instant::now().duration_since(start_instant).as_millis());
          }
        };
      }
      None => match subject_list_shortcut_registry.get(subject_command_name) {
        Some(subject) => {
          if subject.requires_dsh_api_client() {
            let factory = get_api_client_factory(&matches, settings.as_ref()).await?;
            let context = get_dcli_context(&matches, Some(factory.client().await?))?;
            let suppress_show_execution_time = subject.execute_subject_list_shortcut(sub_matches, &context).await?;
            if !suppress_show_execution_time {
              context.print_execution_time(Instant::now().duration_since(start_instant).as_millis());
            }
          } else {
            let context = get_dcli_context(&matches, None)?;
            let suppress_show_execution_time = subject.execute_subject_list_shortcut(sub_matches, &context).await?;
            if !suppress_show_execution_time {
              context.print_execution_time(Instant::now().duration_since(start_instant).as_millis());
            }
          };
        }
        None => return Err("unexpected error, list shortcut not found".to_string()),
      },
    },
    None => return Err("unexpected error, no subcommand".to_string()),
  };

  Ok(false)
}

async fn get_api_client_factory(matches: &ArgMatches, settings: Option<&Settings>) -> Result<DshApiClientFactory, String> {
  let dsh_api_tenant: DshApiTenant = get_dsh_api_tenant(matches, settings)?;
  let secret = get_password(matches, &dsh_api_tenant)?;
  Ok(DshApiClientFactory::create(dsh_api_tenant, secret)?)
}

// fn get_dcli_context<'a>(matches: &'a ArgMatches, dsh_api_client: Option<DshApiClient<'a>>) -> Result<DcliContext<'a>, String> {
//   if let Some(settings) = read_settings(None)? {
//     let hide_border = if matches.get_flag(HIDE_BORDER_ARGUMENT) { true } else { settings.hide_border.unwrap_or(false) };
//     let verbosity: Verbosity = match matches.get_one::<Verbosity>(SET_VERBOSITY_ARGUMENT) {
//       Some(verbosity_argument) => verbosity_argument.to_owned(),
//       None => settings.verbosity.unwrap_or(Verbosity::Low).to_owned(),
//     };
//     let show_execution_time = if matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT) { true } else { settings.show_execution_time.unwrap_or(false) };
//     Ok(DcliContext { output_format: OutputFormat::Table, verbosity, hide_border, show_execution_time, dsh_api_client })
//   } else {
//     let hide_border = matches.get_flag(HIDE_BORDER_ARGUMENT);
//     let verbosity: Verbosity = matches.get_one::<Verbosity>(SET_VERBOSITY_ARGUMENT).unwrap_or(&Verbosity::Low).to_owned();
//     let show_execution_time = matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT);
//     Ok(DcliContext { output_format: OutputFormat::Table, verbosity, hide_border, show_execution_time, dsh_api_client })
//   }
// }

/// # Get the target platform
///
/// This method will get the target platform.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--platform`.
/// 1. Environment variable `DSH_API_PLATFORM`.
/// 1. Parameter `default-platform` from settings file, if available.
/// 1. Ask the user to enter the value.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// An `Ok<Platform>` containing the [`DshPlatform`], or an `Err<String>`.
fn get_platform(matches: &ArgMatches, settings: Option<&Settings>) -> Result<DshPlatform, String> {
  match matches.get_one::<PlatformArgument>(PLATFORM_ARGUMENT) {
    Some(name_from_argument) => DshPlatform::try_from(name_from_argument.to_string().as_str()),
    None => match std::env::var(PLATFORM_ENVIRONMENT_VARIABLE) {
      Ok(name_from_env_var) => DshPlatform::try_from(name_from_env_var.as_str()),
      Err(_) => match settings {
        Some(settings) => match settings.default_platform.clone() {
          Some(name_from_settings) => DshPlatform::try_from(name_from_settings.as_str()),
          None => DshPlatform::try_from(read_single_line("platform name: ")?.as_str()),
        },
        None => DshPlatform::try_from(read_single_line("platform name: ")?.as_str()),
      },
    },
  }
}

/// # Get the target tenant
///
/// This method will get the target tenant.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--tenant`.
/// 1. Environment variable `DSH_API_TENANT`.
/// 1. Parameter `default-tenant` from settings file, if available.
/// 1. Ask the user to enter the value.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `settings` - optional contents of the settings file, if available
///
/// ## Returns
/// An `Ok<String>` containing the tenant name, or an `Err<String>`.
fn get_tenant_name(matches: &ArgMatches, settings: Option<&Settings>) -> Result<String, String> {
  match matches.get_one::<String>(TENANT_ARGUMENT) {
    Some(name_from_argument) => Ok(name_from_argument.to_string()),
    None => match std::env::var(TENANT_ENVIRONMENT_VARIABLE) {
      Ok(name_from_env_var) => Ok(name_from_env_var),
      Err(_) => match settings {
        Some(settings) => match settings.default_tenant.clone() {
          Some(name_from_settings) => Ok(name_from_settings),
          None => read_single_line("tenant: "),
        },
        None => read_single_line("tenant: "),
      },
    },
  }
}

/// # Get the target guid
///
/// This method will get the target group and user id.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. Command line argument `--guid`.
/// 1. Environment variable `DSH_API_GUID_[tenant_name]`.
/// 1. Parameter `group-user-id` from the target settings file, if available.
/// 1. Ask the user to enter the value.
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
    None => match std::env::var(guid_environment_variable(tenant_name)) {
      Ok(guid_from_env_var) => Ok(parse_and_validate_guid(guid_from_env_var)?),
      Err(_) => match read_target(platform, tenant_name)? {
        Some(target) => Ok(target.group_user_id),
        None => read_single_line(format!("group and user id for tenant {}: ", tenant_name).as_str()).and_then(|guid| parse_and_validate_guid(guid).map_err(|e| e.to_string())),
      },
    },
  }
}

/// # Get the target password
///
/// This method will get the target password.
/// This function will try the potential sources listed below, and returns at the first match.
/// 1. If the command line argument `--password` is present, ask the user to enter the value
///    and stop if the user doesn't provide the password.
/// 1. Environment variable `DSH_API_SECRET_[platform]_[tenant_name]`.
/// 1. Entry `dcli.[platform].[tenant_name]` from the keychain, if available.
///    This can result in a pop-up where the user must authenticate for the keychain.
/// 1. Parameter `password` from the target settings file, if available.
/// 1. Ask the user to enter the value.
///
/// ## Parameters
/// * `matches` - parsed clap command line arguments
/// * `platform` - used to determine the target settings file
/// * `tenant_name` - used to determine the environment variable or the target settings file
///
/// ## Returns
/// An `Ok<u16>` containing the group and user id, or an `Err<String>`.
fn get_password(matches: &ArgMatches, dsh_api_tenant: &DshApiTenant) -> Result<String, String> {
  if matches.get_flag(PASSWORD_ARGUMENT) {
    read_single_line_password(format!("password for tenant {}: ", dsh_api_tenant).as_str())
  } else {
    match std::env::var(secret_environment_variable(dsh_api_tenant.platform().to_string().as_str(), dsh_api_tenant.name())) {
      Ok(password_from_env_var) => Ok(password_from_env_var),
      Err(_) => match get_password_from_keyring(dsh_api_tenant.platform(), dsh_api_tenant.name())? {
        Some(password_from_keyring) => Ok(password_from_keyring),
        None => match read_target(dsh_api_tenant.platform(), dsh_api_tenant.name())? {
          Some(target) => match target.password {
            Some(password_from_target) => Ok(password_from_target),
            None => read_single_line_password(format!("password for tenant {}: ", dsh_api_tenant).as_str()),
          },
          None => read_single_line_password(format!("password for tenant {}: ", dsh_api_tenant).as_str()),
        },
      },
    }
  }
}

fn get_dsh_api_tenant(matches: &ArgMatches, settings: Option<&Settings>) -> Result<DshApiTenant, String> {
  let platform = get_platform(matches, settings)?;
  let tenant_name = get_tenant_name(matches, settings)?;
  let guid = get_guid(matches, &platform, &tenant_name)?;
  Ok(DshApiTenant::new(tenant_name, guid, platform))
}

pub(crate) fn to_command_error_with_id(error: DshApiError, subject: &str, which: &str) -> DcliResult {
  match error {
    DshApiError::Configuration(message) => Err(message),
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => Err(format!("{} {} not found", subject, which)),
    DshApiError::Unexpected(error, _) => Err(format!("unexpected error, {}", error)),
  }
}

pub(crate) fn read_multi_line() -> Result<String, String> {
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

pub(crate) fn read_single_line(prompt: impl AsRef<str>) -> Result<String, String> {
  print!("{}", prompt.as_ref());
  let _ = stdout().lock().flush();
  let mut line = String::new();
  stdin().read_line(&mut line).expect("could not read line");
  Ok(line.trim().to_string())
}

pub(crate) fn read_single_line_password(prompt: impl AsRef<str>) -> Result<String, String> {
  print!("{}", prompt.as_ref());
  let mut stdout = stdout();
  let _ = stdout.flush();
  let mut stdin = stdin();
  match stdin.read_passwd(&mut stdout).map_err(|error| error.to_string())? {
    Some(line) => {
      let _ = stdout.write("\n".as_bytes());
      Ok(line.trim().to_string())
    }
    None => Err("empty input".to_string()),
  }
}

pub(crate) fn confirmed(prompt: impl AsRef<str>) -> Result<bool, String> {
  print!("{}", prompt.as_ref());
  let _ = stdout().lock().flush();
  let mut line = String::new();
  stdin().read_line(&mut line).expect("could not read line");
  Ok(line == *"yes\n")
}

pub(crate) fn include_app_application(matches: &ArgMatches) -> (bool, bool) {
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
    if env_var.starts_with("DSH_API_") {
      environment_variables.push((env_var, value));
    }
  }
  environment_variables.sort_by(|(env_var_a, _), (env_var_b, _)| env_var_a.cmp(env_var_b));
  environment_variables
}
