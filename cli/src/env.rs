use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::dsh_api_client::DshApiClient;
use trifonius_dsh_api::types::AppCatalogApp;

use crate::app::get_application_from_app;
use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::subject::Subject;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

pub(crate) struct EnvSubject {}

const ENV_SUBJECT_TARGET: &str = "env";

lazy_static! {
  pub static ref ENV_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(EnvSubject {});
}

#[async_trait]
impl Subject for EnvSubject {
  fn subject(&self) -> &'static str {
    ENV_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Env"
  }

  fn subject_command_about(&self) -> String {
    "Find values used in configurations.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Find values used in environment variables used to configure applications/services and apps deployed on the DSH.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("e")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Find, ENV_FIND_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref ENV_FIND_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Find,
    command_about: "Find environment variable values".to_string(),
    command_long_about: Some("Find values used in environment variables used to configure applications/services and apps deployed on the DSH.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::App, &EnvFindInApps {}, None), (FlagType::Application, &EnvFindInApplications {}, None),],
    default_command_executor: Some(&EnvFindInApplications {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct EnvFindInApps {}

#[async_trait]
impl CommandExecutor for EnvFindInApps {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let query = target.unwrap_or_else(|| unreachable!());
    let apps: &HashMap<String, AppCatalogApp> = &dsh_api_client.get_app_configurations().await?;
    let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    app_ids.sort();
    let mut table: Vec<Vec<String>> = vec![];
    for app_id in app_ids {
      let app = apps.get(&app_id).unwrap();
      if let Some((resource_id, application)) = get_application_from_app(app) {
        let mut keys: Vec<String> = application
          .env
          .iter()
          .filter_map(|(key, value)| if &query == value { Some(key.to_string()) } else { None })
          .collect();
        if !keys.is_empty() {
          keys.sort();
          table.push(vec![app_id, resource_id.to_string(), keys.join(", ")]);
        }
      }
    }
    for line in make_tabular_with_headers(&["app", "application resource", "environment variables"], table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct EnvFindInApplications {}

#[async_trait]
impl CommandExecutor for EnvFindInApplications {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let query = target.unwrap_or_else(|| unreachable!());
    let applications = &dsh_api_client.get_application_actual_configurations().await?;
    let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    application_ids.sort();
    let mut table: Vec<Vec<String>> = vec![];
    for application_id in application_ids {
      let application = applications.get(&application_id).unwrap();
      let mut keys: Vec<String> = application
        .env
        .iter()
        .filter_map(|(key, value)| if &query == value { Some(key.to_string()) } else { None })
        .collect();
      if !keys.is_empty() {
        keys.sort();
        table.push(vec![application_id, keys.join(", ")]);
      }
    }
    for line in make_tabular_with_headers(&["application", "environment variables"], table) {
      println!("{}", line)
    }
    Ok(())
  }
}
