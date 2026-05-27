use crate::subject::Requirements;

use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::AccessRights;
use futures::try_join;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::DshApiError;
use dsh_api::stream::Stream;
use dsh_api::types::{
  ManagedStream, ManagedStreamId, PublicManagedStreamContract, PublicManagedStreamKafkaDefaultPartitioner, PublicManagedStreamKafkaDefaultPartitionerKind,
  PublicManagedStreamTopicLevelPartitioner, PublicManagedStreamTopicLevelPartitionerKind,
};

use crate::arguments::{managed_stream_argument, MANAGED_STREAM_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, DELETE_COMMAND, DELETE_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::error::DshCliError;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{hashmap_to_table, Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::subject::Subject;
use crate::subjects::topic::{
  cleanup_policy_flag, compression_type_flag, create_topic, delete_retention_ms_flag, get_implicit_properties, max_message_size_flag, message_timestamp_type_flag, partitions_flag,
  retention_bytes_flag, retention_ms_flag, segment_bytes_flag, CLEANUP_POLICY_PROPERTY, COMPRESSION_TYPE_PROPERTY, DELETE_RETENTION_MS_PROPERTY, MAX_MESSAGE_BYTES_PROPERTY,
  MESSAGE_TIMESTAMP_PROPERTY, RETENTION_BYTES_PROPERTY, RETENTION_MS_PROPERTY, SEGMENT_BYTES_PROPERTY,
};
use crate::{err, error_map, plain, read_single_line, Context, DshCliResult};
use dsh_api::types::{PublicManagedStream, PublicManagedStreamContractPartitioner};
use itertools::Itertools;
use serde::Serialize;

struct StreamSubject {}

const STREAM_SUBJECT_TARGET: &str = "stream";

lazy_static! {
  pub(crate) static ref STREAM_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(StreamSubject {});
}

#[async_trait]
impl Subject for StreamSubject {
  fn subject(&self) -> &'static str {
    STREAM_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list internal and public managed streams.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list internal and public managed streams deployed on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(STREAM_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(STREAM_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(STREAM_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(STREAM_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &STREAM_CAPABILITIES
  }
}

const CREATE_INTERNAL_FLAG: &str = "create-internal-flag";
const CREATE_PUBLIC_FLAG: &str = "create-public-flag";

lazy_static! {
  static ref STREAM_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, None, &StreamCreate {}, "Create stream")
      .add_extra_argument(
        Arg::new(CREATE_INTERNAL_FLAG)
          .long("internal")
          .action(ArgAction::SetTrue)
          .help("Create internal managed stream")
      )
      .add_extra_argument(
        Arg::new(CREATE_PUBLIC_FLAG)
          .long("public")
          .action(ArgAction::SetTrue)
          .help("Create public managed stream")
          .conflicts_with(CREATE_INTERNAL_FLAG)
      )
      .add_target_argument(managed_stream_argument())
      .add_extra_arguments(vec![
        can_be_retained_flag(),
        cleanup_policy_flag(),
        compression_type_flag(),
        delete_retention_ms_flag(),
        kafka_default_partitioner_flag(),
        max_message_size_flag(),
        message_timestamp_type_flag(),
        partitions_flag(),
        retention_bytes_flag(),
        retention_ms_flag(),
        segment_bytes_flag(),
        topic_level_partitioner_arg(),
      ])
      .set_long_about("Create an internal or public managed stream.")
  );
  static ref STREAM_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &StreamDelete {}, "Delete managed stream")
      .set_long_about("Delete an internal or public managed stream.")
      .add_target_argument(managed_stream_argument().required(true))
  );
  static ref STREAM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &StreamListAll {}, "List streams")
      .set_long_about("Lists all available internal and public managed streams.")
      .add_filter_flags(vec![
        (FilterFlagType::Internal, Some("List all internal managed streams.".to_string())),
        (FilterFlagType::Public, Some("List all public managed streams.".to_string()))
      ])
      .add_command_executor(FlagType::Ids, &StreamListIds {}, None)
  );
  static ref STREAM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &StreamShow {}, "Show managed stream configuration")
      .add_target_argument(managed_stream_argument().required(true))
  );
  static ref STREAM_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![STREAM_CREATE_CAPABILITY.as_ref(), STREAM_DELETE_CAPABILITY.as_ref(), STREAM_LIST_CAPABILITY.as_ref(), STREAM_SHOW_CAPABILITY.as_ref()];
}

