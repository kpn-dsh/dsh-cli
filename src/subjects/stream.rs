use crate::subject::Requirements;

use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches};
use futures::{join, try_join};
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{
  ManagedStream, ManagedStreamId, PublicManagedStreamContract, PublicManagedStreamKafkaDefaultPartitioner, PublicManagedStreamKafkaDefaultPartitionerKind,
  PublicManagedStreamTopicLevelPartitioner, PublicManagedStreamTopicLevelPartitionerKind, Topic,
};

use crate::arguments::{managed_stream_argument, MANAGED_STREAM_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::formatter::{hashmap_to_table, Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::subject::Subject;
use crate::subjects::topic::{
  cleanup_policy_flag, compression_type_flag, create_topic, delete_retention_ms_flag, get_implicit_properties, max_message_size_flag, message_timestamp_type_flag, partitions_flag,
  retention_bytes_flag, retention_ms_flag, segment_bytes_flag, CLEANUP_POLICY_PROPERTY, COMPRESSION_TYPE_PROPERTY, DELETE_RETENTION_MS_PROPERTY, MAX_MESSAGE_BYTES_PROPERTY,
  MESSAGE_TIMESTAMP_PROPERTY, RETENTION_BYTES_PROPERTY, RETENTION_MS_PROPERTY, SEGMENT_BYTES_PROPERTY,
};
use crate::{read_single_line, Context, DshCliResult};
use dsh_api::types::{PublicManagedStream, PublicManagedStreamContractPartitioner};
use futures::future::try_join_all;
use serde::Serialize;

pub(crate) struct StreamSubject {}

const STREAM_SUBJECT_TARGET: &str = "stream";

lazy_static! {
  pub static ref STREAM_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(StreamSubject {});
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

const STREAM_OPTIONS_HEADING: &str = "Stream options";

const INTERNAL_FLAG: &str = "internal-flag";
const PUBLIC_FLAG: &str = "public-flag";

lazy_static! {
  static ref STREAM_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, None, &StreamCreate {}, "Create stream")
      .add_extra_argument(
        Arg::new(INTERNAL_FLAG)
          .long("internal")
          .action(ArgAction::SetTrue)
          .help("Create internal managed stream")
      )
      .add_extra_argument(
        Arg::new(PUBLIC_FLAG)
          .long("public")
          .action(ArgAction::SetTrue)
          .help("Create public managed stream")
          .conflicts_with(INTERNAL_FLAG)
      )
      .add_target_argument(managed_stream_argument())
      .add_extra_arguments(vec![
        can_be_retained_flag(STREAM_OPTIONS_HEADING),
        cleanup_policy_flag(STREAM_OPTIONS_HEADING),
        compression_type_flag(STREAM_OPTIONS_HEADING),
        delete_retention_ms_flag(STREAM_OPTIONS_HEADING),
        kafka_default_partitioner_flag(STREAM_OPTIONS_HEADING),
        max_message_size_flag(STREAM_OPTIONS_HEADING),
        message_timestamp_type_flag(STREAM_OPTIONS_HEADING),
        partitions_flag(STREAM_OPTIONS_HEADING),
        retention_bytes_flag(STREAM_OPTIONS_HEADING),
        retention_ms_flag(STREAM_OPTIONS_HEADING),
        segment_bytes_flag(STREAM_OPTIONS_HEADING),
        topic_level_partitioner_arg(STREAM_OPTIONS_HEADING),
      ])
      .set_long_about("Create an internal or public managed stream.")
  );
  static ref STREAM_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &StreamDelete {}, "Delete managed stream")
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
  pub static ref STREAM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &StreamShowAll {}, "Show managed stream configuration")
      .add_target_argument(managed_stream_argument().required(true))
  );
  static ref STREAM_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![STREAM_CREATE_CAPABILITY.as_ref(), STREAM_DELETE_CAPABILITY.as_ref(), STREAM_LIST_CAPABILITY.as_ref(), STREAM_SHOW_CAPABILITY.as_ref()];
}

const CAN_BE_RETAINED_FLAG: &str = "can-be-retained";

fn can_be_retained_flag(heading: &'static str) -> Arg {
  Arg::new(CAN_BE_RETAINED_FLAG)
    .long("can-be-retained")
    .action(ArgAction::SetTrue)
    .help("Can be retained")
    .long_help(
      "Whether MQTT records can have the 'retained' flag. \
    This option is only meaningful for public managed streams.",
    )
    .help_heading(heading)
}

const KAFKA_DEFAULT_PARTITIONER: &str = "kafka-default-partitioner";

