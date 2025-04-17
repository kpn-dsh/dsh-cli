use crate::arguments::managed_tenant_name_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS, UPDATE_COMMAND};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::limits_flags::{
  certificate_count_flag, consumer_rate_flag, cpu_flag, mem_flag, partition_count_flag, producer_rate_flag, request_rate_flag, secret_count_flag, topic_count_flag,
  CERTIFICATE_COUNT_FLAG, CONSUMER_RATE_FLAG, CPU_FLAG, KAFKA_ACL_GROUP_COUNT_FLAG, MEM_FLAG, PARTITION_COUNT_FLAG, PRODUCER_RATE_FLAG, REQUEST_RATE_FLAG, SECRET_COUNT_FLAG,
  TOPIC_COUNT_FLAG,
};
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{
  LimitValue, LimitValueCertificateCount, LimitValueCertificateCountName, LimitValueConsumerRate, LimitValueConsumerRateName, LimitValueCpu, LimitValueCpuName,
};
use dsh_api::DshApiError;
use futures::future::try_join_all;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::warn;
use serde::Serialize;

pub(crate) struct TenantLimitSubject {}

const TENANT_LIMIT_SUBJECT_TARGET: &str = "tenant-limit";

lazy_static! {
  pub static ref TENANT_LIMIT_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TenantLimitSubject {});
}

lazy_static! {
  static ref TENANT_LIMIT_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TenantLimitListAll {}, "List managed tenant limits")
      .set_long_about("Lists all managed tenant limits.")
      .add_target_argument(managed_tenant_name_argument())
      .add_command_executor(FlagType::Ids, &TenantListIds {}, None)
  );
  static ref TENANT_LIMIT_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(
      SHOW_COMMAND,
      Some(SHOW_COMMAND_ALIAS),
      &TenantLimitShowAll {},
      "Show managed tenant configuration"
    )
    .set_long_about("Show the configuration of a DSH service.")
    .add_target_argument(managed_tenant_name_argument().required(true))
  );
  static ref TENANT_LIMIT_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &TenantLimitUpdate {}, "Update managed tenant limits")
      .set_long_about("Update a DSH service.")
      .add_target_argument(managed_tenant_name_argument().required(true))
      .add_extra_argument(certificate_count_flag().help_heading(HELP_HEADING))
      .add_extra_argument(consumer_rate_flag().help_heading(HELP_HEADING))
      .add_extra_argument(cpu_flag().help_heading(HELP_HEADING))
      // .add_extra_argument(kafka_acl_group_flag().help_heading(HELP_HEADING))
      .add_extra_argument(mem_flag().help_heading(HELP_HEADING))
      .add_extra_argument(partition_count_flag().help_heading(HELP_HEADING))
      .add_extra_argument(producer_rate_flag().help_heading(HELP_HEADING))
      .add_extra_argument(request_rate_flag().help_heading(HELP_HEADING))
      .add_extra_argument(secret_count_flag().help_heading(HELP_HEADING))
      .add_extra_argument(topic_count_flag().help_heading(HELP_HEADING))
  );
  static ref TENANT_LIMIT_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TENANT_LIMIT_LIST_CAPABILITY.as_ref(), TENANT_LIMIT_SHOW_CAPABILITY.as_ref(), TENANT_LIMIT_UPDATE_CAPABILITY.as_ref()];
}

