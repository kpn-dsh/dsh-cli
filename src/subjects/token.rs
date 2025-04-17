use crate::capability::{Capability, CommandExecutor, FETCH_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::token_fetcher::AccessToken;
use lazy_static::lazy_static;
use serde::Serialize;

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
      FETCH_COMMAND => Some(TOKEN_FETCH_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(TOKEN_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TOKEN_CAPABILITIES
  }
}

lazy_static! {
  static ref TOKEN_FETCH_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(FETCH_COMMAND, None, &TokenFetch {}, "Fetch token").set_long_about("Fetch a DSH API token."));
  static ref TOKEN_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &TokenShow {}, "Fetch and show token").set_long_about("Fetch a DSH API token and display its parameters.")
  );
  static ref TOKEN_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![TOKEN_FETCH_CAPABILITY.as_ref(), TOKEN_SHOW_CAPABILITY.as_ref()];
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

struct TokenShow {}

#[async_trait]
impl CommandExecutor for TokenShow {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("fetch dsh api token");
    let start_instant = context.now();

    let access_token = client.token_fetcher().fetch_access_token_from_server().await.map_err(|error| error.to_string())?;
    context.print_execution_time(start_instant);
    UnitFormatter::new("", &ACCES_TOKEN_LABELS, None, context).print_non_serializable(&access_token, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Clone, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum AccessTokenLabel {
  AccessToken,
  ExpiresIn,
  Formatted,
  NotBeforePolicy,
  RefreshExpiresIn,
  Scope,
  TokenType,
}

impl Label for AccessTokenLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AccessToken => "access token",
      Self::ExpiresIn => "expires in",
      Self::Formatted => "formatted",
      Self::NotBeforePolicy => "not before policy",
      Self::RefreshExpiresIn => "refresh expires in",
      Self::Scope => "scope",
      Self::TokenType => "type",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Formatted)
  }
}

impl SubjectFormatter<AccessTokenLabel> for AccessToken {
  fn value(&self, label: &AccessTokenLabel, _target_id: &str) -> String {
    match label {
      AccessTokenLabel::AccessToken => self.access_token.to_string(),
      AccessTokenLabel::ExpiresIn => self.expires_in.to_string(),
      AccessTokenLabel::Formatted => self.formatted_token(),
      AccessTokenLabel::NotBeforePolicy => self.not_before_policy.to_string(),
      AccessTokenLabel::RefreshExpiresIn => self.refresh_expires_in.to_string(),
      AccessTokenLabel::Scope => self.scope.to_string(),
      AccessTokenLabel::TokenType => self.token_type.to_string(),
    }
  }
}

pub static ACCES_TOKEN_LABELS: [AccessTokenLabel; 7] = [
  AccessTokenLabel::Formatted,
  AccessTokenLabel::Scope,
  AccessTokenLabel::TokenType,
  AccessTokenLabel::ExpiresIn,
  AccessTokenLabel::NotBeforePolicy,
  AccessTokenLabel::RefreshExpiresIn,
  AccessTokenLabel::AccessToken,
];
