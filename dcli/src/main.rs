#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::{ExitCode, Termination};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::builder::styling;
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_client_factory::{get_secret_from_platform_and_tenant, DshApiClientFactory};
use dsh_api::dsh_api_tenant::{get_default_tenant_name, DshApiTenant};
use dsh_api::platform::DshPlatform;
use dsh_api::DshApiError;

use crate::app::APP_SUBJECT;
use crate::application::APPLICATION_SUBJECT;
use crate::arguments::{
  no_border_argument, platform_argument, set_verbosity_argument, tenant_argument, verbosity_argument, Verbosity, NO_BORDER_ARGUMENT, PLATFORM_ARGUMENT, SET_VERBOSITY_ARGUMENT,
  TENANT_ARGUMENT, VERBOSITY_ARGUMENT,
};
use crate::autocomplete::{generate_autocomplete_file, generate_autocomplete_file_argument, AutocompleteShell, AUTOCOMPLETE_ARGUMENT};
use crate::bucket::BUCKET_SUBJECT;
use crate::certificate::CERTIFICATE_SUBJECT;
use crate::env::ENV_SUBJECT;
use crate::filter_flags::FilterFlagType;
use crate::image::IMAGE_SUBJECT;
use crate::manifest::MANIFEST_SUBJECT;
use crate::metric::METRIC_SUBJECT;
use crate::proxy::PROXY_SUBJECT;
use crate::secret::SECRET_SUBJECT;
use crate::set::SET_SUBJECT;
#[cfg(feature = "stream")]
use crate::stream::STREAM_SUBJECT;
use crate::subject::{clap_list_shortcut_command, clap_subject_command, Subject};
use crate::topic::TOPIC_SUBJECT;
use crate::vhost::VHOST_SUBJECT;
use crate::volume::VOLUME_SUBJECT;

mod app;
mod application;
mod arguments;
mod autocomplete;
mod bucket;
mod capability;
mod certificate;
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
mod set;
mod settings;
#[cfg(feature = "stream")]
mod stream;
mod subject;
mod topic;
mod vhost;
mod volume;

pub(crate) static APPLICATION_NAME: &str = "dcli";

static ABOUT: &str = "DSH api command line interface";

static LONG_ABOUT: &str = "DSH api command line interface\n\n\
   The DSH api command line tool enables the user to call a subset of the functions \
   in the DSH api from the command line. \
   It also supports functions that are not supported directly from the DSH api, \
   such as finding all applications that use a certain resource (e.g. a secret) or showing a \
   list of all resources of a certain type (e.g. list all volumes).";

static AFTER_HELP: &str = "For most commands adding an 's' as a postfix will yield the same result \
   as using the 'list' subcommand, e.g. using 'dcli apps' will be the same \
   as using 'dcli app list'.";
static LONG_VERSION: &str = "version: 0.2.0\ndsh-api library version: 0.2.0\ndsh rest api version: 1.8.0";

type DcliResult = Result<bool, String>;

pub(crate) struct DcliContext {
  pub(crate) verbosity: Verbosity,
  pub(crate) border: bool,
}

impl DcliContext {
  pub(crate) fn show_capability_explanation(&self) -> bool {
    match self.verbosity {
      Verbosity::Off | Verbosity::Low => false,
      Verbosity::Medium | Verbosity::High => true,
    }
  }

  pub(crate) fn show_execution_time(&self) -> bool {
    match self.verbosity {
      Verbosity::Off | Verbosity::Low => false,
      Verbosity::Medium | Verbosity::High => true,
    }
  }

  pub(crate) fn show_headers(&self) -> bool {
    match self.verbosity {
      Verbosity::Off => false,
      Verbosity::Low | Verbosity::Medium | Verbosity::High => true,
    }
  }

