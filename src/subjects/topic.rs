use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::{notifications_to_string, OutputFormat};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::application::find_applications_that_use_topic;
use dsh_api::types::Application;
use dsh_api::types::{Topic, TopicStatus};
use dsh_api::{Injection, UsedBy};
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

use crate::arguments::topic_id_argument;
use crate::capability::{Capability, CommandExecutor, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::formatter::PROPERTY_LABELS;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, USED_BY_LABELS_LIST};
use crate::DshCliResult;

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

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH topics.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list topics deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("t")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(TOPIC_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(TOPIC_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(TOPIC_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TOPIC_CAPABILITIES
  }
}

lazy_static! {
  static ref TOPIC_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, "Delete scratch topic")
      .set_long_about("Delete a scratch topic.")
      .set_default_command_executor(&TopicDelete {})
      .add_target_argument(topic_id_argument().required(true))
  );
  static ref TOPIC_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List topics")
      .set_long_about("Lists all available scratch topics.")
      .set_default_command_executor(&TopicListConfiguration {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &TopicListAllocationStatus {}, None),
        (FlagType::Ids, &TopicListIds {}, None),
        (FlagType::Usage, &TopicListUsage {}, None),
      ])
      .set_run_all_executors(true)
  );
  static ref TOPIC_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), "Show topic configuration")
      .set_default_command_executor(&TopicShow {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &TopicShowAllocationStatus {}, None),
        (FlagType::Properties, &TopicShowProperties {}, None),
        (FlagType::Usage, &TopicShowUsage {}, None),
      ])
      .add_target_argument(topic_id_argument().required(true))
  );
  static ref TOPIC_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TOPIC_DELETE_CAPABILITY.as_ref(), TOPIC_LIST_CAPABILITY.as_ref(), TOPIC_SHOW_CAPABILITY.as_ref()];
}

struct TopicDelete {}

