use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::subject::Subject;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub(crate) struct ApiSubject {}

const API_SUBJECT_TARGET: &str = "api";

lazy_static! {
  pub static ref API_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ApiSubject {});
}

#[async_trait]
impl Subject for ApiSubject {
  fn subject(&self) -> &'static str {
    API_SUBJECT_TARGET
  }

  /// Help text printed for -h flag
  fn subject_command_about(&self) -> String {
    "List and call DSH resource management api.".to_string()
  }

  /// Help text printed for --help flag
  fn subject_command_long_about(&self) -> String {
    "List and call DSH resource management api.".to_string()
  }

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Show, API_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref API_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(CapabilityType::Show, "Print the open api specification.").set_default_command_executor(&ApiShow {}));
}

struct ApiShow {}

#[async_trait]
impl CommandExecutor for ApiShow {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("print the open api specification");
    context.print(DshApiClient::openapi_spec());
    Ok(())
  }
}
