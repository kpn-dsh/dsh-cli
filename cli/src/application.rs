use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::{DshApiClient, DshApiResult};

use crate::command::SubjectCommand;
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table, allocation_status_to_table_row};
use crate::formatters::application::{application_to_default_vector, default_application_column_labels, default_application_table};
use crate::tabular::{make_tabular, make_tabular_with_headers, print_table, print_tabular};
use crate::CommandResult;

pub(crate) struct ApplicationCommand {}

lazy_static! {
  pub static ref APPLICATION_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(ApplicationCommand {});
}

#[async_trait]
impl SubjectCommand for ApplicationCommand {
  fn subject(&self) -> &'static str {
    "application"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Application"
  }

  fn about(&self) -> String {
    "Show, manage and list DSH applications.".to_string()
  }

  fn long_about(&self) -> String {
    "Show, manage and list applications deployed on the DSH.".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("a")
  }

  fn list_flags(&self) -> &'static [FlagType] {
    &[FlagType::All, FlagType::AllocationStatus, FlagType::Configuration, FlagType::Ids, FlagType::Tasks]
  }

  fn show_flags(&self) -> &'static [FlagType] {
    &[FlagType::All, FlagType::AllocationStatus, FlagType::Configuration, FlagType::Tasks]
  }

  async fn list_all(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    print_applications(&dsh_api_client.get_application_actual_configurations().await?)
  }

  async fn list_allocation_status(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_ids = dsh_api_client.get_application_ids().await?;
    let allocation_statuses = futures::future::join_all(
      application_ids
        .iter()
        .map(|application_id| dsh_api_client.get_application_allocation_status(application_id.as_str())),
    )
    .await;
    let mut table = vec![];
    for (id, allocation_status) in application_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_configuration(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    print_applications(&dsh_api_client.get_application_configurations().await?)
  }

  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_ids(matches, dsh_api_client).await
  }

  async fn list_ids(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_ids = dsh_api_client.get_application_ids().await?;
    for application_id in application_ids {
      println!("{}", application_id)
    }
    Ok(())
  }

  async fn list_tasks(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application_ids = dsh_api_client.get_application_ids_with_derived_tasks().await?;
    let tasks: Vec<DshApiResult<Vec<String>>> = futures::future::join_all(
      application_ids
        .iter()
        .map(|application_id| dsh_api_client.get_application_derived_task_ids(application_id.as_str())),
    )
    .await;
    let mut table = vec![];
    for (application_id, tasks) in application_ids.iter().zip(tasks) {
      if let Ok(mut ts) = tasks {
        if !ts.is_empty() {
          ts.sort();
          let vector = vec![
            application_id.to_string(),
            if ts.len() <= 4 {
              ts.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
            } else {
              format!(
                "{}, plus {} more",
                ts.iter().take(4).map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
                ts.len() - 4,
              )
            },
          ];

          table.push(vector);
        }
      }
    }
    for line in make_tabular_with_headers(&["application", "tasks"], table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn show_all(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    match dsh_api_client.get_application_actual_configuration(target_id).await {
      Ok(application) => {
        let table = default_application_table(target_id, &application);
        let tabular = make_tabular(table, "", "  ", "");
        print_tabular("", &tabular);
        Ok(())
      }
      Err(error) => self.to_command_error_with_id(error, target_id),
    }
  }

  async fn show_allocation_status(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let allocation_status = dsh_api_client.get_application_allocation_status(target_id).await?;
    let table = allocation_status_to_table(self.subject(), target_id, &allocation_status);
    print_table(table, "", "  ", "");
    Ok(())
  }

  async fn show_configuration(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let application = dsh_api_client.get_application_configuration(target_id).await?;
    let table = default_application_table(target_id, &application);
    let tabular = make_tabular(table, "", "  ", "");
    print_tabular("", &tabular);
    Ok(())
  }

  async fn show_default(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.show_configuration(target_id, matches, dsh_api_client).await
  }

  async fn show_tasks(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let task_ids = dsh_api_client.get_application_derived_task_ids(target_id).await?;
    let allocation_statuses = futures::future::join_all(
      task_ids
        .iter()
        .map(|task_id| dsh_api_client.get_application_task_allocation_status(target_id, task_id.as_str())),
    )
    .await;
    let mut table = vec![];
    for (task_id, allocation_status) in task_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(task_id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

fn print_applications(applications: &HashMap<String, Application>) -> CommandResult {
  let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
  application_ids.sort();
  let mut table: Vec<Vec<String>> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    table.push(application_to_default_vector(application_id.as_str(), application));
  }
  for line in make_tabular_with_headers(&default_application_column_labels(), table) {
    println!("{}", line)
  }
  Ok(())
}
