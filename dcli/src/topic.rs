use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::Application;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::formatter::{print_ids, TableBuilder};
use crate::formatters::topic::{TOPIC_LABELS, TOPIC_STATUS_LABELS};
use crate::formatters::usage::{Usage, UsageLabel, USAGE_LABELS_LIST, USAGE_LABELS_SHOW};
use crate::subject::Subject;
use crate::{confirmed, DcliContext, DcliResult};

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

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    // capabilities.insert(CapabilityType::Create, TOPIC_CREATE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Delete, TOPIC_DELETE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, TOPIC_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, TOPIC_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  // pub static ref TOPIC_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
  //   capability_type: CapabilityType::Create,
  //   command_about: "Create scratch topic".to_string(),
  //   command_long_about: Some("Create a scratch topic.".to_string()),
  //   command_after_help: None,
  //   command_after_long_help: None,
  //   command_executors: vec![],
  //   default_command_executor: Some(&TopicCreate {}),
  //   run_all_executors: false,
  //   extra_arguments: vec![],
  //   extra_flags: vec![],
  // });
  pub static ref TOPIC_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Delete,
    command_about: "Delete scratch topic".to_string(),
    command_long_about: Some("Delete a scratch topic.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![],
    default_command_executor: Some(&TopicDelete {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
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

// struct TopicCreate {}

// #[async_trait]
// impl CommandExecutor for TopicCreate {
//   async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -Ï> DcliResult {
//     let topic_id = target.unwrap_or_else(|| unreachable!());
//     if context.show_capability_explanation() {
//       println!("create new scratch topic '{}'", topic_id);
//     }
//     if let Ok(_) = dsh_api_client.get_topic(&topic_id).await {
//       println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>> 1");
//       return Err(format!("scratch topic '{}' already exists", topic_id));
//     }
//     println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>> 2");
//     // let mut line = String::new();
//     // let stdin = stdin();
//     // stdin.lock().read_line(&mut line).expect("could not read line");
//     // println!("{}", line);
//     let topic = Topic { kafka_properties: Default::default(), partitions: 1, replication_factor: 3 };
//     let internal_stream = InternalManagedStream { kafka_properties: Default::default(), kind: InternalManagedStreamKind::Internal, partitions: 1, replication_factor: 1 };
//     dsh_api_client.create_topic(&topic_id, &topic).await?;
//     println!("ok");
//     Ok(true)
//   }
// }

struct TopicDelete {}

#[async_trait]
impl CommandExecutor for TopicDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("delete topic '{}'", topic_id);
    }
    if dsh_api_client.get_topic(&topic_id).await.is_err() {
      return Err(format!("scratch topic '{}' does not exists", topic_id));
    }
    println!("type 'yes' and Enter to delete scratch topic '{}'", topic_id);
    if confirmed()? {
      dsh_api_client.delete_topic(&topic_id).await?;
      println!("ok");
    } else {
      println!("cancelled");
    }
    Ok(false)
  }
}

struct TopicListAll {}

#[async_trait]
impl CommandExecutor for TopicListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all stream and internal topics");
    }
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let statuses = try_join_all(topic_ids.iter().map(|id| dsh_api_client.get_topic(id.as_str()))).await?;
    let mut builder = TableBuilder::list(&TOPIC_STATUS_LABELS, context);
    for (topic_id, status) in topic_ids.iter().zip(statuses) {
      builder.value(topic_id.to_string(), &status);
    }
    builder.print();
    Ok(false)
  }
}

struct TopicListAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all stream and internal topics with their allocation status");
    }
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let allocation_statuses = try_join_all(topic_ids.iter().map(|id| dsh_api_client.get_topic_allocation_status(id.as_str()))).await?;
    print_allocation_statuses(topic_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct TopicListConfiguration {}

#[async_trait]
impl CommandExecutor for TopicListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all stream and internal topics with their configurations");
    }
    let topic_ids = dsh_api_client.get_topic_ids().await?;
    let configurations = try_join_all(topic_ids.iter().map(|id| dsh_api_client.get_topic_configuration(id.as_str()))).await?;
    let mut builder = TableBuilder::list(&TOPIC_LABELS, context);
    for (topic_id, configuration) in topic_ids.iter().zip(configurations) {
      builder.value(topic_id.to_string(), &configuration);
    }
    builder.print();
    Ok(false)
  }
}

struct TopicListIds {}

#[async_trait]
impl CommandExecutor for TopicListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all stream and internal topic ids");
    }
    print_ids("topic ids".to_string(), dsh_api_client.get_topic_ids().await?, context);
    Ok(false)
  }
}

struct TopicListUsage {}

#[async_trait]
impl CommandExecutor for TopicListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all stream and internal topics with the applications that use them");
    }
    let (topic_ids, applications) = try_join!(dsh_api_client.get_topic_ids(), dsh_api_client.get_application_configurations())?;
    let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_LABELS_LIST, context);
    for topic_id in &topic_ids {
      let mut first = true;
      let usages: Vec<(String, Vec<String>)> = applications_that_use_topic(topic_id, &applications);
      for (application_id, envs) in usages {
        if !envs.is_empty() {
          if first {
            builder.row(&Usage::application(topic_id.to_string(), application_id, envs));
          } else {
            builder.row(&Usage::application("".to_string(), application_id, envs));
          }
          first = false;
        }
      }
    }
    builder.print();
    Ok(false)
  }
}

struct TopicShowAll {}

#[async_trait]
impl CommandExecutor for TopicShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for topic '{}'", topic_id);
    }
    let topic = dsh_api_client.get_topic(topic_id.as_str()).await?;
    println!("{:#?}", topic);
    Ok(false)
  }
}

struct TopicShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the allocation status for topic '{}'", topic_id);
    }
    print_allocation_status(topic_id.clone(), dsh_api_client.get_topic_allocation_status(topic_id.as_str()).await?, context);
    Ok(false)
  }
}

struct TopicShowConfiguration {}

#[async_trait]
impl CommandExecutor for TopicShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the configuration for topic '{}'", topic_id);
    }
    let mut builder = TableBuilder::show(&TOPIC_STATUS_LABELS, context);
    builder.value(topic_id.clone(), &dsh_api_client.get_topic(topic_id.as_str()).await?);
    builder.print();
    Ok(false)
  }
}

struct TopicShowUsage {}

#[async_trait]
impl CommandExecutor for TopicShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the applications that use topic '{}'", topic_id);
    }
    let applications = dsh_api_client.get_application_configurations().await?;
    let usages: Vec<(String, Vec<String>)> = applications_that_use_topic(topic_id.as_str(), &applications);
    if !usages.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_LABELS_SHOW, context);
      for (application_id, envs) in usages {
        builder.row(&Usage::application(application_id.clone(), application_id.to_string(), envs));
      }
      builder.print();
    } else {
      println!("topic not used")
    }
    Ok(false)
  }
}

pub(crate) fn applications_that_use_topic(topic_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, Vec<String>)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, Vec<String>)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.env.is_empty() {
      let mut envs_that_contain_topic_id: Vec<String> = application.env.clone().into_iter().filter(|(_, v)| v.contains(topic_id)).map(|(k, _)| k).collect();
      envs_that_contain_topic_id.sort();
      pairs.push((application_id.clone(), envs_that_contain_topic_id));
    }
  }
  pairs
}
