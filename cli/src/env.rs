use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::AppCatalogApp;
use trifonius_dsh_api::DshApiClient;

use crate::app::get_application_from_app;
use crate::command::SubjectCommand;
use crate::flags::FlagType;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

pub(crate) struct EnvCommand {}

lazy_static! {
  pub static ref ENV_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(EnvCommand {});
}

#[async_trait]
impl SubjectCommand for EnvCommand {
  fn subject(&self) -> &'static str {
    "env"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Env"
  }

  fn about(&self) -> String {
    "Find environment variables.".to_string()
  }

  fn long_about(&self) -> String {
    "Find environment".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("e")
  }

  fn supports_find(&self) -> bool {
    true
  }

  fn supports_list(&self) -> bool {
    false
  }

  fn supports_show(&self) -> bool {
    false
  }

  fn find_flags(&self) -> &'static [FlagType] {
    &[FlagType::All, FlagType::Apps, FlagType::Applications]
  }

  async fn find_all(&self, query: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.find_in_applications(query, matches, dsh_api_client).await?;
    self.find_in_apps(query, matches, dsh_api_client).await
  }

  async fn find_default(&self, query: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.find_all(query, matches, dsh_api_client).await
  }

  async fn find_in_apps(&self, query: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
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
          .filter_map(|(key, value)| if query == value { Some(key.to_string()) } else { None })
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

  async fn find_in_applications(&self, query: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = &dsh_api_client.get_application_actual_configurations().await?;
    let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    application_ids.sort();
    let mut table: Vec<Vec<String>> = vec![];
    for application_id in application_ids {
      let application = applications.get(&application_id).unwrap();
      let mut keys: Vec<String> = application
        .env
        .iter()
        .filter_map(|(key, value)| if query == value { Some(key.to_string()) } else { None })
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
