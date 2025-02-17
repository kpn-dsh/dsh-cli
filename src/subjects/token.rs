use crate::capability::{Capability, CommandExecutor, REQUEST_COMMAND, REQUEST_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_sdk::management_api::AccessToken;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

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

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(false, false, true, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      REQUEST_COMMAND => Some(TOKEN_REQUEST_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TOKEN_CAPABILITIES
  }
}

lazy_static! {
  static ref TOKEN_REQUEST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(REQUEST_COMMAND_PAIR, "Request token")
      .set_long_about("Request a DSH API token.")
      .set_default_command_executor(&TokenRequest {})
  );
  static ref TOKEN_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![TOKEN_REQUEST_CAPABILITY.as_ref()];
}

struct TokenRequest {}

#[async_trait]
impl CommandExecutor for TokenRequest {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("request dsh api token");
    let start_instant = Instant::now();

    let access_token = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .token_fetcher()
      .fetch_access_token_from_server()
      .await
      .map_err(|error| error.to_string())?;
    context.print_execution_time(start_instant);
    UnitFormatter::new("", &ACCES_TOKEN_LABELS, None, context).print_non_serializable(&access_token)
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
      AccessTokenLabel::AccessToken => self.access_token().to_string(),
      AccessTokenLabel::ExpiresIn => self.expires_in().to_string(),
      AccessTokenLabel::Formatted => self.formatted_token(),
      AccessTokenLabel::NotBeforePolicy => self.not_before_policy().to_string(),
      AccessTokenLabel::RefreshExpiresIn => self.refresh_expires_in().to_string(),
      AccessTokenLabel::Scope => self.scope().to_string(),
      AccessTokenLabel::TokenType => self.token_type().to_string(),
    }
  }

  fn target_label(&self) -> Option<AccessTokenLabel> {
    Some(AccessTokenLabel::Scope)
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

// use dsh_api::dsh_api_client_factory::DshApiClientFactory;
// use dsh_sdk::protocol_adapters::token::api_client_token_fetcher::ApiClientTokenFetcher;
// use dsh_sdk::protocol_adapters::token::data_access_token::RequestDataAccessToken;
// use dsh_sdk::Platform;
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//   let client = DshApiClientFactory::default().client().await?;
//
//   println!("{}", client.tenant());
//
//   // Create a request for the Data Access Token (this request for full access)
//   let request = RequestDataAccessToken::new(client.tenant_name(), client.platform().client_id());
//
//   let sdk_platform = Platform::try_from(client.platform())?;
//   let api_key = "";
//
//   let token_fetcher = ApiClientTokenFetcher::new(api_key, sdk_platform);
//
//   let token = token_fetcher.fetch_data_access_token(request).await.unwrap();
//
//   println!("{:#?}", token);
//
//   Ok(())
// }
