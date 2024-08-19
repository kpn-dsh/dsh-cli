use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::arguments::Flag;
use crate::command::SubjectCommand;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table, allocation_status_to_table_row};
use crate::tabular::{make_tabular_with_headers, print_table};
use crate::CommandResult;

pub(crate) struct SecretCommand {}

lazy_static! {
  pub static ref SECRET_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(SecretCommand {});
}

#[async_trait]
impl SubjectCommand for SecretCommand {
  fn subject(&self) -> &'static str {
    "secret"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Secret"
  }

  fn about(&self) -> String {
    "Show secret details".to_string()
  }

  fn long_about(&self) -> String {
    "Show secret details".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("s")
  }

  fn list_flags(&self) -> &'static [Flag] {
    &[Flag::All, Flag::AllocationStatus, Flag::Ids, Flag::Usage]
  }

  fn show_flags(&self) -> &'static [Flag] {
    &[Flag::AllocationStatus, Flag::Usage, Flag::Value]
  }

  async fn list_all(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_ids(matches, dsh_api_client).await
  }

  async fn list_allocation_status(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    let allocation_statusses = futures::future::join_all(secret_ids.iter().map(|id| dsh_api_client.get_secret_allocation_status(id.as_str()))).await;
    let mut table = vec![];
    for (secret_id, secret_status) in secret_ids.iter().zip(allocation_statusses) {
      table.push(allocation_status_to_table_row(secret_id, secret_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_ids(matches, dsh_api_client).await
  }

  async fn list_ids(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    for secret_id in secret_ids {
      println!("{}", secret_id)
    }
    Ok(())
  }

  async fn list_usages(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = dsh_api_client.get_application_configurations().await?;
    let secret_ids = dsh_api_client.get_secret_ids().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for secret_id in &secret_ids {
      let mut first = true;
      let usages: Vec<(String, String)> = applications_that_use_secret(secret_id, &applications);
      for (application_id, envs) in usages {
        if first {
          table.push(vec![secret_id.clone(), application_id, envs])
        } else {
          table.push(vec!["".to_string(), application_id, envs])
        }
        first = false;
      }
    }
    for line in make_tabular_with_headers(&["secret", "application", "usage"], table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn show_allocation_status(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let allocation_status = dsh_api_client.get_secret_allocation_status(target_id).await?;
    let table = allocation_status_to_table(self.subject(), target_id, &allocation_status);
    print_table(table, "", "  ", "");
    Ok(())
  }

  async fn show_usage(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = dsh_api_client.get_application_configurations().await?;
    let usage = applications_that_use_secret(target_id, &applications);
    if !usage.is_empty() {
      let table: Vec<Vec<String>> = usage.iter().map(|(application_id, usage)| vec![application_id.clone(), usage.clone()]).collect();
      for line in make_tabular_with_headers(&["application", "usage"], table) {
        println!("{}", line)
      }
    } else {
      println!("secret not used")
    }
    Ok(())
  }

  async fn show_value(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let secret = dsh_api_client.get_secret(target_id).await?;
    println!("{}", secret);
    Ok(())
  }
}

// Returns vector with pairs (application_id, environment variables)
fn applications_that_use_secret(secret_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, String)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, String)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.secrets.is_empty() {
      for application_secret in &application.secrets {
        if application_secret.name == secret_id {
          let mut envs: Vec<String> = application_secret
            .clone()
            .injections
            .into_iter()
            .filter_map(|injection| injection.get("env").cloned())
            .collect();
          if envs.len() == 1 {
            pairs.push((application_id.clone(), format!("env:{}", envs.first().unwrap())));
          }
          if envs.len() > 1 {
            envs.sort();
            let joined_envs: String = envs.join(",");
            pairs.push((application_id.clone(), format!("envs:{}", joined_envs)));
          }
        }
      }
    }
  }
  pairs
}
