use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::Application;
use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::topic::{topic_configuration_table_column_labels, topic_configuration_to_table_row, topic_status_table_column_labels, topic_status_to_table_row};
use crate::subject::Subject;
use crate::tabular::make_tabular_with_headers;
use crate::CommandResult;

pub(crate) struct TopicSubject {}

const TOPIC_SUBJECT_TARGET: &str = "topic";

lazy_static! {
  pub static ref TOPIC_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TopicSubject {});
}

#[async_trait]
impl Subject for TopicSubject {
  fn subject(&self) -> &'static str {
    TOPIC_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Topic"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH topics.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list topics deployed on the DSH.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("t")
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> {
    let mut capabilities: HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> = HashMap::new();
    capabilities.insert(CapabilityType::List, &TOPIC_LIST_CAPABILITY);
    capabilities.insert(CapabilityType::Show, &TOPIC_SHOW_CAPABILITY);
    capabilities
  }
}

lazy_static! {
  pub static ref TOPIC_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List topics".to_string(),
    command_long_about: Some("Lists all available topics.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &TopicListAll {}, None),
      (FlagType::AllocationStatus, &TopicListAllocationStatus {}, None),
      (FlagType::Configuration, &TopicListConfiguration {}, None),
      (FlagType::Ids, &TopicListIds {}, None),
      (FlagType::Usage, &TopicListUsage {}, None),
    ],
    default_command_executor: Some(&TopicListIds {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref TOPIC_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show topic configuration".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &TopicShowAll {}, None),
      (FlagType::AllocationStatus, &TopicShowAllocationStatus {}, None),
      (FlagType::Configuration, &TopicShowConfiguration {}, None),
      (FlagType::Usage, &TopicShowUsage {}, None),
    ],
    default_command_executor: Some(&TopicShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct TopicListAll {}

#[async_trait]
impl CommandExecutor for TopicListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for topic_id in &topic_ids {
      let topic_status = dsh_api_client.get_topic(topic_id).await?;
      table.push(topic_status_to_table_row(topic_id, &topic_status));
    }
    for line in make_tabular_with_headers(&topic_status_table_column_labels(TOPIC_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct TopicListAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let allocation_statuses = futures::future::join_all(topic_ids.iter().map(|id| dsh_api_client.get_topic_allocation_status(id.as_str()))).await;
    let mut table = vec![];
    for (topic_id, topic_status) in topic_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(topic_id, topic_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(TOPIC_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct TopicListConfiguration {}

#[async_trait]
impl CommandExecutor for TopicListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let mut table: Vec<Vec<String>> = vec![];
    for topic_id in &topic_ids {
      let topic = dsh_api_client.get_topic_configuration(topic_id).await?;
      table.push(topic_configuration_to_table_row(topic_id, &topic));
    }
    for line in make_tabular_with_headers(&topic_configuration_table_column_labels(TOPIC_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct TopicListIds {}

#[async_trait]
impl CommandExecutor for TopicListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    for topic_id in topic_ids {
      println!("{}", topic_id)
    }
    Ok(())
  }
}

struct TopicListUsage {}

#[async_trait]
impl CommandExecutor for TopicListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
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
}

struct TopicShowAll {}

#[async_trait]
impl CommandExecutor for TopicShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    let topic = dsh_api_client.get_topic(topic_id.as_str()).await?;
    println!("{:#?}", topic);
    Ok(())
  }
}

struct TopicShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    let allocation_status = dsh_api_client.get_topic_allocation_status(topic_id.as_str()).await?;
    println!("{:?}", allocation_status);
    Ok(())
  }
}

struct TopicShowConfiguration {}

#[async_trait]
impl CommandExecutor for TopicShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    let topic = dsh_api_client.get_topic_configuration(topic_id.as_str()).await?;
    println!("{:#?}", topic);
    Ok(())
  }
}

struct TopicShowUsage {}

#[async_trait]
impl CommandExecutor for TopicShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    let applications = dsh_api_client.get_application_configurations().await?;
    let usage = applications_that_use_topic(topic_id.as_str(), &applications);
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
