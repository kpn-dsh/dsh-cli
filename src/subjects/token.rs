use crate::capability::{Capability, CommandExecutor, COPY_COMMAND, FETCH_COMMAND};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use lazy_static::lazy_static;
use log::debug;

pub(crate) struct TokenSubject {}

const TOKEN_SUBJECT_TARGET: &str = "token";

lazy_static! {
  pub static ref TOKEN_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TokenSubject {});
}

#[async_trait]
impl Subject for TokenSubject {
  fn subject(&self) -> &'static str {
    TOKEN_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Request DSH tokens.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Request DSH tokens.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      COPY_COMMAND => Some(TOKEN_COPY_CAPABILITY.as_ref()),
      FETCH_COMMAND => Some(TOKEN_FETCH_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TOKEN_CAPABILITIES
  }
}

lazy_static! {
  static ref TOKEN_COPY_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(COPY_COMMAND, None, &TokenCopy {}, "Copy token to clipboard").set_long_about("Fetch a DSH API token and copy it to the clipboard."));
  static ref TOKEN_FETCH_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(FETCH_COMMAND, None, &TokenFetch {}, "Fetch token").set_long_about("Fetch a DSH API token."));
  static ref TOKEN_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![TOKEN_COPY_CAPABILITY.as_ref(), TOKEN_FETCH_CAPABILITY.as_ref()];
}

struct TokenCopy {}

#[async_trait]
impl CommandExecutor for TokenCopy {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("fetch dsh api token");
    let start_instant = context.now();
    let access_token = client.token_fetcher().fetch_access_token_from_server().await.map_err(|error| error.to_string())?;
    context.print_execution_time(start_instant);
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(access_token.access_token)) {
      Ok(_) => {
        let not_before = if access_token.not_before_policy > 0 { format!(", not before: {}", access_token.not_before_policy) } else { "".to_string() };
        let expires_in = if access_token.refresh_expires_in > 0 { format!(", expires in: {}", access_token.refresh_expires_in) } else { "".to_string() };
        context.print_outcome(format!(
          "token copied to clipboard (type: {}, expires: {}{}{})",
          access_token.token_type, access_token.expires_in, not_before, expires_in
        ))
      }
      Err(error) => {
        debug!("clipboard error {}", error);
        context.print_error("could not copy token to clipboard")
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TokenFetch {}

#[async_trait]
impl CommandExecutor for TokenFetch {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("fetch dsh api token");
    let start_instant = context.now();
    let access_token = client.token_fetcher().fetch_access_token_from_server().await.map_err(|error| error.to_string())?;
    context.print_execution_time(start_instant);
    context.print(access_token.access_token);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}
