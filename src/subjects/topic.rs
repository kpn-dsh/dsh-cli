use crate::arguments::topic_id_argument;
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::formatter::{hashmap_to_table, PROPERTY_LABELS};
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{notifications_to_string, OutputFormat};
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, USED_BY_LABELS_LIST};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::builder::PossibleValue;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::application::find_applications_that_use_topic;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::Application;
use dsh_api::types::{Topic, TopicStatus};
use dsh_api::{Injection, UsedBy};
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;

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
      CREATE_COMMAND => Some(TOPIC_CREATE_CAPABILITY.as_ref()),
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
  static ref TOPIC_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &TopicCreate {}, "Create new topic")
      .add_target_argument(topic_id_argument().required(true))
      .add_extra_arguments(vec![
        cleanup_policy_flag(TOPIC_OPTIONS_HEADING),
        compression_type_flag(TOPIC_OPTIONS_HEADING),
        delete_retention_ms_flag(TOPIC_OPTIONS_HEADING),
        max_message_size_flag(TOPIC_OPTIONS_HEADING),
        message_timestamp_type_flag(TOPIC_OPTIONS_HEADING),
        partitions_flag(TOPIC_OPTIONS_HEADING),
        retention_bytes_flag(TOPIC_OPTIONS_HEADING),
        retention_ms_flag(TOPIC_OPTIONS_HEADING),
        segment_bytes_flag(TOPIC_OPTIONS_HEADING),
      ])
  );
  static ref TOPIC_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &TopicDelete {}, "Delete scratch topic")
      .set_long_about("Delete a scratch topic.")
      .add_target_argument(topic_id_argument().required(true))
  );
  static ref TOPIC_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TopicListConfiguration {}, "List topics")
      .set_long_about("Lists all available scratch topics.")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &TopicListAllocationStatus {}, None),
        (FlagType::Ids, &TopicListIds {}, None),
        (FlagType::Usage, &TopicListUsage {}, None),
      ])
  );
  static ref TOPIC_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &TopicShow {}, "Show topic configuration")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &TopicShowAllocationStatus {}, None),
        (FlagType::Properties, &TopicShowProperties {}, None),
        (FlagType::Usage, &TopicShowUsage {}, None),
      ])
      .add_target_argument(topic_id_argument().required(true))
  );
  static ref TOPIC_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TOPIC_CREATE_CAPABILITY.as_ref(), TOPIC_DELETE_CAPABILITY.as_ref(), TOPIC_LIST_CAPABILITY.as_ref(), TOPIC_SHOW_CAPABILITY.as_ref()];
}

const TOPIC_OPTIONS_HEADING: &str = "Topic options";

pub(crate) const CLEANUP_POLICY_FLAG: &str = "cleanup-policy";

pub(crate) fn cleanup_policy_flag(heading: &'static str) -> Arg {
  Arg::new(CLEANUP_POLICY_FLAG)
    .long("cleanup-policy")
    .action(ArgAction::Set)
    .value_parser(builder::PossibleValuesParser::new(vec![
      PossibleValue::new("compact"),
      PossibleValue::new("delete").alias("1day"), // TODO Remove in next non backwards compatible version
    ]))
    .value_name("POLICY")
    .help("Cleanup policy")
    .long_help("Cleanup policy for the new topic.")
    .help_heading(heading)
}

pub(crate) const COMPRESSION_TYPE_FLAG: &str = "compression-type";

pub(crate) fn compression_type_flag(heading: &'static str) -> Arg {
  Arg::new(COMPRESSION_TYPE_FLAG)
    .long("compression-type")
    .action(ArgAction::Set)
    .value_parser(builder::PossibleValuesParser::new(vec![
      PossibleValue::new("gzip"),
      PossibleValue::new("lz4"),
      PossibleValue::new("producer"),
      PossibleValue::new("snappy"),
      PossibleValue::new("uncompressed"),
      PossibleValue::new("zstd"),
    ]))
    .value_name("TYPE")
    .help("Compression type")
    .long_help("Compression type for the new topic.")
    .help_heading(heading)
}

