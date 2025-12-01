use crate::capability::{Capability, CommandExecutor, COPY_COMMAND, FETCH_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::OutputFormat;
use crate::formatters::{Label, SubjectFormatter};
use crate::subject::{Requirements, Subject};
use crate::{error_map, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use chrono::Local;
use chrono::LocalResult::{Ambiguous, Single};
use chrono::TimeZone;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use serde::Serialize;
use serde_json::Value;

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
    let jwt = client.fresh_jwt().await.map_err(error_map!("could not retrieve token: {}"))?;
    context.print_execution_time(start_instant);
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(jwt.token().secret())) {
      Ok(_) => {
        let jwt_type = jwt.payload().token_type.clone().unwrap_or("unknown".to_string());
        let not_before = &jwt
          .payload()
          .not_before
          .map(|nbf| if nbf > 0 { format!(", not before: {}", nbf) } else { "".to_string() })
          .unwrap_or_default();
        let expires_in = if jwt.payload().expires_in() > 0 { format!(", expires in: {}", jwt.payload().expires_in()) } else { "".to_string() };
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
    let jwt = client.fresh_jwt().await.map_err(error_map!("could not retrieve token: {}"))?;
    context.print_execution_time(start_instant);
    context.print(jwt.token().secret());
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TokenShow {}

#[async_trait]
impl CommandExecutor for TokenShow {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let complete = matches.get_flag(FilterFlagType::Complete.id());
    let start_instant = context.now();
    let jwt = client.fresh_jwt().await.map_err(error_map!("could not retrieve token: {}"))?;
    context.print_execution_time(start_instant);
    if complete {
      context.print_explanation("dsh api token header");
      print_content("header", context, jwt.header_bytes(), &TOKEN_HEADER_LABELS_LIST);
    }
    context.print_explanation("dsh api token payload");
    print_content("payload", context, jwt.payload_bytes(), &TOKEN_PAYLOAD_LABELS_LIST);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn print_content(kind: &str, context: &Context, decoded_payload: Vec<u8>, labels: &[TokenLabel]) {
  if let Ok(json_payload) = String::from_utf8(decoded_payload) {
    match serde_json::from_str::<Value>(&json_payload) {
      Ok(deserialized_payload) => match context.output_format(None) {
        OutputFormat::Json => context.print_serializable(deserialized_payload, Some(OutputFormat::Json)),
        OutputFormat::JsonCompact => context.print_serializable(deserialized_payload, Some(OutputFormat::Json)),
        _ => {
          if let Some(payload_object) = deserialized_payload.as_object() {
            let mut formatter = ListFormatter::new(labels, None, context);
            let mut vals: Vec<(String, &Value)> = Vec::from_iter(payload_object.iter().map(|(key, value)| (key.to_string(), value)));
            vals.sort_by(|(key_a, _), (key_b, _)| key_a.cmp(key_b));
            for (key, value) in vals {
              formatter.push_target_id_value(key, value);
            }
            let _ = formatter.print(None);
          }
        }
      },
      Err(_) => context.print_error(format!("{} could not be parsed as valid json", kind)),
    }
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum TokenLabel {
  Key,
  Timestamp,
  Value,
}

impl Label for TokenLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Key => "key",
      Self::Timestamp => "timestamp",
      Self::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Key)
  }
}

impl SubjectFormatter<TokenLabel> for Value {
  fn value(&self, label: &TokenLabel, target_id: &str) -> String {
    match label {
      TokenLabel::Key => target_id.to_string(),
      TokenLabel::Timestamp => timestamp(self, target_id),
      TokenLabel::Value => match self {
        Value::Array(array) => array
          .iter()
          .map(|value| match value {
            Value::Array(_) => "array (unexpected)".to_string(),
            Value::Bool(boolean) => boolean.to_string(),
            Value::Null => "null".to_string(),
            Value::Number(number) => number.to_string(),
            Value::Object(_) => "object (unexpected)".to_string(),
            Value::String(string) => string.to_string(),
          })
          .collect_vec()
          .join("\n"),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Null => "null".to_string(),
        Value::Number(number) => number.to_string(),
        Value::Object(_) => "object (unexpected)".to_string(),
        Value::String(string) => string.to_string(),
      },
    }
  }
}

fn timestamp(value: &Value, target_id: &str) -> String {
  if ["auth_time", "exp", "iat", "nbf"].contains(&target_id) {
    match value
      .as_number()
      .and_then(|number| number.as_i64())
      .map(|valid_number| Local.timestamp_opt(valid_number, 0))
    {
      Some(mapped_local_time) => match mapped_local_time {
        Single(single) => single.to_string(),
        Ambiguous(ambiguous, _) => ambiguous.to_string(),
        _ => value.to_string(),
      },
      None => "".to_string(),
    }
  } else {
    "".to_string()
  }
}

static TOKEN_HEADER_LABELS_LIST: [TokenLabel; 2] = [TokenLabel::Key, TokenLabel::Value];
static TOKEN_PAYLOAD_LABELS_LIST: [TokenLabel; 3] = [TokenLabel::Key, TokenLabel::Value, TokenLabel::Timestamp];
