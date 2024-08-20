#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::builder::styling;
use clap::Command;

use trifonius_dsh_api::{DshApiClient, DshApiError};

use crate::app::APP_COMMAND;
use crate::application::APPLICATION_COMMAND;
use crate::arguments::{set_verbosity_argument, verbosity_argument};
use crate::bucket::BUCKET_COMMAND;
use crate::command::SubjectCommand;
use crate::env::ENV_COMMAND;
use crate::processor::PROCESSOR_COMMAND;
use crate::secret::SECRET_COMMAND;
use crate::topic::TOPIC_COMMAND;
use crate::vhost::VHOST_COMMAND;

mod app;
mod application;
mod arguments;
mod bucket;
mod command;
mod def_impl;
mod env;
mod formatters;
mod processor;
mod secret;
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

  let subject_commands: Vec<&Box<dyn SubjectCommand + Send + Sync>> =
    vec![&APP_COMMAND, &APPLICATION_COMMAND, &BUCKET_COMMAND, &ENV_COMMAND, &PROCESSOR_COMMAND, &SECRET_COMMAND, &TOPIC_COMMAND, &VHOST_COMMAND];

  let mut subject_command_registry: HashMap<String, &Box<dyn SubjectCommand + Send + Sync>> = HashMap::new();
  let mut subject_command_shortcut_registry: HashMap<String, &Box<dyn SubjectCommand + Send + Sync>> = HashMap::new();

  let mut clap_commands: Vec<Command> = Vec::new();

  for subject_command in subject_commands {
    let (command_name, clap_command) = subject_command.create_command();
    subject_command_registry.insert(command_name.to_string(), subject_command);
    clap_commands.push(clap_command);
    if let Some((list_shortcut_command_name, list_shortcut_clap_command)) = subject_command.create_list_shortcut_command() {
      subject_command_shortcut_registry.insert(list_shortcut_command_name.to_string(), subject_command);
      clap_commands.push(list_shortcut_clap_command);
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

  let dsh_api_client = DshApiClient::default().await;

  let command_result = match matches.subcommand() {
    Some((command_name, sub_matches)) => match subject_command_registry.get(command_name) {
      Some(subject_command) => subject_command.run_command(sub_matches, &dsh_api_client).await,
      None => match subject_command_shortcut_registry.get(command_name) {
        Some(subject_command) => subject_command.run_list_shortcut(sub_matches, &dsh_api_client).await,
        None => Err("unexpected error".to_string()),
      },
    },
    None => Err("unexpected error".to_string()),
  };
  if let Err(message) = command_result {
    println!("{}", message);
  }
}

pub(crate) fn _to_command_error(error: DshApiError, subject_command: &dyn SubjectCommand) -> CommandResult {
  match error {
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => Err(format!("{} not found", subject_command.subject())),
    DshApiError::Unexpected(error) => Err(format!("unexpected error {}", error)),
  }
}

pub(crate) fn to_command_error_with_id(error: DshApiError, subject_command: &dyn SubjectCommand, which: &str) -> CommandResult {
  match error {
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => Err(format!("{} {} not found", subject_command.subject(), which)),
    DshApiError::Unexpected(error) => Err(format!("unexpected error {}", error)),
  }
}

pub(crate) fn to_command_error_missing_id(subject_command: &dyn SubjectCommand) -> CommandResult {
  Err(format!("missing {} id", subject_command.subject()))
}