const CAN_BE_RETAINED_FLAG: &str = "can-be-retained";

fn can_be_retained_flag() -> Arg {
  Arg::new(CAN_BE_RETAINED_FLAG)
    .long("can-be-retained")
    .action(ArgAction::SetTrue)
    .help("Can be retained")
    .long_help(
      "Whether MQTT records can have the 'retained' flag. \
    This option is only meaningful for public managed streams.",
    )
}

const KAFKA_DEFAULT_PARTITIONER: &str = "kafka-default-partitioner";

fn kafka_default_partitioner_flag() -> Arg {
  Arg::new(KAFKA_DEFAULT_PARTITIONER)
    .long("kafka-default-partitioner")
    .action(ArgAction::SetTrue)
    .help("Use Kafka default partitioner")
    .long_help(
      "Use the Kafka default partitioner to partition messages across different Kafka partitions. \
      This option is only meaningful for public managed streams.",
    )
}

const TOPIC_LEVEL_PARTITIONER: &str = "topic-level-partitioner";

fn topic_level_partitioner_arg() -> Arg {
  Arg::new(TOPIC_LEVEL_PARTITIONER)
    .long("topic-level-partitioner")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..))
    .value_name("LEVEL")
    .help("Use topic level partitioner")
    .long_help(
      "Use the topic level partitioner to partition messages across different Kafka partitions. \
      This option is only meaningful for public managed streams.",
    )
    .conflicts_with(KAFKA_DEFAULT_PARTITIONER)
}

struct StreamCreate {}