  pub(crate) fn show_settings(&self) -> bool {
    match self.verbosity {
      Verbosity::Off | Verbosity::Low => false,
      Verbosity::Medium | Verbosity::High => true,
    }
  }
}

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
  let halted = Arc::new(AtomicBool::new(false));
  let h = halted.clone();
  let _ = ctrlc::set_handler(move || {
    h.store(true, Ordering::SeqCst);
  });

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
    SET_SUBJECT.as_ref(),
    #[cfg(feature = "stream")]
    STREAM_SUBJECT.as_ref(),
    TOPIC_SUBJECT.as_ref(),
    VHOST_SUBJECT.as_ref(),
    VOLUME_SUBJECT.as_ref(),
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
    .long_about(LONG_ABOUT)
    .after_help(AFTER_HELP)
    .args(vec![
      no_border_argument(),
      platform_argument(),
      set_verbosity_argument(),
      tenant_argument(),
      verbosity_argument(),
      generate_autocomplete_file_argument(),
    ])
    .arg_required_else_help(true)
    .term_width(80)
    .disable_help_subcommand(true)
    .max_term_width(100)
    .hide_possible_values(true)
    .styles(styles)
    .subcommands(clap_commands)
    .version("0.2.0")
    .long_version(LONG_VERSION);

  let matches = command.clone().get_matches();

  if let Some(shell) = matches.get_one::<AutocompleteShell>(AUTOCOMPLETE_ARGUMENT) {
    generate_autocomplete_file(&mut command, shell);
    return Ok(false);
  }

  let border = !matches.get_flag(NO_BORDER_ARGUMENT);

  let verbosity: Verbosity = matches
    .get_one::<Verbosity>(SET_VERBOSITY_ARGUMENT)
    .cloned()
    .unwrap_or(match matches.get_one::<u8>(VERBOSITY_ARGUMENT) {
      Some(verbosity) => match verbosity {
        0 => Verbosity::Low,
        1 => Verbosity::Low,
        2 => Verbosity::Medium,
        _ => Verbosity::High,
      },
      None => Verbosity::Low,
    });

  let context = DcliContext { verbosity, border };

  let tenant_name = matches
    .get_one::<String>(TENANT_ARGUMENT)
    .map(|a| a.to_string())
    .unwrap_or(get_default_tenant_name()?);
  let platform = match matches.get_one::<String>(PLATFORM_ARGUMENT) {
    Some(platform_name) => DshPlatform::try_from(platform_name.as_str())?,
    None => DshPlatform::try_from(())?,
  };
  let secret = get_secret_from_platform_and_tenant(platform.to_string().as_str(), tenant_name.as_str())?;
  let dsh_api_tenant = DshApiTenant::from_tenant_and_platform(tenant_name.clone(), platform.clone())?;
  let dsh_api_client_factory = DshApiClientFactory::create(dsh_api_tenant, secret)?;
  let dsh_api_client = dsh_api_client_factory.client().await?;

  if context.show_settings() {
    println!("tenant {}@{}", tenant_name, platform);
  }

  let start_instant = Instant::now();

  let suppress_show_execution_time = match matches.subcommand() {
    Some((command_name, sub_matches)) => match subject_registry.get(command_name) {
      Some(subject) => subject.execute_subject_command(sub_matches, &context, &dsh_api_client).await?,
      None => match subject_list_shortcut_registry.get(command_name) {
        Some(subject) => subject.execute_subject_list_shortcut(sub_matches, &context, &dsh_api_client).await?,
        None => return Err("unexpected error, list shortcut not found".to_string()),
      },
    },
    None => return Err("unexpected error, no subcommand".to_string()),
  };

  if !suppress_show_execution_time && context.show_execution_time() {
    println!("execution took {} milliseconds", Instant::now().duration_since(start_instant).as_millis());
  }
  Ok(false)
}

pub(crate) fn to_command_error_with_id(error: DshApiError, subject: &str, which: &str) -> DcliResult {
  match error {
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => Err(format!("{} {} not found", subject, which)),
    DshApiError::Unexpected(error) => Err(format!("unexpected error, {}", error)),
  }
}

pub(crate) fn read_multi_line() -> Result<String, String> {
  let mut multi_line = String::new();
  let stdin = stdin();
  loop {
    match stdin.lock().read_line(&mut multi_line) {
      Ok(0) => break,
      Ok(_) => continue,
      Err(_) => return Err("error reading line".to_string()),
    }
  }
  Ok(multi_line)
}

pub(crate) fn read_single_line(prompt: &str) -> Result<String, String> {
  print!("{}", prompt);
  let _ = stdout().lock().flush();
  let mut line = String::new();
  stdin().lock().read_line(&mut line).expect("could not read line");
  Ok(line.trim().to_string())
}

pub(crate) fn confirmed(prompt: &str) -> Result<bool, String> {
  print!("{}", prompt);
  let _ = stdout().lock().flush();
  let mut line = String::new();
  stdin().lock().read_line(&mut line).expect("could not read line");
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
