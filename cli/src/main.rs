#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::builder::styling;
use clap::{builder, Arg, ArgAction, Command};

use trifonius_dsh_api::DEFAULT_DSH_API_CLIENT_FACTORY;

use crate::app::{app_command, run_app_command, APP_COMMAND};
use crate::application::{application_command, run_application_command, APPLICATION_COMMAND};
use crate::processor::{processor_command, run_processor_command, PROCESSOR_COMMAND};
use crate::secret::{run_secret_command, secret_command, SECRET_COMMAND};
use crate::task::{run_task_command, task_command, TASK_COMMAND};
use crate::vhost::{run_vhost_command, vhost_command, VHOST_COMMAND};

mod app;
mod application;
mod processor;
mod secret;
pub(crate) mod tabular;
mod task;
mod vhost;

static ABOUT: &str = "Trifonius command line interface";
static LONG_ABOUT: &str = "Trifonius command line interface, enables listing, deploying, undeploying and managing DSH components controlled by Trifonius.";

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
    .args(arguments())
    .arg_required_else_help(true)
    .term_width(80)
    .disable_help_subcommand(true)
    .max_term_width(100)
    .hide_possible_values(true)
    .styles(styles)
    .subcommands([app_command(), application_command(), processor_command(), secret_command(), task_command(), vhost_command()])
    .version("0.0.6")
    .long_version("version: 0.0.6\ntrifonius version: 0.0.6\ndsh api version: 1.7.0");

  let matches = command.get_matches();

  let dsh_api_client = &DEFAULT_DSH_API_CLIENT_FACTORY.client().await.expect("unable to create dsh api client");

  match matches.subcommand() {
    Some((APP_COMMAND, sub_matches)) => run_app_command(sub_matches, dsh_api_client).await,
    Some((APPLICATION_COMMAND, sub_matches)) => run_application_command(sub_matches, dsh_api_client).await,
    Some((PROCESSOR_COMMAND, sub_matches)) => run_processor_command(sub_matches, dsh_api_client).await,
    Some((SECRET_COMMAND, sub_matches)) => run_secret_command(sub_matches, dsh_api_client).await,
    Some((TASK_COMMAND, sub_matches)) => run_task_command(sub_matches, dsh_api_client).await,
    Some((VHOST_COMMAND, sub_matches)) => run_vhost_command(sub_matches, dsh_api_client).await,
    Some((_command, _sub_matches)) => {}
    None => (),
  }
}

pub(crate) const TENANT_ARGUMENT: &str = "tenant-argument";

pub(crate) fn arguments() -> Vec<Arg> {
  vec![Arg::new(TENANT_ARGUMENT)
    .long("tenant")
    .short('t')
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Target tenant")
    .long_help("Target tenant name.")]
}
