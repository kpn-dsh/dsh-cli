use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::command::SubjectCommand;
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::topic::{topic_configuration_table_column_labels, topic_configuration_to_table_row, topic_status_table_column_labels, topic_status_to_table_row};
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

pub(crate) struct TopicCommand {}

lazy_static! {
  pub static ref TOPIC_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(TopicCommand {});
}

#[async_trait]
impl SubjectCommand for TopicCommand {
  fn subject(&self) -> &'static str {
    "topic"
  }

  fn subject_first_upper(&self) -> &'static str {
    "Topic"
  }

  fn about(&self) -> String {
    "Show, manage and list DSH topics.".to_string()
  }

  fn long_about(&self) -> String {
    "Show, manage and list topics managed by the DSH. Note that 'tcli' can only work with stream and internal topics. Scratch topics can not be used by this tool.".to_string()
  }

  fn alias(&self) -> Option<&str> {
    Some("t")
  }

  fn list_flags(&self) -> &'static [FlagType] {
    &[FlagType::All, FlagType::AllocationStatus, FlagType::Configuration, FlagType::Ids, FlagType::Usage]
  }

  fn show_flags(&self) -> &'static [FlagType] {
    &[FlagType::All, FlagType::AllocationStatus, FlagType::Configuration, FlagType::Usage]
  }

  async fn list_all(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for topic_id in &topic_ids {
      let topic_status = dsh_api_client.get_topic(topic_id).await?;
      table.push(topic_status_to_table_row(topic_id, &topic_status));
    }
    for line in make_tabular_with_headers(&topic_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_allocation_status(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let allocation_statuses = futures::future::join_all(topic_ids.iter().map(|id| dsh_api_client.get_topic_allocation_status(id.as_str()))).await;
    let mut table = vec![];
    for (topic_id, topic_status) in topic_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(topic_id, topic_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_configuration(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for topic_id in &topic_ids {
      let topic = dsh_api_client.get_topic_configuration(topic_id).await?;
      table.push(topic_configuration_to_table_row(topic_id, &topic));
    }
    for line in make_tabular_with_headers(&topic_configuration_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_ids(matches, dsh_api_client).await
  }

  async fn list_ids(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    for topic_id in topic_ids {
      println!("{}", topic_id)
    }
    Ok(())
  }

  async fn list_usages(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let applications = dsh_api_client.get_application_configurations().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for topic_id in &topic_ids {
      let mut first = true;
      let usages: Vec<(String, String)> = applications_that_use_topic(topic_id, &applications);
      for (application_id, envs) in usages {
        if first {
          table.push(vec![topic_id.clone(), application_id, envs])
        } else {
          table.push(vec!["".to_string(), application_id, envs])
        }
        first = false;
      }
    }
    for line in make_tabular_with_headers(&["topic", "application", "usage"], table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn show_all(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic = dsh_api_client.get_topic(target_id).await?;
    println!("{:#?}", topic);
    Ok(())
  }

  async fn show_allocation_status(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let allocation_status = dsh_api_client.get_topic_allocation_status(target_id).await?;
    println!("{:?}", allocation_status);
    Ok(())
  }

  async fn show_configuration(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic = dsh_api_client.get_topic_configuration(target_id).await?;
    println!("{:#?}", topic);
    Ok(())
  }

  async fn show_default(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.show_all(target_id, matches, dsh_api_client).await
  }

  async fn show_usage(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let applications = dsh_api_client.get_application_configurations().await?;
    let usage = applications_that_use_topic(target_id, &applications);
    if !usage.is_empty() {
      let table: Vec<Vec<String>> = usage.iter().map(|(application_id, usage)| vec![application_id.clone(), usage.clone()]).collect();
      for line in make_tabular_with_headers(&["application", "usage"], table) {
        println!("{}", line)
      }
    } else {
      println!("topic not used")
    }
    Ok(())
  }
}

fn applications_that_use_topic(topic_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, String)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, String)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.env.is_empty() {
      let mut envs_that_contain_topic_id: Vec<String> = application.env.clone().into_iter().filter(|(_, v)| v.contains(topic_id)).map(|(k, _)| k).collect();
      if envs_that_contain_topic_id.len() == 1 {
        pairs.push((application_id.clone(), format!("env:{}", envs_that_contain_topic_id.first().unwrap())));
      }
      if envs_that_contain_topic_id.len() > 1 {
        envs_that_contain_topic_id.sort();
        let joined_envs: String = envs_that_contain_topic_id.join(",");
        pairs.push((application_id.clone(), format!("envs:{}", joined_envs)));
      }
    }
  }
  pairs
}
