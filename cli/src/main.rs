#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::builder::styling;
use clap::Command;

use trifonius_dsh_api::dsh_api_client_factory::DshApiClientFactory;
use trifonius_dsh_api::DshApiError;

use crate::app::APP_SUBJECT;
use crate::application::APPLICATION_SUBJECT;
use crate::arguments::{set_verbosity_argument, verbosity_argument};
use crate::bucket::BUCKET_SUBJECT;
use crate::env::ENV_SUBJECT;
use crate::manifest::MANIFEST_SUBJECT;
use crate::processor::PROCESSOR_SUBJECT;
use crate::secret::SECRET_SUBJECT;
use crate::subject::{clap_subject_command, clap_subject_list_shortcut, Subject};
use crate::topic::TOPIC_SUBJECT;
use crate::vhost::VHOST_SUBJECT;

mod app;
mod application;
mod arguments;
mod bucket;
mod capability;
mod env;
mod flags;
mod formatters;
mod manifest;
mod processor;
mod secret;
mod subject;
mod tabular;
mod topic;
mod vhost;

static ABOUT: &str = "Trifonius command line interface";
static LONG_ABOUT: &str = "Trifonius command line interface, enables listing, deploying, undeploying and managing DSH components controlled by Trifonius.";

type CommandResult = Result<(), String>;

#[tokio::main]
async fn main() {
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
    ENV_SUBJECT.as_ref(),
    MANIFEST_SUBJECT.as_ref(),
    PROCESSOR_SUBJECT.as_ref(),
    SECRET_SUBJECT.as_ref(),
    TOPIC_SUBJECT.as_ref(),
    VHOST_SUBJECT.as_ref(),
  ];

  let mut subject_registry: HashMap<String, &(dyn Subject + Send + Sync)> = HashMap::new();
  let mut clap_commands: Vec<Command> = Vec::new();

  let mut subject_list_shortcut_registry: HashMap<String, &(dyn Subject + Send + Sync)> = HashMap::new();

  for subject in subjects {
    let (command_name, clap_command) = clap_subject_command(subject);
    subject_registry.insert(command_name.to_string(), subject);
    clap_commands.push(clap_command);
    if let Some((list_shortcut_name, clap_list_shortcut)) = clap_subject_list_shortcut(subject) {
      subject_list_shortcut_registry.insert(list_shortcut_name.to_string(), subject);
      clap_commands.push(clap_list_shortcut);
    }
  }

  let command = Command::new("tcli")
    .about(ABOUT)
    .long_about(LONG_ABOUT)
    .after_help("For most commands adding an 's' will yield the same result as using the 'list' subcommand, e.g. using 'tcli apps' will be the same as using 'tcli app list'.")
    .before_help("before help")
    .args(vec![set_verbosity_argument(), verbosity_argument()])
    .arg_required_else_help(true)
    .term_width(80)
    .disable_help_subcommand(true)
    .max_term_width(100)
    .hide_possible_values(true)
    .styles(styles)
    .subcommands(clap_commands)
    .version("0.0.6")
    .long_version("version: 0.0.6\ntrifonius version: 0.0.6\ndsh api version: 1.7.0");

  let matches = command.get_matches();

  let dsh_api_client_factory = DshApiClientFactory::default();
  let dsh_api_client = dsh_api_client_factory.client().await.expect("unable to create dsh api client");

  let command_result = match matches.subcommand() {
    Some((command_name, sub_matches)) => match subject_registry.get(command_name) {
      Some(subject) => subject.execute_subject_command(sub_matches, &dsh_api_client).await,
      None => match subject_list_shortcut_registry.get(command_name) {
        Some(subject) => subject.execute_subject_list_shortcut(sub_matches, &dsh_api_client).await,
        None => Err("unexpected error, list shortcut not found".to_string()),
      },
    },
    None => Err("unexpected error, no subcommand".to_string()),
  };
  if let Err(message) = command_result {
    println!("{}", message);
  }
}

pub(crate) fn to_command_error_with_id(error: DshApiError, subject: &str, which: &str) -> CommandResult {
  match error {
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => Err(format!("{} {} not found", subject, which)),
    DshApiError::Unexpected(error) => Err(format!("unexpected error, {}", error)),
  }
}