fn kafka_default_partitioner_flag(heading: &'static str) -> Arg {
  Arg::new(KAFKA_DEFAULT_PARTITIONER)
    .long("kafka-default-partitioner")
    .action(ArgAction::SetTrue)
    .help("Use kafka default partitioner")
    .long_help(
      "Use the kafka default partitioner to partition messages across different kafka partitions. \
      This option is only meaningful for public managed streams.",
    )
    .help_heading(heading)
}

const TOPIC_LEVEL_PARTITIONER: &str = "topic-level-partitioner";

fn topic_level_partitioner_arg(heading: &'static str) -> Arg {
  Arg::new(TOPIC_LEVEL_PARTITIONER)
    .long("topic-level-partitioner")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..))
    .value_name("LEVEL")
    .help("Use topic level partitioner")
    .long_help(
      "Use the topic level partitioner to partition messages across different kafka partitions. \
      This option is only meaningful for public managed streams.",
    )
    .help_heading(heading)
    .conflicts_with(KAFKA_DEFAULT_PARTITIONER)
}

struct StreamCreate {}

#[async_trait]
impl CommandExecutor for StreamCreate {
  async fn execute_with_client(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let managed_stream_id = get_managed_stream_id(matches, client.tenant_name())?;
    let (internal, public) = join!(
      client.get_stream_internal_configuration(&managed_stream_id),
      client.get_stream_public_configuration(&managed_stream_id)
    );
    if internal.is_ok() {
      return Err(format!("internal managed stream '{}' already exists", managed_stream_id));
    }
    if public.is_ok() {
      return Err(format!("public managed stream '{}' already exists", managed_stream_id));
    }
    let topic = create_topic(matches)?;
    if matches.get_flag(PUBLIC_FLAG) {
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
      if context.dry_run {
        context.print_warning("dry-run mode, public managed stream not created");
      } else {
        client.post_stream_public_configuration(&managed_stream_id, &public_managed_stream).await?;
        context.print_outcome(format!("public managed stream '{}' created", managed_stream_id));
      }
    } else {
      context.print_explanation(format!("create new internal managed stream '{}'", managed_stream_id));
      let managed_stream = ManagedStream(topic);
      if context.dry_run {
        context.print_warning("dry-run mode, internal managed stream not created");
      } else {
        client.post_stream_internal_configuration(&managed_stream_id, &managed_stream).await?;
        context.print_outcome(format!("internal managed stream '{}' created", managed_stream_id));
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn get_managed_stream_id(matches: &ArgMatches, managing_tenant: &str) -> Result<ManagedStreamId, String> {
  match matches.get_one::<String>(MANAGED_STREAM_ARGUMENT) {
    Some(managed_stream_argument) => Ok(ManagedStreamId::try_from(managed_stream_argument).map_err(|error| error.to_string())?),
    None => {
      let line = read_single_line(format!("enter managed stream id: {}---", managing_tenant))?;
      let managed_stream_id = format!("{}---{}", managing_tenant, line);
      let managed_stream_id = ManagedStreamId::try_from(managed_stream_id).map_err(|error| error.to_string())?;
      Ok(managed_stream_id)
    }
  }
}

struct StreamDelete {}

#[async_trait]
impl CommandExecutor for StreamDelete {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let managed_stream_id = get_managed_stream_id(matches, client.tenant_name())?;
    match join!(
      client.get_stream_internal_configuration(&managed_stream_id),
      client.get_stream_public_configuration(&managed_stream_id)
    ) {
      (Err(_), Err(_)) => return Err(format!("managed stream '{}' does not exist", managed_stream_id)),
      (Err(_), Ok(_)) => {
        context.print_explanation(format!("delete public managed stream '{}'", managed_stream_id));
        if context.confirmed(format!("delete public managed stream '{}'?", managed_stream_id))? {
          if context.dry_run {
            context.print_warning("dry-run mode, public managed stream not deleted");
          } else {
            client.delete_stream_public_configuration(&managed_stream_id).await?;
            context.print_outcome(format!("public managed stream '{}' deleted", managed_stream_id));
          }
        } else {
          context.print_outcome(format!("cancelled, public managed stream '{}' not deleted", managed_stream_id));
        }
      }
      (Ok(_), Err(_)) => {
        context.print_explanation(format!("delete internal managed stream '{}'", managed_stream_id));
        if context.confirmed(format!("delete internal managed stream '{}'?", managed_stream_id))? {
          if context.dry_run {
            context.print_warning("dry-run mode, internal managed stream not deleted");
          } else {
            client.delete_stream_internal_configuration(&managed_stream_id).await?;
            context.print_outcome(format!("internal managed stream '{}' deleted", managed_stream_id));
          }
        } else {
          context.print_outcome(format!("cancelled, internal managed stream '{}' not deleted", managed_stream_id));
        }
      }
      (Ok(_), Ok(_)) => unreachable!(),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct StreamListAll {}

#[async_trait]
impl CommandExecutor for StreamListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let start_instant = context.now();
    let (internals, publics) = match (matches.get_flag(INTERNAL_FLAG), matches.get_flag(PUBLIC_FLAG)) {
      (false, false) | (true, true) => {
        context.print_explanation("list all internal and public managed streams");
        let (internal_ids, public_ids) = try_join!(client.get_stream_internals(), client.get_stream_publics())?;
        let (internal_streams, public_streams) = try_join!(
          try_join_all(internal_ids.iter().map(|isi| client.get_stream_internal_configuration(isi))),
          try_join_all(public_ids.iter().map(|psi| client.get_stream_public_configuration(psi)))
        )?;
        (Some((internal_ids, internal_streams)), Some((public_ids, public_streams)))
      }
      (false, true) => {
        context.print_explanation("list all public managed streams");
        let public_ids = client.get_stream_publics().await?;
        let public_streams = try_join_all(public_ids.iter().map(|psi| client.get_stream_public_configuration(psi))).await?;
        (None, Some((public_ids, public_streams)))
      }
      (true, false) => {
        context.print_explanation("list all internal managed streams");
        let internal_ids = client.get_stream_internals().await?;
        let internal_streams = try_join_all(internal_ids.iter().map(|isi| client.get_stream_internal_configuration(isi))).await?;
        (Some((internal_ids, internal_streams)), None)
      }
    };
    context.print_execution_time(start_instant);

    let internals_zipped = internals.map(|(internal_ids, internal_streams)| {
      let internal_streams = internal_streams.into_iter().map(FormatterManagedStream::Internal).collect::<Vec<_>>();
      internal_ids.into_iter().zip(internal_streams).collect::<Vec<_>>()
    });

    let publics_zipped = publics.map(|(public_ids, public_streams)| {
      let public_streams = public_streams.into_iter().map(FormatterManagedStream::Public).collect::<Vec<_>>();
      public_ids.into_iter().zip(public_streams).collect::<Vec<_>>()
    });

    let mut formatter =
      if publics_zipped.is_some() { ListFormatter::new(&LIST_PUBLIC_STREAM_LABELS, None, context) } else { ListFormatter::new(&LIST_INTERNAL_STREAM_LABELS, None, context) };
    if let Some(internals) = &internals_zipped {
      for (internal_id, internal_stream) in internals.iter() {
        formatter.push_target_id_value(internal_id.to_string(), internal_stream);
      }
    }
    if let Some(publics) = &publics_zipped {
      for (public_id, public_stream) in publics.iter() {
        formatter.push_target_id_value(public_id.to_string(), public_stream);
      }
    }

    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct StreamListIds {}

#[async_trait]
impl CommandExecutor for StreamListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let start_instant = context.now();
    let stream_ids = match (matches.get_flag(INTERNAL_FLAG), matches.get_flag(PUBLIC_FLAG)) {
      (false, false) | (true, true) => {
        context.print_explanation("list all internal and public managed stream ids");
        let (mut stream_ids, mut public_ids) = try_join!(client.get_stream_internals(), client.get_stream_publics())?;
        stream_ids.append(&mut public_ids);
        stream_ids
      }
      (false, true) => {
        context.print_explanation("list all public managed stream ids");
        client.get_stream_publics().await?
      }
      (true, false) => {
        context.print_explanation("list all internal managed stream ids");
        client.get_stream_internals().await?
      }
    };
    context.print_execution_time(start_instant);
    let mut stream_ids = stream_ids.iter().map(|msi| msi.to_string()).collect::<Vec<_>>();
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

struct StreamShowAll {}

#[async_trait]
impl CommandExecutor for StreamShowAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let managed_stream_id = get_managed_stream_id(matches, client.tenant_name())?;
    context.print_explanation(format!("show configuration for managed stream '{}'", managed_stream_id));
    let start_instant = context.now();
    match join!(
      client.get_stream_internal_configuration(&managed_stream_id),
      client.get_stream_public_configuration(&managed_stream_id)
    ) {
      (Err(_), Err(_)) => {
        context.print_execution_time(start_instant);
        return Err(format!("managed stream '{}' does not exist", managed_stream_id));
      }
      (Err(_), Ok(public_managed_stream)) => {
        context.print_execution_time(start_instant);
        UnitFormatter::new(managed_stream_id, &PUBLIC_STREAM_LABELS, None, context).print(&FormatterManagedStream::Public(public_managed_stream), None)?
      }
      (Ok(internal_managed_stream), Err(_)) => {
        context.print_execution_time(start_instant);
        UnitFormatter::new(managed_stream_id, &INTERNAL_STREAM_LABELS, None, context).print(&FormatterManagedStream::Internal(internal_managed_stream), None)?
      }
      (Ok(_), Ok(_)) => unreachable!(),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum ManagedStreamLabel {
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
      Self::TimestampType => "ts",
      Self::Type => "type",
    }
  }

  fn is_target_label(&self) -> bool {
    *self == ManagedStreamLabel::Target
  }
}

#[derive(Serialize)]
pub enum FormatterManagedStream {
  Internal(ManagedStream),
  Public(PublicManagedStream),
}

impl SubjectFormatter<ManagedStreamLabel> for FormatterManagedStream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
    match self {
      FormatterManagedStream::Internal(internal) => topic_value(&internal.0, label, target_id),
      FormatterManagedStream::Public(public) => public.value(label, target_id),
    }
  }
}

fn topic_value(topic: &Topic, label: &ManagedStreamLabel, target_id: &str) -> String {
  match label {
    ManagedStreamLabel::CanBeRetained => "".to_string(),
    ManagedStreamLabel::CleanupPolicy => topic.kafka_properties.get(CLEANUP_POLICY_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::CompressionType => topic.kafka_properties.get(COMPRESSION_TYPE_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::DeleteRetentionMs => topic.kafka_properties.get(DELETE_RETENTION_MS_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::KafkaProperties => hashmap_to_table(&get_implicit_properties(&topic.kafka_properties)),
    ManagedStreamLabel::MaxMessageBytes => topic.kafka_properties.get(MAX_MESSAGE_BYTES_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::Partitioner => "".to_string(),
    ManagedStreamLabel::Partitions => topic.partitions.to_string(),
    ManagedStreamLabel::ReplicationFactor => topic.replication_factor.to_string(),
    ManagedStreamLabel::RetentionBytes => topic.kafka_properties.get(RETENTION_BYTES_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::RetentionMs => topic.kafka_properties.get(RETENTION_MS_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::SegmentBytes => topic.kafka_properties.get(SEGMENT_BYTES_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::Target => target_id.to_string(),
    ManagedStreamLabel::TimestampType => topic.kafka_properties.get(MESSAGE_TIMESTAMP_PROPERTY).cloned().unwrap_or_default(),
    ManagedStreamLabel::Type => "internal".to_string(),
  }
}

impl SubjectFormatter<ManagedStreamLabel> for PublicManagedStream {
  fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
    match label {
      ManagedStreamLabel::CanBeRetained => self.contract.can_be_retained.to_string(),
      ManagedStreamLabel::CleanupPolicy => self.kafka_properties.get(CLEANUP_POLICY_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::CompressionType => self.kafka_properties.get(COMPRESSION_TYPE_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::DeleteRetentionMs => self.kafka_properties.get(DELETE_RETENTION_MS_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::KafkaProperties => hashmap_to_table(&get_implicit_properties(&self.kafka_properties)),
      ManagedStreamLabel::MaxMessageBytes => self.kafka_properties.get(MAX_MESSAGE_BYTES_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::Partitioner => match self.contract.partitioner {
        PublicManagedStreamContractPartitioner::TopicLevelPartitioner(ref topic_level_partitioner) => format!("topic level {}", topic_level_partitioner.topic_level),
        PublicManagedStreamContractPartitioner::KafkaDefaultPartitioner(_) => "kafka default".to_string(),
      },
      ManagedStreamLabel::Partitions => self.partitions.to_string(),
      ManagedStreamLabel::ReplicationFactor => self.replication_factor.to_string(),
      ManagedStreamLabel::RetentionBytes => self.kafka_properties.get(RETENTION_BYTES_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::RetentionMs => self.kafka_properties.get(RETENTION_MS_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::SegmentBytes => self.kafka_properties.get(SEGMENT_BYTES_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::Target => target_id.to_string(),
      ManagedStreamLabel::TimestampType => self.kafka_properties.get(MESSAGE_TIMESTAMP_PROPERTY).cloned().unwrap_or_default(),
      ManagedStreamLabel::Type => "public".to_string(),
    }
  }
}

pub static INTERNAL_STREAM_LABELS: [ManagedStreamLabel; 13] = [
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
];

pub static PUBLIC_STREAM_LABELS: [ManagedStreamLabel; 15] = [
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
];

pub static LIST_INTERNAL_STREAM_LABELS: [ManagedStreamLabel; 8] = [
  ManagedStreamLabel::Target,
  ManagedStreamLabel::Type,
  ManagedStreamLabel::Partitions,
  ManagedStreamLabel::ReplicationFactor,
  ManagedStreamLabel::CleanupPolicy,
  ManagedStreamLabel::MaxMessageBytes,
  ManagedStreamLabel::SegmentBytes,
  ManagedStreamLabel::TimestampType,
];

pub static LIST_PUBLIC_STREAM_LABELS: [ManagedStreamLabel; 10] = [
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