#[async_trait]
impl CommandExecutor for StreamCreate {
  async fn execute_with_client(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let managed_stream_id = get_managed_stream_id(matches, client.tenant_name())?;
    if let Some(managed_stream) = client.managed_stream_configuration(&managed_stream_id).await? {
      match managed_stream {
        Stream::Internal { .. } => return err!("internal managed stream '{}' already exists", managed_stream_id),
        Stream::Public { .. } => return err!("public managed stream '{}' already exists", managed_stream_id),
      }
    }
    let topic = create_topic(matches)?;
    if matches.get_flag(CREATE_PUBLIC_FLAG) {
      let partitioner = match matches.get_one::<i64>(TOPIC_LEVEL_PARTITIONER) {
        Some(topic_level) => PublicManagedStreamContractPartitioner::TopicLevelPartitioner(PublicManagedStreamTopicLevelPartitioner {
          kind: PublicManagedStreamTopicLevelPartitionerKind::TopicLevel,
          topic_level: *topic_level,
        }),
        None => PublicManagedStreamContractPartitioner::KafkaDefaultPartitioner(PublicManagedStreamKafkaDefaultPartitioner {
          kind: PublicManagedStreamKafkaDefaultPartitionerKind::KafkaDefault,
        }),
      };
      context.print_explanation(format!("create new public managed stream '{}'", managed_stream_id));
      let can_be_retained = matches.get_flag(CAN_BE_RETAINED_FLAG);
      let contract = PublicManagedStreamContract { can_be_retained, partitioner };
      let public_managed_stream =
        PublicManagedStream { contract, kafka_properties: topic.kafka_properties, partitions: topic.partitions, replication_factor: topic.replication_factor };
      if context.dry_run() {
        context.print_warning("dry-run mode, public managed stream not created");
      } else {
        client.post_stream_public_configuration(managed_stream_id.as_str(), &public_managed_stream).await?;
        context.print_outcome(format!("public managed stream '{}' created", managed_stream_id));
      }
    } else {
      context.print_explanation(format!("create new internal managed stream '{}'", managed_stream_id));
      let managed_stream = ManagedStream(topic);
      if context.dry_run() {
        context.print_warning("dry-run mode, internal managed stream not created");
      } else {
        client.post_stream_internal_configuration(managed_stream_id.as_str(), &managed_stream).await?;
        context.print_outcome(format!("internal managed stream '{}' created", managed_stream_id));
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct StreamDelete {}

#[async_trait]
impl CommandExecutor for StreamDelete {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let managed_stream_id = get_managed_stream_id(matches, client.tenant_name())?;
    match client.managed_stream_configuration(&managed_stream_id).await? {
      Some(Stream::Internal { .. }) => {
        if context.confirmed(format!("delete internal managed stream '{}'?", managed_stream_id))? {
          if context.dry_run() {
            context.print_warning("dry-run mode, internal managed stream not deleted");
          } else {
            client.delete_stream_internal_configuration(managed_stream_id.as_str()).await?;
            context.print_outcome(format!("internal managed stream '{}' deleted", managed_stream_id));
          }
        } else {
          context.print_outcome(format!("cancelled, internal managed stream '{}' not deleted", managed_stream_id));
        }
        Ok(())
      }
      Some(Stream::Public { .. }) => {
        if context.confirmed(format!("delete public managed stream '{}'?", managed_stream_id))? {
          if context.dry_run() {
            context.print_warning("dry-run mode, public managed stream not deleted");
          } else {
            client.delete_stream_public_configuration(managed_stream_id.as_str()).await?;
            context.print_outcome(format!("public managed stream '{}' deleted", managed_stream_id));
          }
        } else {
          context.print_outcome(format!("cancelled, public managed stream '{}' not deleted", managed_stream_id));
        }
        Ok(())
      }
      None => err!("managed stream '{}' does not exist", managed_stream_id),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static LIST_PUBLIC_STREAM_LABELS: [ManagedStreamLabel; 10] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Type,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::CleanupPolicy,
  ManagedStreamLabel::MaxMessageBytes,
  ManagedStreamLabel::SegmentBytes,
  ManagedStreamLabel::TimestampType,
  ManagedStreamLabel::Partitioner,
  ManagedStreamLabel::CanBeRetained,
];
static LIST_INTERNAL_STREAM_LABELS: [ManagedStreamLabel; 8] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Type,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::CleanupPolicy,
  ManagedStreamLabel::MaxMessageBytes,
  ManagedStreamLabel::SegmentBytes,
  ManagedStreamLabel::TimestampType,
];

struct StreamListAll {}

#[async_trait]
impl CommandExecutor for StreamListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    match (matches.get_flag(FilterFlagType::Internal.id()), matches.get_flag(FilterFlagType::Public.id())) {
      (false, false) | (true, true) => {
        context.print_explanation("list all internal and public managed streams");
        let start_instant = context.now();
        let streams = client.managed_stream_configurations().await?;
        context.print_execution_time(start_instant);
        let mut formatter = if streams.iter().any(|(_, stream)| matches!(stream, Stream::Public { .. })) {
          ListFormatter::new(&LIST_PUBLIC_STREAM_LABELS, context)
        } else {
          ListFormatter::new(&LIST_INTERNAL_STREAM_LABELS, context)
        };
        for (stream_id, stream) in streams.iter() {
          formatter.push_target_id_value(stream_id.to_string(), stream);
        }
        formatter.print(None)
      }
      (true, false) => {
        context.print_explanation("list all internal managed streams");
        let start_instant = context.now();
        let internal_streams = client.managed_stream_configurations_internal().await?;
        context.print_execution_time(start_instant);
        let mut formatter = ListFormatter::new(&LIST_INTERNAL_STREAM_LABELS, context);
        for (internal_stream_id, internal_stream) in internal_streams.iter() {
          formatter.push_target_id_value(internal_stream_id.to_string(), internal_stream);
        }
        formatter.print(None)
      }
      (false, true) => {
        context.print_explanation("list all public managed streams");
        let start_instant = context.now();
        let public_streams = client.managed_stream_configurations_public().await?;
        context.print_execution_time(start_instant);
        let mut formatter = ListFormatter::new(&LIST_PUBLIC_STREAM_LABELS, context);
        for (public_stream_id, public_stream) in public_streams.iter() {
          formatter.push_target_id_value(public_stream_id.to_string(), public_stream);
        }
        formatter.print(None)
      }
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct StreamListIds {}

#[async_trait]
impl CommandExecutor for StreamListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let start_instant = context.now();
    let stream_ids = match (matches.get_flag(FilterFlagType::Internal.id()), matches.get_flag(FilterFlagType::Public.id())) {
      (false, false) | (true, true) => {
        context.print_explanation("list all internal and public managed stream ids");
        let (mut stream_ids, mut public_ids) = try_join!(client.get_stream_internals(), client.get_stream_publics())?;
        stream_ids.append(&mut public_ids);
        stream_ids
      }
      (true, false) => {
        context.print_explanation("list all internal managed stream ids");
        client.get_stream_internals().await?
      }
      (false, true) => {
        context.print_explanation("list all public managed stream ids");
        client.get_stream_publics().await?
      }
    };
    context.print_execution_time(start_instant);
    let mut stream_ids = stream_ids.iter().map(|msi| msi.to_string()).collect_vec();
    stream_ids.sort();
    let mut formatter = IdsFormatter::new("stream id", context);
    formatter.push_target_ids(&stream_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static INTERNAL_STREAM_LABELS: [ManagedStreamLabel; 15] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Type,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::CleanupPolicy,
  ManagedStreamLabel::CompressionType,
  ManagedStreamLabel::DeleteRetentionMs,
  ManagedStreamLabel::MaxMessageBytes,
  ManagedStreamLabel::SegmentBytes,
  ManagedStreamLabel::TimestampType,
  ManagedStreamLabel::RetentionBytes,
  ManagedStreamLabel::RetentionMs,
  ManagedStreamLabel::KafkaProperties,
  ManagedStreamLabel::TenantsGrantedReadAccess,
  ManagedStreamLabel::TenantsGrantedWriteAccess,
];
static PUBLIC_STREAM_LABELS: [ManagedStreamLabel; 17] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Type,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::CleanupPolicy,
  ManagedStreamLabel::CompressionType,
  ManagedStreamLabel::DeleteRetentionMs,
  ManagedStreamLabel::MaxMessageBytes,
  ManagedStreamLabel::SegmentBytes,
  ManagedStreamLabel::TimestampType,
  ManagedStreamLabel::RetentionBytes,
  ManagedStreamLabel::RetentionMs,
  ManagedStreamLabel::KafkaProperties,
  ManagedStreamLabel::Partitioner,
  ManagedStreamLabel::CanBeRetained,
  ManagedStreamLabel::TenantsGrantedReadAccess,
  ManagedStreamLabel::TenantsGrantedWriteAccess,
];

struct StreamShow {}

#[async_trait]
impl CommandExecutor for StreamShow {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let managed_stream_id = get_managed_stream_id(matches, client.tenant_name())?;
    context.print_explanation(format!("show configuration for managed stream '{}'", managed_stream_id));
    let start_instant = context.now();
    match try_join!(
      client.managed_stream_configuration(&managed_stream_id),
      client.managed_stream_tenants_with_access_rights(&managed_stream_id)
    ) {
      Ok((Some(Stream::Internal { internal_stream }), access_rights)) => {
        context.print_execution_time(start_instant);
        UnitFormatter::new(managed_stream_id, &INTERNAL_STREAM_LABELS, context).print(&(Stream::Internal { internal_stream }, &access_rights), None)
      }
      Ok((Some(Stream::Public { public_stream }), access_rights)) => {
        context.print_execution_time(start_instant);
        UnitFormatter::new(managed_stream_id, &PUBLIC_STREAM_LABELS, context).print(&(Stream::Public { public_stream }, &access_rights), None)
      }
      Ok((None, _)) => {
        context.print_error(format!("stream '{}' does not exist", managed_stream_id));
        Ok(())
      }
      Err(error) => match error {
        DshApiError::NotFound { .. } => {
          context.print_error(format!("stream '{}' does not exist", managed_stream_id));
          Ok(())
        }
        DshApiError::BadRequest { .. } => {
          context.print_error(format!("you are not authorized to manage stream '{}'", managed_stream_id));
          Ok(())
        }
        error => Err(DshCliError::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum ManagedStreamLabel {
  CanBeRetained,
  CleanupPolicy,
  CompressionType,
  DeleteRetentionMs,
  KafkaProperties,
  MaxMessageBytes,
  Partitioner,
  Partitions,
  ReplicationFactor,
  RetentionBytes,
  RetentionMs,
  SegmentBytes,
  Target,
  TenantsGrantedReadAccess,
  TenantsGrantedWriteAccess,
  TimestampType,
  Type,
}

impl Label for ManagedStreamLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CanBeRetained => "can be retained",
      Self::CleanupPolicy => "cleanup policy",
      Self::CompressionType => "compression type",
      Self::DeleteRetentionMs => "delete retention",
      Self::KafkaProperties => "kafka properties",
      Self::MaxMessageBytes => "max message bytes",
      Self::Partitioner => "partitioner",
      Self::Partitions => "number of partitions",
      Self::ReplicationFactor => "replication factor",
      Self::RetentionBytes => "retention bytes",
      Self::RetentionMs => "retention ms",
      Self::SegmentBytes => "segment bytes",
      Self::Target => "stream id",
      Self::TenantsGrantedReadAccess => "tenants granted read access",
      Self::TenantsGrantedWriteAccess => "tenants granted write access",
      Self::TimestampType => "timestamp type",
      Self::Type => "type",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::CanBeRetained => "retained",
      Self::CleanupPolicy => "cleanup",
      Self::CompressionType => "compression",
      Self::DeleteRetentionMs => "delete ret",
      Self::KafkaProperties => "props",
      Self::MaxMessageBytes => "max bytes",
      Self::Partitioner => "partitioner",
      Self::Partitions => "partitions",
      Self::ReplicationFactor => "repl",
      Self::RetentionBytes => "ret bytes",
      Self::RetentionMs => "ret ms",
      Self::SegmentBytes => "seg bytes",
      Self::Target => "id",
      Self::TenantsGrantedReadAccess => "read",
      Self::TenantsGrantedWriteAccess => "write",
      Self::TimestampType => "ts",
      Self::Type => "type",
    }
  }

  fn is_target_label(&self) -> bool {
    *self == ManagedStreamLabel::Target
  }
}

impl SubjectFormatter<ManagedStreamLabel> for Stream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> Value {
    match self {
      Stream::Internal { internal_stream } => internal_stream.value(label, target_id),
      Stream::Public { public_stream } => public_stream.value(label, target_id),
    }
  }
}

impl SubjectFormatter<ManagedStreamLabel> for ManagedStream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> Value {
    match label {
      ManagedStreamLabel::CanBeRetained => Value::plain("NA"),
      ManagedStreamLabel::CleanupPolicy => Value::some_or_hide(self.0.kafka_properties.get(CLEANUP_POLICY_PROPERTY).cloned()),
      ManagedStreamLabel::CompressionType => Value::some_or_hide(self.0.kafka_properties.get(COMPRESSION_TYPE_PROPERTY).cloned()),
      ManagedStreamLabel::DeleteRetentionMs => Value::some_or_hide(self.0.kafka_properties.get(DELETE_RETENTION_MS_PROPERTY).cloned()),
      ManagedStreamLabel::KafkaProperties => Value::plain(hashmap_to_table(&get_implicit_properties(&self.0.kafka_properties))),
      ManagedStreamLabel::MaxMessageBytes => Value::some_or_hide(self.0.kafka_properties.get(MAX_MESSAGE_BYTES_PROPERTY).cloned()),
      ManagedStreamLabel::Partitioner => Value::plain("NA"),
      ManagedStreamLabel::Partitions => Value::plain(self.0.partitions),
      ManagedStreamLabel::ReplicationFactor => Value::plain(self.0.replication_factor),
      ManagedStreamLabel::RetentionBytes => Value::some_or_hide(self.0.kafka_properties.get(RETENTION_BYTES_PROPERTY).cloned()),
      ManagedStreamLabel::RetentionMs => Value::some_or_hide(self.0.kafka_properties.get(RETENTION_MS_PROPERTY).cloned()),
      ManagedStreamLabel::SegmentBytes => Value::some_or_hide(self.0.kafka_properties.get(SEGMENT_BYTES_PROPERTY).cloned()),
      ManagedStreamLabel::Target => Value::target(target_id),
      ManagedStreamLabel::TimestampType => Value::some_or_hide(self.0.kafka_properties.get(MESSAGE_TIMESTAMP_PROPERTY).cloned()),
      ManagedStreamLabel::Type => Value::plain("internal"),
      _ => unreachable!("label '{}' was not expected", label.as_str()),
    }
  }
}

impl SubjectFormatter<ManagedStreamLabel> for PublicManagedStream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> Value {
    match label {
      ManagedStreamLabel::CanBeRetained => Value::plain(self.contract.can_be_retained),
      ManagedStreamLabel::CleanupPolicy => Value::some_or_hide(self.kafka_properties.get(CLEANUP_POLICY_PROPERTY).cloned()),
      ManagedStreamLabel::CompressionType => Value::some_or_hide(self.kafka_properties.get(COMPRESSION_TYPE_PROPERTY).cloned()),
      ManagedStreamLabel::DeleteRetentionMs => Value::some_or_hide(self.kafka_properties.get(DELETE_RETENTION_MS_PROPERTY).cloned()),
      ManagedStreamLabel::KafkaProperties => Value::plain(hashmap_to_table(&get_implicit_properties(&self.kafka_properties))),
      ManagedStreamLabel::MaxMessageBytes => Value::some_or_hide(self.kafka_properties.get(MAX_MESSAGE_BYTES_PROPERTY).cloned()),
      ManagedStreamLabel::Partitioner => match self.contract.partitioner {
        PublicManagedStreamContractPartitioner::TopicLevelPartitioner(ref topic_level_partitioner) => plain!("topic level {}", topic_level_partitioner.topic_level),
        PublicManagedStreamContractPartitioner::KafkaDefaultPartitioner(_) => Value::plain("kafka default"),
      },
      ManagedStreamLabel::Partitions => Value::plain(self.partitions),
      ManagedStreamLabel::ReplicationFactor => Value::plain(self.replication_factor),
      ManagedStreamLabel::RetentionBytes => Value::some_or_hide(self.kafka_properties.get(RETENTION_BYTES_PROPERTY).cloned()),
      ManagedStreamLabel::RetentionMs => Value::some_or_hide(self.kafka_properties.get(RETENTION_MS_PROPERTY).cloned()),
      ManagedStreamLabel::SegmentBytes => Value::some_or_hide(self.kafka_properties.get(SEGMENT_BYTES_PROPERTY).cloned()),
      ManagedStreamLabel::Target => Value::target(target_id),
      ManagedStreamLabel::TimestampType => Value::some_or_hide(self.kafka_properties.get(MESSAGE_TIMESTAMP_PROPERTY).cloned()),
      ManagedStreamLabel::Type => Value::plain("public"),
      _ => unreachable!("label '{}' was not expected", label.as_str()),
    }
  }
}

impl SubjectFormatter<ManagedStreamLabel> for (Stream, &Vec<(String, AccessRights)>) {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> Value {
    match label {
      ManagedStreamLabel::TenantsGrantedReadAccess => Value::plain(
        self
          .1
          .iter()
          .filter(|(_, access_rights)| access_rights == &AccessRights::Read || access_rights == &AccessRights::ReadWrite)
          .map(|(tenant_id, _)| tenant_id)
          .join(", "),
      ),
      ManagedStreamLabel::TenantsGrantedWriteAccess => Value::plain(
        self
          .1
          .iter()
          .filter(|(_, access_rights)| access_rights == &AccessRights::Write || access_rights == &AccessRights::ReadWrite)
          .map(|(tenant_id, _)| tenant_id)
          .join(", "),
      ),
      _ => self.0.value(label, target_id),
    }
  }
}

fn get_managed_stream_id(matches: &ArgMatches, managing_tenant: &str) -> DshCliResult<ManagedStreamId> {
  match matches.get_one::<String>(MANAGED_STREAM_ARGUMENT) {
    Some(managed_stream_argument) => Ok(ManagedStreamId::try_from(managed_stream_argument).map_err(error_map!("{}"))?),
    None => {
      let line = read_single_line(format!("enter managed stream id: {}---", managing_tenant))?;
      let managed_stream_id = format!("{}---{}", managing_tenant, line);
      let managed_stream_id = ManagedStreamId::try_from(managed_stream_id).map_err(error_map!("{}"))?;
      Ok(managed_stream_id)
    }
  }
}
