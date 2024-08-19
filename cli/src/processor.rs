use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::arguments::Flag;
use crate::command::SubjectCommand;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

const TRIFONIUS_PIPELINE_NAME: &str = "TRIFONIUS_PIPELINE_NAME";
const TRIFONIUS_PROCESSOR_ID: &str = "TRIFONIUS_PROCESSOR_ID";
const TRIFONIUS_PROCESSOR_NAME: &str = "TRIFONIUS_PROCESSOR_NAME";
const TRIFONIUS_PROCESSOR_TYPE: &str = "TRIFONIUS_PROCESSOR_TYPE";
const TRIFONIUS_SERVICE_NAME: &str = "TRIFONIUS_SERVICE_NAME";

pub(crate) struct ProcessorCommand {}

lazy_static! {
  pub static ref PROCESSOR_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(ProcessorCommand {});
}

#[async_trait]
impl SubjectCommand for ProcessorCommand {
  fn subject(&self) -> &'static str {
    "processor"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Processor"
  }

  fn about(&self) -> String {
    "Show Trifonius processor details".to_string()
  }

  fn long_about(&self) -> String {
    "Show Trifonius processor details.".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("p")
  }

  fn list_flags(&self) -> &'static [Flag] {
    &[Flag::All, Flag::AllocationStatus, Flag::Configuration, Flag::Ids, Flag::Usage]
  }

  fn show_flags(&self) -> &'static [Flag] {
    &[Flag::All, Flag::AllocationStatus, Flag::Configuration, Flag::Usage]
  }

  async fn list_all(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
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

  async fn list_allocation_status(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_configuration(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_ids(matches, dsh_api_client).await
  }

  async fn list_ids(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_usages(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_all(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_allocation_status(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_configuration(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_default(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.show_all(target_id, matches, dsh_api_client).await
  }

  async fn show_usage(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
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
