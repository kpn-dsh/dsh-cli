#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::builder::styling;
use clap::Command;

use trifonius_dsh_api::{DshApiError, DEFAULT_DSH_API_CLIENT_FACTORY};

use crate::app::{app_command, apps_command, run_app_command, run_apps_command, APPS_COMMAND, APP_COMMAND};
use crate::application::{application_command, applications_command, run_application_command, run_applications_command, APPLICATIONS_COMMAND, APPLICATION_COMMAND};
use crate::arguments::{set_verbosity_argument, verbosity_argument};
use crate::bucket::{bucket_command, buckets_command, run_bucket_command, run_buckets_command, BUCKETS_COMMAND, BUCKET_COMMAND};
use crate::processor::{processor_command, run_processor_command, PROCESSOR_COMMAND};
use crate::secret::{run_secret_command, run_secrets_command, secret_command, secrets_command, SECRETS_COMMAND, SECRET_COMMAND};
use crate::task::{run_task_command, task_command, TASK_COMMAND};
use crate::vhost::{run_vhost_command, vhost_command, VHOST_COMMAND};

mod app;
mod application;
mod arguments;
mod bucket;
mod formatters;
mod processor;
mod secret;
mod subcommands;
mod tabular;
mod task;
mod vhost;

static ABOUT: &str = "Trifonius command line interface";
static LONG_ABOUT: &str = "Trifonius command line interface, enables listing, deploying, undeploying and managing DSH components controlled by Trifonius.";

type CommandResult = Result<(), String>;

pub(crate) fn to_command_error_with_id(error: DshApiError, what: &str, which: &str) -> CommandResult {
  match error {
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => Err(format!("{} {} not found", what, which)),
    DshApiError::Unexpected(error) => Err(format!("unexpected error {}", error)),
  }
}

pub(crate) fn to_command_error(error: DshApiError) -> CommandResult {
  match error {
    DshApiError::NotAuthorized => Err("not authorized".to_string()),
    DshApiError::NotFound => unreachable!(),
    DshApiError::Unexpected(error) => Err(format!("unexpected error {}", error)),
  }
}

pub(crate) fn to_command_error_missing_id(what: &str) -> CommandResult {
  Err(format!("missing {} id", what))
}

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
    .subcommands([
      app_command(),
      apps_command(),
      application_command(),
      applications_command(),
      bucket_command(),
      buckets_command(),
      processor_command(),
      secret_command(),
      secrets_command(),
      task_command(),
      vhost_command(),
    ])
    .version("0.0.6")
    .long_version("version: 0.0.6\ntrifonius version: 0.0.6\ndsh api version: 1.7.0");

  let matches = command.get_matches();

  let dsh_api_client = &DEFAULT_DSH_API_CLIENT_FACTORY.client().await.expect("unable to create dsh api client");

  let command_result = match matches.subcommand() {
    Some((APP_COMMAND, sub_matches)) => run_app_command(sub_matches, dsh_api_client).await,
    Some((APPS_COMMAND, sub_matches)) => run_apps_command(sub_matches, dsh_api_client).await,
    Some((APPLICATION_COMMAND, sub_matches)) => run_application_command(sub_matches, dsh_api_client).await,
    Some((APPLICATIONS_COMMAND, sub_matches)) => run_applications_command(sub_matches, dsh_api_client).await,
    Some((BUCKET_COMMAND, sub_matches)) => run_bucket_command(sub_matches, dsh_api_client).await,
    Some((BUCKETS_COMMAND, sub_matches)) => run_buckets_command(sub_matches, dsh_api_client).await,
    Some((PROCESSOR_COMMAND, sub_matches)) => run_processor_command(sub_matches, dsh_api_client).await,
    Some((SECRET_COMMAND, sub_matches)) => run_secret_command(sub_matches, dsh_api_client).await,
    Some((SECRETS_COMMAND, sub_matches)) => run_secrets_command(sub_matches, dsh_api_client).await,
    Some((TASK_COMMAND, sub_matches)) => run_task_command(sub_matches, dsh_api_client).await,
    Some((VHOST_COMMAND, sub_matches)) => run_vhost_command(sub_matches, dsh_api_client).await,
    Some((command, _sub_matches)) => Err(format!("command {} is not recognized", command)),
    None => Err("unexpected error".to_string()),
  };

  if let Err(message) = command_result {
    println!("{}", message);
  }
}
