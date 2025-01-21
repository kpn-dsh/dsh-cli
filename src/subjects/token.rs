use crate::capability::{Capability, CommandExecutor, REQUEST_COMMAND, REQUEST_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::subject::Subject;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
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

  fn requires_dsh_api_client(&self) -> bool {
    true
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

    let rest_api_token = context.dsh_api_client.as_ref().unwrap().token().await?;
    context.print_execution_time(start_instant);

    if let Some(token) = rest_api_token.strip_prefix("Bearer ") {
      // match jsonwebtoken::decode_header(token) {
      //   Ok(header) => println!("header {:?}", header),
      //   Err(error) => return Err(error.to_string()),
      // }
      // let token_data = jsonwebtoken::decode::<String>(&token, &DecodingKey::from_secret(token.as_ref()), &Validation::new(Algorithm::RS256));
      // println!("{:?}", token_data);
      context.print(token);
    }
    Ok(())
  }
}
