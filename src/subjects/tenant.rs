use crate::arguments::managed_tenant_argument;
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
  LimitValue, LimitValueCertificateCount, LimitValueCertificateCountName, LimitValueConsumerRate, LimitValueConsumerRateName, LimitValueCpu, LimitValueCpuName, ManagedTenant,
  ManagedTenantServicesName,
};
use dsh_api::DshApiError;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;
use log::warn;
use serde::Serialize;

pub(crate) struct TenantSubject {}

const TENANT_SUBJECT_TARGET: &str = "tenant";

lazy_static! {
  pub static ref TENANT_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TenantSubject {});
}

const HELP_HEADING: &str = "Tenant options";

lazy_static! {
  static ref TENANT_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TenantListAll {}, "List managed tenants")
      .set_long_about("Lists all managed tenants.")
      .add_target_argument(managed_tenant_argument())
      .add_command_executor(FlagType::Ids, &TenantListIds {}, None)
  );
  static ref TENANT_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(
      SHOW_COMMAND,
      Some(SHOW_COMMAND_ALIAS),
      &TenantShowAll {},
      "Show managed tenant configuration"
    )
    .set_long_about("Show the configuration of a managed tenant.")
    .add_target_argument(managed_tenant_argument().required(true))
  );
  static ref TENANT_UPDATE_LIMIT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &TenantUpdateLimit {}, "Update managed tenant limits")
      .set_long_about("Update the limits of a managed tenant.")
      .add_target_argument(managed_tenant_argument().required(true))
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
  static ref TENANT_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TENANT_LIST_CAPABILITY.as_ref(), TENANT_SHOW_CAPABILITY.as_ref(), TENANT_UPDATE_LIMIT_CAPABILITY.as_ref()];
}

