use crate::authentication::get_stored_refresh_token;
use crate::capability::{Capability, CommandExecutor, COPY_COMMAND, FETCH_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::Value as FormatterValue;
use crate::formatters::Value;
use crate::formatters::{Label, SubjectFormatter};
use crate::subject::{Requirements, Subject};
use crate::{error_map, get_target_platform, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::dsh_jwt::{DshJwt, DshJwtHeader, DshJwtPayload};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use serde::Serialize;
use std::str::FromStr;

struct TokenSubject {}

const TOKEN_SUBJECT_TARGET: &str = "token";

lazy_static! {
  pub(crate) static ref TOKEN_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TokenSubject {});
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
      SHOW_COMMAND => Some(TOKEN_SHOW_CAPABILITY.as_ref()),
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
  static ref TOKEN_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &TokenShow {}, "Show token payload")
      .set_long_about("Fetch a DSH API token and show its payload contents.")
      .add_filter_flag(FilterFlagType::Complete, Some("Include header contents.".to_string()))
      .add_command_executor(FlagType::OpenId, &TokenShowOpenId {}, Some("Show openid token".to_string()))
  );
  static ref TOKEN_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TOKEN_COPY_CAPABILITY.as_ref(), TOKEN_FETCH_CAPABILITY.as_ref(), TOKEN_SHOW_CAPABILITY.as_ref()];
}

struct TokenCopy {}

