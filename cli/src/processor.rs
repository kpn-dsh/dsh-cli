use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::subject::Subject;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

const TRIFONIUS_PIPELINE_NAME: &str = "TRIFONIUS_PIPELINE_NAME";
const TRIFONIUS_PROCESSOR_ID: &str = "TRIFONIUS_PROCESSOR_ID";
const TRIFONIUS_PROCESSOR_NAME: &str = "TRIFONIUS_PROCESSOR_NAME";
const TRIFONIUS_PROCESSOR_TYPE: &str = "TRIFONIUS_PROCESSOR_TYPE";
const TRIFONIUS_SERVICE_NAME: &str = "TRIFONIUS_SERVICE_NAME";

pub(crate) struct ProcessorSubject {}

const PROCESSOR_SUBJECT_TARGET: &str = "processor";

lazy_static! {
  pub static ref PROCESSOR_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ProcessorSubject {});
}

#[async_trait]
impl Subject for ProcessorSubject {
  fn subject(&self) -> &'static str {
    PROCESSOR_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Processor"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list Trifonius processors.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list Trifonius processors.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("p")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> {
    let mut capabilities: HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> = HashMap::new();
    capabilities.insert(CapabilityType::List, &PROCESSORS_LIST_CAPABILITY);
    capabilities
  }
}

lazy_static! {
  pub static ref PROCESSORS_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List Trifonius processors".to_string(),
    command_long_about: Some("Lists all available Trifonius processors.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::All, &ListAll {}, None),],
    default_command_executor: Some(&ListAll {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct ListAll {}

#[async_trait]
impl CommandExecutor for ListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = dsh_api_client.get_application_configurations().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for (application_id, application) in applications {
      if let Some(trifonius_parameters) = find_trifonius_parameters(&application) {
        let parameters = vec![
          application_id,
          trifonius_parameters.get(TRIFONIUS_PIPELINE_NAME).cloned().unwrap_or("-".to_string()),
          trifonius_parameters.get(TRIFONIUS_PROCESSOR_NAME).cloned().unwrap_or("-".to_string()),
          trifonius_parameters.get(TRIFONIUS_PROCESSOR_TYPE).cloned().unwrap_or("-".to_string()),
          trifonius_parameters.get(TRIFONIUS_PROCESSOR_ID).cloned().unwrap_or("-".to_string()),
          trifonius_parameters.get(TRIFONIUS_SERVICE_NAME).cloned().unwrap_or("-".to_string()),
          application.exposed_ports.keys().map(|k| k.to_string()).collect::<Vec<String>>().join(","),
          application.cpus.to_string(),
          application.mem.to_string(),
          application.instances.to_string(),
          application.user,
          application.metrics.clone().map(|m| format!("{}:{}", m.path, m.port)).unwrap_or_default(),
        ];
        table.push(parameters);
      }
    }
    for line in make_tabular_with_headers(
      &["application", "pipeline", "processor", "type", "processor id", "service name", "ports", "cpus", "mem", "#", "user", "metrics"],
      table,
    ) {
      println!("{}", line)
    }
    Ok(())
  }
}

fn find_trifonius_parameters(application: &Application) -> Option<HashMap<&'static str, String>> {
  let mut parameters: HashMap<&'static str, String> = HashMap::new();
  if let Some(pipeline_name) = application.env.get(TRIFONIUS_PIPELINE_NAME) {
    parameters.insert(TRIFONIUS_PIPELINE_NAME, pipeline_name.to_string());
  }
  if let Some(processor_name) = application.env.get(TRIFONIUS_PROCESSOR_ID) {
    parameters.insert(TRIFONIUS_PROCESSOR_ID, processor_name.to_string());
  }
  if let Some(processor_name) = application.env.get(TRIFONIUS_PROCESSOR_NAME) {
    parameters.insert(TRIFONIUS_PROCESSOR_NAME, processor_name.to_string());
  }
  if let Some(processor_name) = application.env.get(TRIFONIUS_PROCESSOR_TYPE) {
    parameters.insert(TRIFONIUS_PROCESSOR_TYPE, processor_name.to_string());
  }
  if let Some(service_name) = application.env.get(TRIFONIUS_SERVICE_NAME) {
    parameters.insert(TRIFONIUS_SERVICE_NAME, service_name.to_string());
  }
  if parameters.is_empty() {
    None
  } else {
    Some(parameters)
  }
}