#[async_trait]
impl Subject for TenantSubject {
  fn subject(&self) -> &'static str {
    TENANT_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show and manage tenants on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(TENANT_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(TENANT_SHOW_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(TENANT_UPDATE_LIMIT_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TENANT_CAPABILITIES
  }
}

struct TenantListAll {}

#[async_trait]
impl CommandExecutor for TenantListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all tenants with their limits");
    let start_instant = context.now();
    let tenant_ids: Vec<String> = client.get_tenant_ids().await?;
    if tenant_ids.is_empty() {
      context.print_error("you are not authorized to manage tenants");
    } else {
      let (managed_tenants, limits) = try_join!(
        try_join_all(tenant_ids.iter().map(|tenant_id| client.get_tenant_configuration(tenant_id))),
        try_join_all(tenant_ids.iter().map(|tenant_id| client.get_tenant_limits(tenant_id)))
      )?;
      context.print_execution_time(start_instant);
      let managed_tenants_limits: Vec<(ManagedTenant, TenantLimits)> = managed_tenants.into_iter().zip(limits.iter().map(TenantLimits::from)).collect::<Vec<_>>();
      let mut formatter = ListFormatter::new(&TENANT_LABELS, None, context);
      for (tenant_id, managed_tenant_limit) in tenant_ids.iter().zip(&managed_tenants_limits) {
        formatter.push_target_id_value(tenant_id.clone(), managed_tenant_limit);
      }
      formatter.print(None)?;
    }
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
    if tenant_ids.is_empty() {
      context.print_error("you are not authorized to manage tenants");
    } else {
      let mut formatter = IdsFormatter::new("tenant id", context);
      formatter.push_target_ids(tenant_ids.as_slice());
      formatter.print(Some(OutputFormat::Plain))?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantShowAll {}

#[async_trait]
impl CommandExecutor for TenantShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all limits for tenant '{}'", tenant_id));
    let start_instant = context.now();
    match try_join!(client.get_tenant_configuration(&tenant_id), client.get_tenant_limits(&tenant_id)) {
      Ok((managed_tenant, limit_values)) => {
        context.print_execution_time(start_instant);
        UnitFormatter::new(tenant_id, &TENANT_LABELS, Some("tenant id"), context).print(&(managed_tenant, TenantLimits::from(&limit_values)), None)
      }
      Err(error) => match error {
        DshApiError::NotFound => {
          context.print_error(format!("tenant '{}' does not exist or you are not authorized to manage it", tenant_id));
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

struct TenantUpdateLimit {}

#[async_trait]
impl CommandExecutor for TenantUpdateLimit {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    let tenant_limits_from_arguments = TenantLimits::try_from(matches)?;
    if tenant_limits_from_arguments.is_empty() {
      return Err("at least one limit argument must be provided".to_string());
    }
    context.print_explanation(format!("update limits of tenant '{}'", tenant_id));
    match client.get_tenant_limits(&tenant_id).await {
      Ok(limits) => {
        let current_tenant_limits = TenantLimits::from(&limits);
        let mut updated_tenant_limits = current_tenant_limits.clone();
        updated_tenant_limits.update(tenant_limits_from_arguments);
        if current_tenant_limits != updated_tenant_limits {
          UnitFormatter::new(tenant_id.clone(), &TENANT_LIMIT_LABELS, Some("tenant id"), context).print(&updated_tenant_limits, None)?;
          if context.confirmed(format!("update limits for tenant '{}' to the above values?", tenant_id))? {
            if context.dry_run {
              context.print_warning("dry-run mode, limits not updated");
            } else {
              let limit_values: Vec<LimitValue> = updated_tenant_limits.try_into()?;
              client.patch_tenant_limit(&tenant_id, &limit_values).await?;
              context.print_outcome(format!("limits for tenant '{}' updated", tenant_id));
            }
            Ok(())
          } else {
            context.print_outcome(format!("cancelled, limits for tenant '{}' not updated", tenant_id));
            Ok(())
          }
        } else {
          context.print_outcome("provided limits are equal to the current tenant limits, limits not updated");
          Ok(())
        }
      }
      Err(error) => match error {
        DshApiError::NotFound => {
          context.print_error(format!("tenant '{}' does not exist or you are not authorized to manage it", tenant_id));
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

#[derive(Clone, Default, PartialEq, Serialize)]
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

impl From<&Vec<LimitValue>> for TenantLimits {
  fn from(limits: &Vec<LimitValue>) -> Self {
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
pub(crate) enum TenantLabel {
  CertificateCount,
  ConsumerRate,
  Cpu,
  KafkaAclGroupCount,
  Manager,
  Mem,
  Monitoring,
  Name,
  PartitionCount,
  ProducerRate,
  RequestRate,
  SecretCount,
  Tenant,
  TopicCount,
  Tracing,
  Vpn,
}

impl Label for TenantLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CertificateCount => "certificate count",
      Self::ConsumerRate => "consumer rate",
      Self::Cpu => "cpu",
      Self::KafkaAclGroupCount => "kafka acl group count",
      Self::Manager => "managing tenant",
      Self::Mem => "mem",
      Self::Monitoring => "monitoring enabled",
      Self::Name => "name",
      Self::PartitionCount => "partition count",
      Self::ProducerRate => "producer rate",
      Self::RequestRate => "request rate",
      Self::SecretCount => "secret count",
      Self::Tenant => "managed tenant",
      Self::TopicCount => "topic count",
      Self::Tracing => "tracing enabled",
      Self::Vpn => "vpn enabled",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::CertificateCount => "certificates",
      Self::ConsumerRate => "consumer",
      Self::Cpu => "cpu",
      Self::KafkaAclGroupCount => "acl groups",
      Self::Manager => "manager",
      Self::Mem => "mem",
      Self::Monitoring => "monitoring",
      Self::Name => "name",
      Self::PartitionCount => "partitions",
      Self::ProducerRate => "producer",
      Self::RequestRate => "request",
      Self::SecretCount => "secrets",
      Self::Tenant => "tenant",
      Self::TopicCount => "topics",
      Self::Tracing => "tracing",
      Self::Vpn => "vpn",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Tenant)
  }
}

impl SubjectFormatter<TenantLabel> for TenantLimits {
  fn value(&self, label: &TenantLabel, target_id: &str) -> String {
    match label {
      TenantLabel::CertificateCount => self.certificate_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLabel::ConsumerRate => self.consumer_rate.map(|rate| rate.to_string()).unwrap_or_default(),
      TenantLabel::Cpu => self.cpu.map(|cpu| cpu.to_string()).unwrap_or_default(),
      TenantLabel::KafkaAclGroupCount => self.kafka_acl_group_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLabel::Mem => self.mem.map(|mem| mem.to_string()).unwrap_or_default(),
      TenantLabel::PartitionCount => self.partition_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLabel::ProducerRate => self.producer_rate.map(|rate| rate.to_string()).unwrap_or_default(),
      TenantLabel::RequestRate => self.request_rate.map(|rate| rate.to_string()).unwrap_or_default(),
      TenantLabel::SecretCount => self.secret_count.map(|count| count.to_string()).unwrap_or_default(),
      TenantLabel::Tenant => target_id.to_string(),
      TenantLabel::TopicCount => self.topic_count.map(|count| count.to_string()).unwrap_or_default(),
      _ => unreachable!(),
    }
  }
}

pub static TENANT_LIMIT_LABELS: [TenantLabel; 11] = [
  TenantLabel::Tenant,
  TenantLabel::CertificateCount,
  TenantLabel::ConsumerRate,
  TenantLabel::Cpu,
  TenantLabel::KafkaAclGroupCount,
  TenantLabel::Mem,
  TenantLabel::PartitionCount,
  TenantLabel::ProducerRate,
  TenantLabel::RequestRate,
  TenantLabel::SecretCount,
  TenantLabel::TopicCount,
];

pub static TENANT_LABELS: [TenantLabel; 16] = [
  TenantLabel::Tenant,
  TenantLabel::Manager,
  TenantLabel::Monitoring,
  TenantLabel::Name,
  TenantLabel::Tracing,
  TenantLabel::Vpn,
  TenantLabel::CertificateCount,
  TenantLabel::ConsumerRate,
  TenantLabel::Cpu,
  TenantLabel::KafkaAclGroupCount,
  TenantLabel::Mem,
  TenantLabel::PartitionCount,
  TenantLabel::ProducerRate,
  TenantLabel::RequestRate,
  TenantLabel::SecretCount,
  TenantLabel::TopicCount,
];

impl SubjectFormatter<TenantLabel> for ManagedTenant {
  fn value(&self, label: &TenantLabel, _target_id: &str) -> String {
    match label {
      TenantLabel::Manager => self.manager.to_string(),
      TenantLabel::Monitoring => service(self, ManagedTenantServicesName::Monitoring),
      TenantLabel::Name => self.name.to_string(),
      TenantLabel::Tracing => service(self, ManagedTenantServicesName::Tracing),
      TenantLabel::Vpn => service(self, ManagedTenantServicesName::Vpn),
      _ => unreachable!(),
    }
  }
}

impl SubjectFormatter<TenantLabel> for (ManagedTenant, TenantLimits) {
  fn value(&self, label: &TenantLabel, target_id: &str) -> String {
    match label {
      TenantLabel::Manager | TenantLabel::Monitoring | TenantLabel::Name | TenantLabel::Tracing | TenantLabel::Vpn => self.0.value(label, target_id),
      _ => self.1.value(label, target_id),
    }
  }
}

fn service(managed_tenant: &ManagedTenant, name: ManagedTenantServicesName) -> String {
  managed_tenant
    .services
    .iter()
    .find_map(|service| if service.name == name { Some(service.enabled.to_string()) } else { None })
    .unwrap_or_default()
}

pub static _MANAGED_TENANT_LABELS: [TenantLabel; 5] = [TenantLabel::Name, TenantLabel::Manager, TenantLabel::Monitoring, TenantLabel::Tracing, TenantLabel::Vpn];