pub(crate) const DELETE_RETENTION_MS_FLAG: &str = "delete-retention-ms";

pub(crate) fn delete_retention_ms_flag(heading: &'static str) -> Arg {
  Arg::new(DELETE_RETENTION_MS_FLAG)
    .long("delete-retention-ms")
    .alias("delete-retention") // TODO Remove in next non backwards compatible version
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(0..))
    .value_name("MS")
    .help("Delete retention")
    .long_help("Delete retention time in milliseconds.")
    .help_heading(heading)
}

pub(crate) const MAX_MESSAGE_BYTES_FLAG: &str = "max-message-bytes";

pub(crate) fn max_message_size_flag(heading: &'static str) -> Arg {
  Arg::new(MAX_MESSAGE_BYTES_FLAG)
    .long("max-message-bytes")
    .alias("max-message-size") // TODO Remove in next non backwards compatible version
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1024..=1048576))
    .value_name("BYTES")
    .help("Max message size")
    .long_help(
      "Maximum message size (in bytes) for the new topic. \
      The minimum message size is 1024 bytes and the maximum size is 1048576 bytes.",
    )
    .help_heading(heading)
}

pub(crate) const MESSAGE_TIMESTAMP_TYPE_FLAG: &str = "message-timestamp-type";
const TIMESTAMP_CREATE_TIME: &str = "create-time";
const TIMESTAMP_LOG_APPEND_TIME: &str = "log-append-time";

pub(crate) fn message_timestamp_type_flag(heading: &'static str) -> Arg {
  Arg::new(MESSAGE_TIMESTAMP_TYPE_FLAG)
    .long("message-timestamp-type")
    .alias("timestamps") // TODO Remove in next non backwards compatible version
    .action(ArgAction::Set)
    .value_parser(builder::PossibleValuesParser::new(vec![
      PossibleValue::new(TIMESTAMP_CREATE_TIME).alias("producer"),   // TODO Remove in next non backwards compatible version
      PossibleValue::new(TIMESTAMP_LOG_APPEND_TIME).alias("broker"), // TODO Remove in next non backwards compatible version
    ]))
    .value_name("TYPE")
    .help("Message timestamp type")
    .long_help(
      "Message timestamp type for the new topic. \
        The allowed values are 'create-time' and 'log-append-time'.",
    )
    .help_heading(heading)
}

pub(crate) const PARTITIONS_FLAG: &str = "partitions";

pub(crate) fn partitions_flag(heading: &'static str) -> Arg {
  Arg::new(PARTITIONS_FLAG)
    .long("partitions")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u32>::new().range(1..=128))
    .value_name("PARTITIONS")
    .help("Number of partitions")
    .long_help(
      "Number of partitions for the new topic. \
          If this option is not specified the created topic will have only 1 partition.",
    )
    .help_heading(heading)
}

pub(crate) const RETENTION_BYTES_FLAG: &str = "retention-bytes";

pub(crate) fn retention_bytes_flag(heading: &'static str) -> Arg {
  Arg::new(RETENTION_BYTES_FLAG)
    .long("retention-bytes")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(0..))
    .value_name("BYTES")
    .help("Retention bytes")
    .long_help(
      "Retention bytes for the new topic. \
      The minimum retention bytes value is 0.",
    )
    .help_heading(heading)
}

pub(crate) const RETENTION_MS_FLAG: &str = "retention-ms";

pub(crate) fn retention_ms_flag(heading: &'static str) -> Arg {
  Arg::new(RETENTION_MS_FLAG)
    .long("retention-ms")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(3600000..=31536000000))
    .value_name("MS")
    .help("Retention milliseconds")
    .long_help(
      "Retention time in milliseconds for the new topic. \
      The minimum retention time value is 3600000 milliseconds (1 hour) \
      and the maximum retention time is 31536000000 milliseconds (1 year).",
    )
    .help_heading(heading)
}