#[async_trait]
impl Subject for TenantLimitSubject {
  fn subject(&self) -> &'static str {
    TENANT_LIMIT_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and limits of tenants on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("s")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(TENANT_LIMIT_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(TENANT_LIMIT_SHOW_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(TENANT_LIMIT_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TENANT_LIMIT_CAPABILITIES
  }
}

const HELP_HEADING: &str = "Manage tenant limit options";

struct TenantLimitListAll {}

#[async_trait]
impl CommandExecutor for TenantLimitListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all tenants with their limits");
    let start_instant = context.now();
    let tenant_ids: Vec<String> = client.get_tenant_ids().await?;
    let tenant_limits: Vec<Vec<LimitValue>> = try_join_all(tenant_ids.iter().map(|tenant_id| client.get_tenant_limits(tenant_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&TENANT_LIMIT_LABELS, None, context);
    for (tenant_id, limits) in tenant_ids.iter().zip(&tenant_limits) {
      formatter.push_target_id_value(tenant_id.clone(), limits);
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantListIds {}

#[async_trait]
impl CommandExecutor for TenantListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all tenant ids");
    let start_instant = context.now();
    let tenant_ids: Vec<String> = client.get_tenant_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("tenant id", context);
    formatter.push_target_ids(tenant_ids.as_slice());
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantLimitShowAll {}

#[async_trait]
impl CommandExecutor for TenantLimitShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all limits for tenant '{}'", tenant_id));
    let start_instant = context.now();
    let limits = client.get_tenant_limits(&tenant_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(tenant_id, &TENANT_LIMIT_LABELS, Some("tenant id"), context).print(&limits, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantLimitUpdate {}

#[async_trait]
impl CommandExecutor for TenantLimitUpdate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    let tenant_limits_from_arguments = TenantLimits::try_from(matches)?;
    if tenant_limits_from_arguments.is_empty() {
      return Err("at least one update argument must be provided".to_string());
    }
    context.print_explanation(format!("update tenant '{}'", tenant_id));
    match client.get_tenant_limits(&tenant_id).await {
      Ok(limits) => {
        let current_tenant_limits = TenantLimits::from(limits);
        let mut updated_tenant_limits = current_tenant_limits.clone();
        updated_tenant_limits.update(tenant_limits_from_arguments);
        if current_tenant_limits != updated_tenant_limits {
          if context.dry_run {
            context.print_warning("dry-run mode, limits not updated");
          } else {
            let limit_values: Vec<LimitValue> = updated_tenant_limits.try_into()?;
            client.patch_tenant_limit(&tenant_id, &limit_values).await?;
            context.print_outcome(format!("tenant '{}' updated", tenant_id));
          }
          Ok(())
        } else {
          context.print_outcome("provided arguments are equal to the current tenant limits, limits not updated");
          Ok(())
        }
      }
      Err(error) => match error {
        DshApiError::NotFound => {
          context.print_error(format!("tenant '{}' does not exist", tenant_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Clone, Default, PartialEq)]
struct TenantLimits {
  certificate_count: Option<u64>,
  consumer_rate: Option<u64>,
  cpu: Option<f64>,
  kafka_acl_group_count: Option<u64>,
  mem: Option<u64>,
  partition_count: Option<u64>,
  producer_rate: Option<u64>,
  request_rate: Option<u64>,
  secret_count: Option<u64>,
  topic_count: Option<u64>,
}

impl TenantLimits {
  fn is_empty(&self) -> bool {
    self.certificate_count.is_none()
      && self.consumer_rate.is_none()
      && self.cpu.is_none()
      && self.kafka_acl_group_count.is_none()
      && self.mem.is_none()
      && self.partition_count.is_none()
      && self.producer_rate.is_none()
      && self.request_rate.is_none()
      && self.secret_count.is_none()
      && self.topic_count.is_none()
  }

  fn update(&mut self, other: TenantLimits) {
    if let Some(count) = other.certificate_count {
      self.certificate_count = Some(count)
    }
    if let Some(rate) = other.consumer_rate {
      self.consumer_rate = Some(rate)
    }
    if let Some(cpu) = other.cpu {
      self.cpu = Some(cpu)
    }
    if let Some(count) = other.kafka_acl_group_count {
      self.kafka_acl_group_count = Some(count)
    }
    if let Some(mem) = other.mem {
      self.mem = Some(mem)
    }
    if let Some(count) = other.partition_count {
      self.partition_count = Some(count)
    }
    if let Some(rate) = other.producer_rate {
      self.producer_rate = Some(rate)
    }
    if let Some(rate) = other.request_rate {
      self.request_rate = Some(rate)
    }
    if let Some(count) = other.secret_count {
      self.secret_count = Some(count)
    }
    if let Some(count) = other.topic_count {
      self.topic_count = Some(count)
    }
  }
}

impl TryFrom<&ArgMatches> for TenantLimits {
  type Error = String;

  fn try_from(matches: &ArgMatches) -> Result<Self, Self::Error> {
    Ok(TenantLimits {
      certificate_count: matches.get_one::<u64>(CERTIFICATE_COUNT_FLAG).cloned(),
      consumer_rate: matches.get_one::<u64>(CONSUMER_RATE_FLAG).cloned(),
      cpu: match matches.get_one::<f64>(CPU_FLAG).cloned() {
        Some(cpus) => {
          if (0.01..=16.0).contains(&cpus) {
            if cpus % 0.01 == 0.0 {
              Some(cpus)
            } else {
              return Err("number of cpus must be a multiple of 0.01".to_string());
            }
          } else {
            return Err("number of cpus should be greater than or equal to 0.01 and lower than or equal to 16.0".to_string());
          }
        }
        None => None,
      },
      kafka_acl_group_count: matches.get_one::<u64>(KAFKA_ACL_GROUP_COUNT_FLAG).cloned(),
      mem: matches.get_one::<u64>(MEM_FLAG).cloned(),
      partition_count: matches.get_one::<u64>(PARTITION_COUNT_FLAG).cloned(),
      producer_rate: matches.get_one::<u64>(PRODUCER_RATE_FLAG).cloned(),
      request_rate: matches.get_one::<u64>(REQUEST_RATE_FLAG).cloned(),
      secret_count: matches.get_one::<u64>(SECRET_COUNT_FLAG).cloned(),
      topic_count: matches.get_one::<u64>(TOPIC_COUNT_FLAG).cloned(),
    })
  }
}

impl From<Vec<LimitValue>> for TenantLimits {
  fn from(limits: Vec<LimitValue>) -> Self {
    let mut tenant_limits = TenantLimits::default();
    for limit in limits {
      match limit {
        LimitValue::CertificateCount(certificate_count) => tenant_limits.certificate_count = Some(certificate_count.value as u64),
        LimitValue::ConsumerRate(consumer_rate) => tenant_limits.consumer_rate = Some(consumer_rate.value as u64),
        LimitValue::Cpu(cpu) => tenant_limits.cpu = Some(cpu.value),
        LimitValue::KafkaAclGroupCount(kafka_acl_group_count) => tenant_limits.kafka_acl_group_count = Some(kafka_acl_group_count.value as u64),
        LimitValue::Mem(mem) => tenant_limits.mem = Some(mem.value as u64),
        LimitValue::PartitionCount(partition_count) => tenant_limits.partition_count = Some(partition_count.value as u64),
        LimitValue::ProducerRate(producer_rate) => tenant_limits.producer_rate = Some(producer_rate.value as u64),
        LimitValue::RequestRate(request_rate) => tenant_limits.request_rate = Some(request_rate.value as u64),
        LimitValue::SecretCount(secret_count) => tenant_limits.secret_count = Some(secret_count.value as u64),
        LimitValue::TopicCount(topic_count) => tenant_limits.topic_count = Some(topic_count.value as u64),
      }
    }
    tenant_limits
  }
}

impl TryFrom<TenantLimits> for Vec<LimitValue> {
  type Error = String;

  fn try_from(value: TenantLimits) -> Result<Self, Self::Error> {
    let mut limit_values = vec![];
    if let Some(count) = value.certificate_count {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    if let Some(rate) = value.consumer_rate {
      limit_values.push(LimitValue::ConsumerRate(LimitValueConsumerRate {
        name: LimitValueConsumerRateName::ConsumerRate,
        value: rate as i64,
      }))
    }
    if let Some(count) = value.cpu {
      limit_values.push(LimitValue::Cpu(LimitValueCpu { name: LimitValueCpuName::Cpu, value: count }))
    }
    if value.kafka_acl_group_count.is_some() {
      warn!("kafka acl group limit not properly implemented in dsh resource api, limit will be skipped");
    }
    if let Some(count) = value.mem {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    if let Some(count) = value.partition_count {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    if let Some(count) = value.producer_rate {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    if let Some(count) = value.request_rate {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    if let Some(count) = value.secret_count {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    if let Some(count) = value.topic_count {
      limit_values.push(LimitValue::CertificateCount(LimitValueCertificateCount {
        name: LimitValueCertificateCountName::Cpu,
        value: count as i64,
      }))
    }
    Ok(limit_values)
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum TenantLimitLabel {
  CertificateCount,
  ConsumerRate,
  Cpu,
  KafkaAclGroupCount,
  Mem,
  PartitionCount,
  ProducerRate,
  RequestRate,
  SecretCount,
  Tenant,
  TopicCount,
}

impl Label for TenantLimitLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CertificateCount => "certificate count",
      Self::ConsumerRate => "consumer rate",
      Self::Cpu => "cpu",
      Self::KafkaAclGroupCount => "kafka acl group count",
      Self::Mem => "mem",
      Self::PartitionCount => "partition count",
      Self::ProducerRate => "producer rate",
      Self::RequestRate => "request rate",
      Self::SecretCount => "secret count",
      Self::Tenant => "managed tenant",
      Self::TopicCount => "topic count",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::CertificateCount => "certificates",
      Self::ConsumerRate => "consumer",
      Self::Cpu => "cpu",
      Self::KafkaAclGroupCount => "acl groups",
      Self::Mem => "mem",
      Self::PartitionCount => "partitions",
      Self::ProducerRate => "producer",
      Self::RequestRate => "request",
      Self::SecretCount => "secrets",
      Self::Tenant => "tenant",
      Self::TopicCount => "topics",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Tenant)
  }
}

impl SubjectFormatter<TenantLimitLabel> for TenantLimits {
  fn value(&self, label: &TenantLimitLabel, target_id: &str) -> String {
    match label {
      TenantLimitLabel::CertificateCount => self.certificate_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLimitLabel::ConsumerRate => self.consumer_rate.map(|rate| rate.to_string()).unwrap_or_default(),
      TenantLimitLabel::Cpu => self.cpu.map(|cpu| cpu.to_string()).unwrap_or_default(),
      TenantLimitLabel::KafkaAclGroupCount => self.kafka_acl_group_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLimitLabel::Mem => self.mem.map(|mem| mem.to_string()).unwrap_or_default(),
      TenantLimitLabel::PartitionCount => self.partition_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLimitLabel::ProducerRate => self.producer_rate.map(|rate| rate.to_string()).unwrap_or_default(),
      TenantLimitLabel::RequestRate => self.request_rate.map(|rate| rate.to_string()).unwrap_or_default(),
      TenantLimitLabel::SecretCount => self.secret_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLimitLabel::Tenant => target_id.to_string(),
      TenantLimitLabel::TopicCount => self.topic_count.map(|count| count.to_string()).unwrap_or_default(),
    }
  }
}

impl SubjectFormatter<TenantLimitLabel> for Vec<LimitValue> {
  fn value(&self, label: &TenantLimitLabel, tenant_id: &str) -> String {
    match label {
      TenantLimitLabel::CertificateCount => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::CertificateCount(count) => Some(count.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::ConsumerRate => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::ConsumerRate(rate) => Some(rate.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::Cpu => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::Cpu(cpu) => Some(cpu.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::KafkaAclGroupCount => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::KafkaAclGroupCount(count) => Some(count.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::Mem => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::Mem(mem) => Some(mem.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::PartitionCount => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::PartitionCount(count) => Some(count.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::ProducerRate => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::ProducerRate(rate) => Some(rate.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::RequestRate => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::RequestRate(rate) => Some(rate.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::SecretCount => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::SecretCount(count) => Some(count.value.to_string()),
          _ => None,
        })
        .join(", "),
      TenantLimitLabel::Tenant => tenant_id.to_string(),
      TenantLimitLabel::TopicCount => self
        .iter()
        .filter_map(|limit| match limit {
          LimitValue::TopicCount(count) => Some(count.value.to_string()),
          _ => None,
        })
        .join(", "),
    }
  }
}

pub static TENANT_LIMIT_LABELS: [TenantLimitLabel; 11] = [
  TenantLimitLabel::Tenant,
  TenantLimitLabel::CertificateCount,
  TenantLimitLabel::ConsumerRate,
  TenantLimitLabel::Cpu,
  TenantLimitLabel::KafkaAclGroupCount,
  TenantLimitLabel::Mem,
  TenantLimitLabel::PartitionCount,
  TenantLimitLabel::ProducerRate,
  TenantLimitLabel::RequestRate,
  TenantLimitLabel::SecretCount,
  TenantLimitLabel::TopicCount,
];