#[async_trait]
impl CommandExecutor for TokenCopy {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("fetch dsh api token and copy to clipboard");
    let start_instant = context.now();
    let jwt = client.jwt().await.map_err(error_map!("could not retrieve token: {}"))?;
    let access_token = client.raw_token().await.map_err(error_map!("could not retrieve token: {}"))?;
    context.print_execution_time(start_instant);
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(access_token)) {
      Ok(_) => {
        let jwt_type = jwt.payload.token_type.clone().unwrap_or("unknown".to_string());
        let not_before = &jwt
          .payload
          .not_before
          .map(|nbf| if nbf > 0 { format!(", not before: {}", nbf) } else { "".to_string() })
          .unwrap_or_default();
        let expires_in = jwt
          .payload
          .expires_in()
          .map(|expires_in| if expires_in > 0 { format!(", expires in: {}", expires_in) } else { "".to_string() })
          .unwrap_or_default();
        context.print_outcome(format!("token copied to clipboard (type: {}, expires: {}{})", jwt_type, not_before, expires_in))
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("fetch dsh api token");
    let start_instant = context.now();
    let raw_token = client.raw_token().await.map_err(error_map!("could not retrieve token: {}"))?;
    context.print_execution_time(start_instant);
    context.println(raw_token);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static TOKEN_HEADER_LABELS_LIST: [TokenLabel; 3] = [TokenLabel::HeaderTyp, TokenLabel::HeaderAlg, TokenLabel::HeaderKid];
static TOKEN_PAYLOAD_LABELS_LIST_DSH: [TokenLabel; 17] = [
  TokenLabel::DshAuthenticatedTenants,
  TokenLabel::DshAuthenticationTime,
  TokenLabel::DshAuthorizedParty,
  TokenLabel::DshClientAddress,
  TokenLabel::DshClientHost,
  TokenLabel::DshClientId,
  TokenLabel::DshEmail,
  TokenLabel::DshEmailVerified,
  TokenLabel::DshExpiresIn,
  TokenLabel::DshFamilyName,
  TokenLabel::DshGivenName,
  TokenLabel::DshName,
  TokenLabel::DshPermissions,
  TokenLabel::DshPreferredUsername,
  TokenLabel::DshScope,
  TokenLabel::DshSessionId,
  TokenLabel::DshTokenType,
];
static TOKEN_PAYLOAD_LABELS_LIST_RFC7519: [TokenLabel; 7] =
  [TokenLabel::Rfc7519Aud, TokenLabel::Rfc7519Exp, TokenLabel::Rfc7519Iat, TokenLabel::Rfc7519Iss, TokenLabel::Rfc7519Jti, TokenLabel::Rfc7519Nbf, TokenLabel::Rfc7519Sub];
static TOKEN_PAYLOAD_LABELS_LIST_CONCISE: [TokenLabel; 14] = [
  TokenLabel::Rfc7519Aud,
  TokenLabel::Rfc7519Exp,
  TokenLabel::Rfc7519Iat,
  TokenLabel::Rfc7519Iss,
  TokenLabel::Rfc7519Jti,
  TokenLabel::Rfc7519Nbf,
  TokenLabel::Rfc7519Sub,
  TokenLabel::DshClientId,
  TokenLabel::DshAuthenticatedTenants,
  TokenLabel::DshAuthenticationTime,
  TokenLabel::DshAuthorizedParty,
  TokenLabel::DshExpiresIn,
  TokenLabel::DshScope,
  TokenLabel::DshTokenType,
];

struct TokenShow {}

#[async_trait]
impl CommandExecutor for TokenShow {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let complete = matches.get_flag(FilterFlagType::Complete.id());
    let start_instant = context.now();
    let jwt = client.jwt().await.map_err(error_map!("could not retrieve token: {}"))?;
    context.print_execution_time(start_instant);
    if complete {
      context.print_explanation("dsh api token header");
      let formatter = UnitFormatter::new("", &TOKEN_HEADER_LABELS_LIST, context);
      formatter.print(&jwt, None)?;
      context.print_explanation("dsh api token rfc7519");
      let formatter = UnitFormatter::new("", &TOKEN_PAYLOAD_LABELS_LIST_RFC7519, context);
      formatter.print(&jwt, None)?;
      context.print_explanation("dsh api token dsh specific");
      let formatter = UnitFormatter::new("", &TOKEN_PAYLOAD_LABELS_LIST_DSH, context);
      formatter.print(&jwt, None)?;
    } else {
      context.print_explanation("dsh api token");
      let formatter = UnitFormatter::new("", &TOKEN_PAYLOAD_LABELS_LIST_CONCISE, context);
      formatter.print(&jwt, None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TokenShowOpenId {}

#[async_trait]
impl CommandExecutor for TokenShowOpenId {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let complete = matches.get_flag(FilterFlagType::Complete.id());
    match get_stored_refresh_token(&platform)? {
      Some(stored_refresh_token) => {
        let refresh_jwt = DshJwt::from_str(stored_refresh_token.secret())?;
        if complete {
          context.print_explanation("openid refresh token header");
          let formatter = UnitFormatter::new("", &TOKEN_HEADER_LABELS_LIST, context);
          formatter.print(&refresh_jwt, None)?;
          context.print_explanation("openid refresh token rfc7519");
          let formatter = UnitFormatter::new("", &TOKEN_PAYLOAD_LABELS_LIST_RFC7519, context);
          formatter.print(&refresh_jwt, None)?;
          context.print_explanation("openid refresh token dsh specific");
          let formatter = UnitFormatter::new("", &TOKEN_PAYLOAD_LABELS_LIST_DSH, context);
          formatter.print(&refresh_jwt, None)?;
        } else {
          context.print_explanation("openid refresh token");
          let formatter = UnitFormatter::new("", &TOKEN_PAYLOAD_LABELS_LIST_CONCISE, context);
          formatter.print(&refresh_jwt, None)?;
        }
      }
      None => context.print_warning(format!("no refresh token for platform '{}'", platform)),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum TokenLabel {
  DshAuthenticatedTenants,
  DshAuthenticationTime,
  DshAuthorizedParty,
  DshClientAddress,
  DshClientHost,
  DshClientId,
  DshEmail,
  DshEmailVerified,
  DshExpiresIn,
  DshFamilyName,
  DshGivenName,
  DshName,
  DshPermissions,
  DshPreferredUsername,
  DshScope,
  DshSessionId,
  DshTokenType,
  HeaderAlg,
  HeaderKid,
  HeaderTyp,
  Rfc7519Aud,
  Rfc7519Exp,
  Rfc7519Iat,
  Rfc7519Iss,
  Rfc7519Jti,
  Rfc7519Nbf,
  Rfc7519Sub,
}

impl Label for TokenLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DshAuthenticatedTenants => "authenticated tenants",
      Self::DshAuthenticationTime => "authentication time",
      Self::DshAuthorizedParty => "authorized party",
      Self::DshClientAddress => "client address",
      Self::DshClientHost => "client host",
      Self::DshClientId => "client id",
      Self::DshEmail => "email",
      Self::DshEmailVerified => "email verified",
      Self::DshExpiresIn => "expires in",
      Self::DshFamilyName => "family name",
      Self::DshGivenName => "given name",
      Self::DshName => "name",
      Self::DshPermissions => "permissions",
      Self::DshPreferredUsername => "preferred username",
      Self::DshScope => "scope",
      Self::DshSessionId => "session id",
      Self::DshTokenType => "token type",
      Self::HeaderAlg => "alg",
      Self::HeaderKid => "kid",
      Self::HeaderTyp => "typ",
      Self::Rfc7519Aud => "aud",
      Self::Rfc7519Exp => "exp",
      Self::Rfc7519Iat => "iat",
      Self::Rfc7519Iss => "iss",
      Self::Rfc7519Jti => "jti",
      Self::Rfc7519Nbf => "nbf",
      Self::Rfc7519Sub => "sub",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

impl SubjectFormatter<TokenLabel> for DshJwt {
  fn value(&self, label: &TokenLabel, target_id: &str) -> FormatterValue {
    match label {
      TokenLabel::DshAuthenticatedTenants
      | TokenLabel::DshAuthenticationTime
      | TokenLabel::DshAuthorizedParty
      | TokenLabel::DshClientAddress
      | TokenLabel::DshClientHost
      | TokenLabel::DshClientId
      | TokenLabel::DshEmail
      | TokenLabel::DshEmailVerified
      | TokenLabel::DshExpiresIn
      | TokenLabel::DshFamilyName
      | TokenLabel::DshGivenName
      | TokenLabel::DshName
      | TokenLabel::DshPermissions
      | TokenLabel::DshPreferredUsername
      | TokenLabel::DshScope
      | TokenLabel::DshSessionId
      | TokenLabel::DshTokenType => self.payload.value(label, target_id),

      TokenLabel::HeaderAlg | TokenLabel::HeaderKid | TokenLabel::HeaderTyp => self.header.value(label, target_id),

      TokenLabel::Rfc7519Aud
      | TokenLabel::Rfc7519Exp
      | TokenLabel::Rfc7519Iat
      | TokenLabel::Rfc7519Iss
      | TokenLabel::Rfc7519Jti
      | TokenLabel::Rfc7519Nbf
      | TokenLabel::Rfc7519Sub => self.payload.value(label, target_id),
    }
  }
}

impl SubjectFormatter<TokenLabel> for DshJwtHeader {
  fn value(&self, label: &TokenLabel, _: &str) -> FormatterValue {
    match label {
      TokenLabel::HeaderAlg => Value::plain(&self.algorithm),
      TokenLabel::HeaderKid => Value::some_or_hide(self.kid.as_ref()),
      TokenLabel::HeaderTyp => Value::plain(&self.typ),
      _ => Value::not_applicable(),
    }
  }
}

impl SubjectFormatter<TokenLabel> for DshJwtPayload {
  fn value(&self, label: &TokenLabel, _: &str) -> FormatterValue {
    match label {
      TokenLabel::DshAuthenticatedTenants => Value::some_or_hide(self.authenticated_tenants().ok().map(|tenants| tenants.join(", "))),
      TokenLabel::DshAuthenticationTime => self.authentication_time.map(Value::timestamp_seconds).unwrap_or_default(),
      TokenLabel::DshAuthorizedParty => Value::some_or_hide(self.authorized_party.as_ref()),
      TokenLabel::DshClientAddress => Value::some_or_hide(self.client_address.as_ref()),
      TokenLabel::DshClientHost => Value::some_or_hide(self.client_host.as_ref()),
      TokenLabel::DshClientId => Value::some_or_hide(self.client_id.as_ref()),
      TokenLabel::DshEmail => Value::some_or_hide(self.email.as_ref()),
      TokenLabel::DshEmailVerified => Value::some_or_hide(self.email_verified.map(|verified| verified.to_string())),
      TokenLabel::DshExpiresIn => Value::some_or_hide(self.expires_in()),
      TokenLabel::DshFamilyName => Value::some_or_hide(self.family_name.as_ref()),
      TokenLabel::DshGivenName => Value::some_or_hide(self.given_name.as_ref()),
      TokenLabel::DshName => Value::some_or_hide(self.name.as_ref()),
      TokenLabel::DshPermissions => Value::some_or_hide(self.dsh_permission_representations.as_ref().map(|permissions| permissions.iter().join("\n"))),
      TokenLabel::DshPreferredUsername => Value::some_or_hide(self.preferred_username.as_ref()),
      TokenLabel::DshScope => Value::some_or_hide(self.scope.as_ref()),
      TokenLabel::DshSessionId => Value::some_or_hide(self.session_id.as_ref()),
      TokenLabel::DshTokenType => Value::some_or_hide(self.token_type.as_ref()),
      TokenLabel::Rfc7519Aud => Value::some_or_hide(self.audience.as_ref()),
      TokenLabel::Rfc7519Exp => self.expiration_time.map(Value::timestamp_seconds).unwrap_or_default(),
      TokenLabel::Rfc7519Iat => self.issued_at.map(Value::timestamp_seconds).unwrap_or_default(),
      TokenLabel::Rfc7519Iss => Value::some_or_hide(self.issuer.as_ref()),
      TokenLabel::Rfc7519Jti => Value::some_or_hide(self.jwt_id.as_ref()),
      TokenLabel::Rfc7519Nbf => self.not_before.map(Value::timestamp_seconds_not_before).unwrap_or_default(),
      TokenLabel::Rfc7519Sub => Value::some_or_hide(self.subject.as_ref()),
      _ => Value::not_applicable(),
    }
  }
}