pub(crate) const SEGMENT_BYTES_FLAG: &str = "segment-bytes";

pub(crate) fn segment_bytes_flag(heading: &'static str) -> Arg {
  Arg::new(SEGMENT_BYTES_FLAG)
    .long("segment-bytes")
    .alias("segment-size") // TODO Remove in next non backwards compatible version
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(52428800..))
    .value_name("BYTES")
    .help("Segment bytes")
    .long_help(
      "Segment bytes for the new topic. \
          The minimum segment bytes value is 52428800 bytes",
    )
    .help_heading(heading)
}

struct TopicCreate {}

#[async_trait]
impl CommandExecutor for TopicCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    const REPLICATION_FACTOR: u32 = 3;
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if client.get_topic_configuration(&topic_id).await.is_ok() {
      return Err(format!("topic '{}' already exists", topic_id));
    }
    let topic = create_topic(matches)?;
    context.print_explanation(format!(
      "create new topic '{}', number of partitions {}, replication factor {}",
      topic_id, topic.partitions, REPLICATION_FACTOR
    ));
    if context.dry_run {
      context.print_warning("dry-run mode, topic not created");
    } else {
      client.put_topic_configuration(&topic_id, &topic).await?;
      context.print_outcome(format!("topic '{}' created", topic_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) const CLEANUP_POLICY_PROPERTY: &str = "cleanup.policy";
pub(crate) const COMPRESSION_TYPE_PROPERTY: &str = "compression.type";
pub(crate) const DELETE_RETENTION_MS_PROPERTY: &str = "delete.retention.ms";
pub(crate) const MAX_MESSAGE_BYTES_PROPERTY: &str = "max.message.bytes";
pub(crate) const MESSAGE_TIMESTAMP_PROPERTY: &str = "message.timestamp.type";
pub(crate) const RETENTION_BYTES_PROPERTY: &str = "retention.bytes";
pub(crate) const RETENTION_MS_PROPERTY: &str = "retention.ms";
pub(crate) const SEGMENT_BYTES_PROPERTY: &str = "segment.bytes";

pub(crate) fn create_topic(matches: &ArgMatches) -> Result<Topic, String> {
  const REPLICATION_FACTOR: u32 = 3;
  let replication_factor = REPLICATION_FACTOR as i64;
  let partitions = matches.get_one::<u32>(PARTITIONS_FLAG).cloned().unwrap_or(1) as i64;
  let mut kafka_properties = HashMap::new();
  if let Some(cleanup_policy) = matches.get_one::<String>(CLEANUP_POLICY_FLAG) {
    kafka_properties.insert(CLEANUP_POLICY_PROPERTY.to_string(), cleanup_policy.to_string());
  }
  if let Some(compression_type) = matches.get_one::<String>(COMPRESSION_TYPE_FLAG) {
    kafka_properties.insert(COMPRESSION_TYPE_PROPERTY.to_string(), compression_type.to_string());
  }
  if let Some(delete_retention_ms) = matches.get_one::<String>(DELETE_RETENTION_MS_FLAG) {
    kafka_properties.insert(DELETE_RETENTION_MS_PROPERTY.to_string(), delete_retention_ms.to_string());
  }
  if let Some(max_message_bytes) = matches.get_one::<u64>(MAX_MESSAGE_BYTES_FLAG) {
    kafka_properties.insert(MAX_MESSAGE_BYTES_PROPERTY.to_string(), max_message_bytes.to_string());
  }
  if let Some(message_timestamp_type) = matches.get_one::<String>(MESSAGE_TIMESTAMP_TYPE_FLAG) {
    if message_timestamp_type == TIMESTAMP_LOG_APPEND_TIME {
      kafka_properties.insert(MESSAGE_TIMESTAMP_PROPERTY.to_string(), "LogAppendTime".to_string());
    } else if message_timestamp_type == TIMESTAMP_CREATE_TIME {
      kafka_properties.insert(MESSAGE_TIMESTAMP_PROPERTY.to_string(), "CreateTime".to_string());
    }
  }
  if let Some(retention_bytes) = matches.get_one::<u64>(RETENTION_BYTES_FLAG) {
    kafka_properties.insert(RETENTION_BYTES_PROPERTY.to_string(), retention_bytes.to_string());
  }
  if let Some(retention_ms) = matches.get_one::<u64>(RETENTION_MS_FLAG) {
    kafka_properties.insert(RETENTION_MS_PROPERTY.to_string(), retention_ms.to_string());
  }
  if let Some(segment_bytes) = matches.get_one::<u64>(SEGMENT_BYTES_FLAG) {
    kafka_properties.insert(SEGMENT_BYTES_PROPERTY.to_string(), segment_bytes.to_string());
  }
  Ok(Topic { kafka_properties, partitions, replication_factor })
}

struct TopicDelete {}

#[async_trait]
impl CommandExecutor for TopicDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete topic '{}'", topic_id));
    if client.get_topic(&topic_id).await.is_err() {
      return Err(format!("scratch topic '{}' does not exists", topic_id));
    }
    if context.confirmed(format!("delete scratch topic '{}'?", topic_id))? {
      if context.dry_run {
        context.print_warning("dry-run mode, topic not deleted");
      } else {
        client.delete_topic_configuration(&topic_id).await?;
        context.print_outcome(format!("topic '{}' deleted", topic_id));
      }
    } else {
      context.print_outcome(format!("cancelled, topic '{}' not deleted", topic_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicListAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicListAllocationStatus {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topics with their allocation status");
    let start_instant = context.now();
    let topic_ids = client.get_topic_ids().await?;
    let allocation_statuses = try_join_all(topic_ids.iter().map(|topic_id| client.get_topic_status(topic_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("topic id"), context);
    formatter.push_target_ids_and_values(topic_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicListConfiguration {}

#[async_trait]
impl CommandExecutor for TopicListConfiguration {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topics with their configurations");
    let start_instant = context.now();
    let topic_ids = client.get_topic_ids().await?;
    let configurations = try_join_all(topic_ids.iter().map(|topic_id| client.get_topic_configuration(topic_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&TOPIC_LABELS, None, context);
    formatter.push_target_ids_and_values(topic_ids.as_slice(), configurations.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicListIds {}

#[async_trait]
impl CommandExecutor for TopicListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topic ids");
    let start_instant = context.now();
    let topic_ids = client.get_topic_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("topic id", context);
    formatter.push_target_ids(&topic_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicListUsage {}

#[async_trait]
impl CommandExecutor for TopicListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all scratch topics with the services that use them");
    let start_instant = context.now();
    let (topic_ids, services) = try_join!(client.get_topic_ids(), client.get_application_configuration_map(),)?;
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
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicShow {}

#[async_trait]
impl CommandExecutor for TopicShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the configuration for topic '{}'", topic_id));
    let start_instant = context.now();
    let topic = client.get_topic_configuration(&topic_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(topic_id, &TOPIC_STATUS_LABELS, None, context).print(&topic, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for TopicShowAllocationStatus {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the allocation status for topic '{}'", topic_id));
    let start_instant = context.now();
    let allocation_status = client.get_topic_status(&topic_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(topic_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("topic id"), context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicShowProperties {}

#[async_trait]
impl CommandExecutor for TopicShowProperties {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the properties for topic '{}'", topic_id));
    let start_instant = context.now();
    let topic_status = client.get_topic(&topic_id).await?;
    context.print_execution_time(start_instant);
    let mut pairs: Vec<(String, String)> = topic_status.actual.unwrap().kafka_properties.into_iter().collect::<Vec<_>>();
    pairs.sort_by(|(key_a, _), (key_b, _)| key_a.cmp(key_b));
    let (properties, values): (Vec<String>, Vec<String>) = pairs.into_iter().unzip();
    let mut formatter = ListFormatter::new(&PROPERTY_LABELS, Some("property"), context);
    formatter.push_target_ids_and_values(&properties, &values);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TopicShowUsage {}

#[async_trait]
impl CommandExecutor for TopicShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the services that use topic '{}'", topic_id));
    let start_instant = context.now();
    let services = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let usages: Vec<(String, &Application, Vec<Injection>)> = find_applications_that_use_topic(&topic_id, &services);
    let used_bys = usages
      .into_iter()
      .filter_map(|(service_id, service, injections)| if injections.is_empty() { None } else { Some(UsedBy::Application(service_id.clone(), service.instances, injections)) })
      .collect::<Vec<_>>();
    if !used_bys.is_empty() {
      let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("topic id"), context);
      formatter.push_values(&used_bys);
      formatter.print(None)?;
    } else {
      context.print_outcome("topic not used");
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum TopicLabel {
  CleanupPolicy,
  CompressionType,
  DeleteRetentionMs,
  #[allow(dead_code)]
  DerivedFrom,
  KafkaProperties,
  MaxMessageBytes,
  Notifications,
  Partitions,
  Provisioned,
  ReplicationFactor,
  RetentionBytes,
  RetentionMs,
  SegmentBytes,
  Target,
  TimestampType,
}

impl Label for TopicLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup policy",
      Self::CompressionType => "compression type",
      Self::DeleteRetentionMs => "delete retention",
      Self::DerivedFrom => "derived from",
      Self::KafkaProperties => "kafka properties",
      Self::MaxMessageBytes => "max message bytes",
      Self::Notifications => "notifications",
      Self::Partitions => "number of partitions",
      Self::Provisioned => "provisioned",
      Self::ReplicationFactor => "replication factor",
      Self::RetentionBytes => "retention bytes",
      Self::RetentionMs => "retention ms",
      Self::SegmentBytes => "segment bytes",
      Self::Target => "topic id",
      Self::TimestampType => "timestamp type",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::CleanupPolicy => "cleanup",
      Self::CompressionType => "compr",
      Self::DeleteRetentionMs => "del ret",
      Self::DerivedFrom => "derived",
      Self::KafkaProperties => "props",
      Self::MaxMessageBytes => "max bytes",
      Self::Notifications => "not",
      Self::Partitions => "part",
      Self::Provisioned => "prov",
      Self::ReplicationFactor => "repl",
      Self::RetentionBytes => "ret bytes",
      Self::RetentionMs => "ret ms",
      Self::SegmentBytes => "seg bytes",
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
      TopicLabel::CleanupPolicy => self.kafka_properties.get(CLEANUP_POLICY_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::DerivedFrom => "".to_string(),
      TopicLabel::KafkaProperties => hashmap_to_table(&get_implicit_properties(&self.kafka_properties)),
      TopicLabel::MaxMessageBytes => self.kafka_properties.get(MAX_MESSAGE_BYTES_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::Notifications => "".to_string(),
      TopicLabel::Partitions => self.partitions.to_string(),
      TopicLabel::Provisioned => "".to_string(),
      TopicLabel::ReplicationFactor => self.replication_factor.to_string(),
      TopicLabel::SegmentBytes => self.kafka_properties.get(SEGMENT_BYTES_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::Target => target_id.to_string(),
      TopicLabel::TimestampType => self.kafka_properties.get(MESSAGE_TIMESTAMP_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::CompressionType => self.kafka_properties.get(COMPRESSION_TYPE_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::DeleteRetentionMs => self.kafka_properties.get(DELETE_RETENTION_MS_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::RetentionBytes => self.kafka_properties.get(RETENTION_BYTES_PROPERTY).cloned().unwrap_or_default(),
      TopicLabel::RetentionMs => self.kafka_properties.get(RETENTION_MS_PROPERTY).cloned().unwrap_or_default(),
    }
  }
}

pub(crate) fn get_implicit_properties(kafka_properties: &HashMap<String, String>) -> HashMap<&String, &String> {
  const EXPLICIT_VALUES: [&str; 8] = [
    CLEANUP_POLICY_PROPERTY,
    COMPRESSION_TYPE_PROPERTY,
    DELETE_RETENTION_MS_PROPERTY,
    MAX_MESSAGE_BYTES_PROPERTY,
    MESSAGE_TIMESTAMP_PROPERTY,
    RETENTION_BYTES_PROPERTY,
    RETENTION_MS_PROPERTY,
    SEGMENT_BYTES_PROPERTY,
  ];
  kafka_properties
    .iter()
    .filter(|(key, _)| !EXPLICIT_VALUES.contains(&key.as_str()))
    .collect::<HashMap<_, _>>()
}

impl SubjectFormatter<TopicLabel> for TopicStatus {
  fn value(&self, label: &TopicLabel, target_id: &str) -> String {
    match label {
      TopicLabel::CleanupPolicy => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(CLEANUP_POLICY_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::CompressionType => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(COMPRESSION_TYPE_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::DeleteRetentionMs => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(DELETE_RETENTION_MS_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::DerivedFrom => self.status.derived_from.clone().unwrap_or_default(),
      TopicLabel::KafkaProperties => self
        .actual
        .as_ref()
        .map(|topic| hashmap_to_table(&get_implicit_properties(&topic.kafka_properties)))
        .unwrap_or_default(),
      TopicLabel::MaxMessageBytes => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(MAX_MESSAGE_BYTES_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::Notifications => notifications_to_string(&self.status.notifications),
      TopicLabel::Partitions => self.actual.as_ref().map(|a| a.partitions.to_string()).unwrap_or_default(),
      TopicLabel::Provisioned => self.status.provisioned.to_string(),
      TopicLabel::ReplicationFactor => self.actual.as_ref().map(|a| a.replication_factor.to_string()).unwrap_or_default(),
      TopicLabel::RetentionBytes => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(RETENTION_BYTES_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::RetentionMs => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(RETENTION_MS_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::SegmentBytes => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(SEGMENT_BYTES_PROPERTY))
        .cloned()
        .unwrap_or_default(),
      TopicLabel::Target => target_id.to_string(),
      TopicLabel::TimestampType => self
        .actual
        .as_ref()
        .and_then(|topic| topic.kafka_properties.get(MESSAGE_TIMESTAMP_PROPERTY))
        .cloned()
        .unwrap_or_default(),
    }
  }
}

pub static TOPIC_STATUS_LABELS: [TopicLabel; 14] = [
  TopicLabel::Target,
  TopicLabel::Partitions,
  TopicLabel::ReplicationFactor,
  TopicLabel::CleanupPolicy,
  TopicLabel::CompressionType,
  TopicLabel::DeleteRetentionMs,
  TopicLabel::TimestampType,
  TopicLabel::MaxMessageBytes,
  TopicLabel::SegmentBytes,
  TopicLabel::RetentionBytes,
  TopicLabel::RetentionMs,
  TopicLabel::Notifications,
  TopicLabel::Provisioned,
  TopicLabel::KafkaProperties,
];

pub static TOPIC_LABELS: [TopicLabel; 10] = [
  TopicLabel::Target,
  TopicLabel::Partitions,
  TopicLabel::ReplicationFactor,
  TopicLabel::CleanupPolicy,
  TopicLabel::TimestampType,
  TopicLabel::MaxMessageBytes,
  TopicLabel::SegmentBytes,
  TopicLabel::Notifications,
  TopicLabel::Provisioned,
  TopicLabel::KafkaProperties,
];