#[async_trait]
impl CommandExecutor for TopicDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete topic '{}'", topic_id));
    if context.client_unchecked().get_topic(&topic_id).await.is_err() {
      return Err(format!("scratch topic '{}' does not exists", topic_id));
    }
    if context.confirmed(format!("type 'yes' to delete scratch topic '{}': ", topic_id).as_str())? {
      if context.dry_run {
        context.print_warning("dry-run mode, topic not deleted");
      } else {
        context.client_unchecked().delete_topic_configuration(&topic_id).await?;
        context.print_outcome(format!("topic '{}' deleted", topic_id));
      }
    } else {
      context.print_outcome(format!("cancelled, topic '{}' not deleted", topic_id));
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicListAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topics with their allocation status");
    let start_instant = Instant::now();
    let topic_ids = context.client_unchecked().get_topic_ids().await?;
    let allocation_statuses = try_join_all(topic_ids.iter().map(|id| context.client_unchecked().get_topic_status(id.as_str()))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("topic id"), context);
    formatter.push_target_ids_and_values(topic_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicListConfiguration {}

#[async_trait]
impl CommandExecutor for TopicListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topics with their configurations");
    let start_instant = Instant::now();
    let topic_ids = context.client_unchecked().get_topic_ids().await?;
    let configurations = try_join_all(topic_ids.iter().map(|id| context.client_unchecked().get_topic_configuration(id.as_str()))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&TOPIC_LABELS, None, context);
    formatter.push_target_ids_and_values(topic_ids.as_slice(), configurations.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicListIds {}

#[async_trait]
impl CommandExecutor for TopicListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topic ids");
    let start_instant = Instant::now();
    let topic_ids = context.client_unchecked().get_topic_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("topic id", context);
    formatter.push_target_ids(&topic_ids);
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(Some(OutputFormat::Plain))
  }
}

struct TopicListUsage {}

#[async_trait]
impl CommandExecutor for TopicListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topics with the services that use them");
    let start_instant = Instant::now();
    let (topic_ids, services) = try_join!(
      context.client_unchecked().get_topic_ids(),
      context.client_unchecked().get_application_configuration_map(),
    )?;
    context.print_execution_time(start_instant);
    let mut tuples: Vec<(String, UsedBy)> = vec![];
    for topic_id in &topic_ids {
      let service_usages: Vec<(String, &Application, Vec<Injection>)> = find_applications_that_use_topic(topic_id, &services);
      for (service_id, service, injections) in service_usages {
        if !injections.is_empty() {
          tuples.push((topic_id.to_string(), UsedBy::Application(service_id, service.instances, injections)));
        }
      }
    }
    if tuples.is_empty() {
      context.print_outcome("no topics found in services");
    } else {
      let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("topic id"), context);
      formatter.push_target_id_value_pairs(&tuples);
      formatter.print()?;
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the allocation status for topic '{}'", topic_id));
    let start_instant = Instant::now();
    let allocation_status = context.client_unchecked().get_topic_status(topic_id.as_str()).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(topic_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("topic id"), context).print(&allocation_status)
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicShow {}

#[async_trait]
impl CommandExecutor for TopicShow {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the configuration for topic '{}'", topic_id));
    let start_instant = Instant::now();
    let topic = context.client_unchecked().get_topic_configuration(topic_id.as_str()).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(topic_id, &TOPIC_STATUS_LABELS, None, context).print(&topic)
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicShowProperties {}

#[async_trait]
impl CommandExecutor for TopicShowProperties {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the properties for topic '{}'", topic_id));
    let start_instant = Instant::now();
    let topic_status = context.client_unchecked().get_topic(topic_id.as_str()).await?;
    context.print_execution_time(start_instant);
    let mut pairs: Vec<(String, String)> = topic_status.actual.unwrap().kafka_properties.into_iter().collect::<Vec<_>>();
    pairs.sort_by(|(key_a, _), (key_b, _)| key_a.cmp(key_b));
    let (properties, values): (Vec<String>, Vec<String>) = pairs.into_iter().unzip();
    let mut formatter = ListFormatter::new(&PROPERTY_LABELS, Some("property"), context);
    formatter.push_target_ids_and_values(&properties, &values);
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct TopicShowUsage {}

#[async_trait]
impl CommandExecutor for TopicShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the services that use topic '{}'", topic_id));
    let start_instant = Instant::now();
    let services = context.client_unchecked().get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let usages: Vec<(String, &Application, Vec<Injection>)> = find_applications_that_use_topic(topic_id.as_str(), &services);
    let used_bys = usages
      .into_iter()
      .filter_map(|(service_id, service, injections)| if injections.is_empty() { None } else { Some(UsedBy::Application(service_id.clone(), service.instances, injections)) })
      .collect::<Vec<_>>();
    if !used_bys.is_empty() {
      let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("topic id"), context);
      formatter.push_values(&used_bys);
      formatter.print()?;
    } else {
      context.print_outcome("topic not used");
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum TopicLabel {
  CleanupPolicy,
  #[allow(dead_code)]
  DerivedFrom,
  MaxMessageBytes,
  Notifications,
  Partitions,
  Provisioned,
  ReplicationFactor,
  Target,
  TimestampType,
}

impl Label for TopicLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup policy",
      Self::DerivedFrom => "derived from",
      Self::MaxMessageBytes => "max message bytes",
      Self::Notifications => "notifications",
      Self::Partitions => "number of partitions",
      Self::Provisioned => "provisioned",
      Self::ReplicationFactor => "replication factor",
      Self::Target => "topic id",
      Self::TimestampType => "timestamp type",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup",
      Self::DerivedFrom => "derived",
      Self::MaxMessageBytes => "max bytes",
      Self::Notifications => "not",
      Self::Partitions => "part",
      Self::Provisioned => "prov",
      Self::ReplicationFactor => "repl",
      Self::Target => "topic id",
      Self::TimestampType => "ts",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<TopicLabel> for Topic {
  fn value(&self, label: &TopicLabel, target_id: &str) -> String {
    match label {
      TopicLabel::CleanupPolicy => self.kafka_properties.get("cleanup.policy").cloned().unwrap_or_default(),
      TopicLabel::DerivedFrom => "".to_string(),
      TopicLabel::MaxMessageBytes => self.kafka_properties.get("max.message.bytes").cloned().unwrap_or_default(),
      TopicLabel::Notifications => "".to_string(),
      TopicLabel::Partitions => self.partitions.to_string(),
      TopicLabel::Provisioned => "".to_string(),
      TopicLabel::ReplicationFactor => self.replication_factor.to_string(),
      TopicLabel::Target => target_id.to_string(),
      TopicLabel::TimestampType => self.kafka_properties.get("message.timestamp.type").cloned().unwrap_or_default(),
    }
  }
}

impl SubjectFormatter<TopicLabel> for TopicStatus {
  fn value(&self, label: &TopicLabel, target_id: &str) -> String {
    match label {
      TopicLabel::CleanupPolicy => self
        .actual
        .as_ref()
        .and_then(|a| a.kafka_properties.get("cleanup.policy"))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::DerivedFrom => self.status.derived_from.clone().unwrap_or_default(),
      TopicLabel::MaxMessageBytes => self
        .actual
        .as_ref()
        .and_then(|a| a.kafka_properties.get("max.message.bytes"))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::Notifications => notifications_to_string(&self.status.notifications),
      TopicLabel::Partitions => self.actual.as_ref().map(|a| a.partitions.to_string()).unwrap_or_default(),
      TopicLabel::Provisioned => self.status.provisioned.to_string(),
      TopicLabel::ReplicationFactor => self.actual.as_ref().map(|a| a.replication_factor.to_string()).unwrap_or_default(),
      TopicLabel::Target => target_id.to_string(),
      TopicLabel::TimestampType => self
        .actual
        .as_ref()
        .and_then(|a| a.kafka_properties.get("message.timestamp.type"))
        .cloned()
        .unwrap_or_default(),
    }
  }
}

pub static TOPIC_STATUS_LABELS: [TopicLabel; 8] = [
  TopicLabel::Target,
  TopicLabel::Partitions,
  TopicLabel::ReplicationFactor,
  TopicLabel::CleanupPolicy,
  TopicLabel::TimestampType,
  TopicLabel::MaxMessageBytes,
  TopicLabel::Notifications,
  TopicLabel::Provisioned,
];

pub static TOPIC_LABELS: [TopicLabel; 8] = [
  TopicLabel::Target,
  TopicLabel::Partitions,
  TopicLabel::ReplicationFactor,
  TopicLabel::CleanupPolicy,
  TopicLabel::TimestampType,
  TopicLabel::MaxMessageBytes,
  TopicLabel::Notifications,
  TopicLabel::Provisioned,
];
