use crate::arguments::managed_tenant_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, GRANT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, REVOKE_COMMAND,
  SHOW_COMMAND, SHOW_COMMAND_ALIAS, UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::error::DshCliError;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::limits_options::{
  certificate_count_flag, consumer_rate_flag, cpu_flag, kafka_acl_group_count_flag, mem_flag, partition_count_flag, producer_rate_flag, request_rate_flag, secret_count_flag,
  stream_read_flag, stream_rw_flag, stream_write_flag, topic_count_flag, tracing_flag, vpn_flag, CERTIFICATE_COUNT_OPTION, CONSUMER_RATE_OPTION, CPU_OPTION,
  KAFKA_ACL_GROUP_COUNT_OPTION, MEM_OPTION, PARTITION_COUNT_OPTION, PRODUCER_RATE_OPTION, REQUEST_RATE_OPTION, SECRET_COUNT_OPTION, STREAM_READ_OPTION, STREAM_RW_OPTION,
  STREAM_WRITE_OPTION, TOPIC_COUNT_OPTION, TRACING_OPTION, VPN_OPTION,
};
use crate::subject::{Requirements, Subject};
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::stream::Stream;
use dsh_api::tenant::TenantLimits;
use dsh_api::types::{LimitValue, ManagedStreamId, ManagedTenant, ManagedTenantServices, ManagedTenantServicesName};
use dsh_api::AccessRights;
use futures::future::{try_join, try_join_all};
use futures::{join, try_join};
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::num::NonZeroU64;

struct TenantSubject {}

const TENANT_SUBJECT_TARGET: &str = "tenant";

lazy_static! {
  pub(crate) static ref TENANT_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TenantSubject {});
}

lazy_static! {
  static ref TENANT_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &TenantCreate {}, "Create managed tenant")
      .set_long_about("Create a configured managed tenant.")
      .add_target_argument(managed_tenant_argument().required(true))
      .add_extra_argument(tracing_flag())
      .add_extra_argument(vpn_flag())
  );
  static ref TENANT_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &TenantDelete {}, "Delete managed tenant")
      .set_long_about("Delete a managed tenant and its configuration.")
      .add_target_argument(managed_tenant_argument().required(true))
  );
  static ref TENANT_GRANT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(GRANT_COMMAND, None, &TenantGrant {}, "Grant access rights")
      .set_long_about(
        "Grant a managed tenant read and/or write access rights to restricted resources \
         (managed streams)."
      )
      .add_target_argument(managed_tenant_argument().required(true))
      .add_extra_argument(stream_read_flag("Grant"))
      .add_extra_argument(stream_write_flag("Grant"))
      .add_extra_argument(stream_rw_flag("Grant"))
  );
  static ref TENANT_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TenantListAll {}, "List managed tenants")
      .set_long_about("Lists all managed tenants.")
      .add_target_argument(managed_tenant_argument())
      .add_command_executor(FlagType::Ids, &TenantListIds {}, None)
      .add_command_executor(FlagType::Stream, &TenantListStreams {}, None)
  );
  static ref TENANT_REVOKE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(REVOKE_COMMAND, None, &TenantRevoke {}, "Revoke access rights")
      .set_long_about(
        "Revoke read and/or write access rights to restricted resources \
         (managed streams) from a managed tenant."
      )
      .add_target_argument(managed_tenant_argument().required(true))
      .add_extra_argument(stream_read_flag("Revoke"))
      .add_extra_argument(stream_write_flag("Revoke"))
      .add_extra_argument(stream_rw_flag("Revoke"))
  );
  static ref TENANT_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &TenantShow {}, "Show managed tenant configuration")
      .set_long_about("Show the configuration of a managed tenant.")
      .add_target_argument(managed_tenant_argument().required(true))
      .add_command_executor(FlagType::Stream, &TenantShowStreams {}, None)
  );
  static ref TENANT_UPDATE_LIMIT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &TenantUpdateLimit {}, "Update managed tenant limits")
      .set_long_about("Update the limits of a managed tenant.")
      .add_target_argument(managed_tenant_argument().required(true))
      .add_extra_argument(certificate_count_flag())
      .add_extra_argument(consumer_rate_flag())
      .add_extra_argument(cpu_flag())
      .add_extra_argument(kafka_acl_group_count_flag())
      .add_extra_argument(mem_flag())
      .add_extra_argument(partition_count_flag())
      .add_extra_argument(producer_rate_flag())
      .add_extra_argument(request_rate_flag())
      .add_extra_argument(secret_count_flag())
      .add_extra_argument(topic_count_flag())
      .add_extra_argument(tracing_flag())
      .add_extra_argument(vpn_flag())
  );
  static ref TENANT_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    TENANT_CREATE_CAPABILITY.as_ref(),
    TENANT_DELETE_CAPABILITY.as_ref(),
    TENANT_GRANT_CAPABILITY.as_ref(),
    TENANT_LIST_CAPABILITY.as_ref(),
    TENANT_REVOKE_CAPABILITY.as_ref(),
    TENANT_SHOW_CAPABILITY.as_ref(),
    TENANT_UPDATE_LIMIT_CAPABILITY.as_ref()
  ];
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
      CREATE_COMMAND => Some(TENANT_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(TENANT_DELETE_CAPABILITY.as_ref()),
      GRANT_COMMAND => Some(TENANT_GRANT_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(TENANT_LIST_CAPABILITY.as_ref()),
      REVOKE_COMMAND => Some(TENANT_REVOKE_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(TENANT_SHOW_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(TENANT_UPDATE_LIMIT_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TENANT_CAPABILITIES
  }
}

struct TenantCreate {}

#[async_trait]
impl CommandExecutor for TenantCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    if client.get_tenant_configuration(&tenant_id).await.is_ok() {
      return err!("managed tenant '{}' already exists", tenant_id);
    }
    let enable_tracing = matches.get_one::<bool>(TRACING_OPTION);
    let enable_vpn = matches.get_one::<bool>(VPN_OPTION);
    context.print_explanation(format!("create new managed tenant '{}'", tenant_id));
    let mut services = vec![
      // Monitoring service is mandatory.
      ManagedTenantServices { enabled: true, name: ManagedTenantServicesName::Monitoring },
    ];
    if let Some(tracing_enabled) = enable_tracing {
      services.push(ManagedTenantServices { enabled: *tracing_enabled, name: ManagedTenantServicesName::Tracing });
    }
    if let Some(vpn_enabled) = enable_vpn {
      services.push(ManagedTenantServices { enabled: *vpn_enabled, name: ManagedTenantServicesName::Vpn });
    }
    let managed_tenant = ManagedTenant { manager: client.tenant_name().to_string(), name: tenant_id.clone(), services };
    if context.dry_run() {
      context.print_warning("dry-run mode, tenant not created");
    } else {
      client.put_tenant_configuration(&tenant_id, &managed_tenant).await?;
      context.print_outcome(format!("tenant '{}' created", tenant_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantDelete {}

#[async_trait]
impl CommandExecutor for TenantDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    if client.get_tenant_configuration(&tenant_id).await.is_err() {
      return err!("tenant '{}' does not exist or you are not authorized to manage it", tenant_id);
    }
    if context.confirmed(format!("delete tenant '{}'?", tenant_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, tenant not deleted");
      } else {
        client.delete_tenant_configuration(&tenant_id).await?;
        context.print_outcome(format!("tenant '{}' deleted", tenant_id));
      }
    } else {
      context.print_outcome(format!("cancelled, tenant '{}' not deleted", tenant_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantGrant {}

#[async_trait]
impl CommandExecutor for TenantGrant {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let managed_tenant_id = target.unwrap_or_else(|| unreachable!());
    let (managed_stream_id, access_rights) = get_managed_stream_id(matches, client.tenant_name())?;
    context.print_explanation(format!(
      "grant {} access to managed stream '{}' to managed tenant '{}'",
      access_rights, managed_stream_id, managed_tenant_id
    ));
    let kind = client
      .managed_stream_grant_access_rights(&managed_stream_id, &managed_tenant_id, &access_rights)
      .await?;
    context.print_outcome(format!("access granted to {} managed stream", kind));
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static TENANT_LABELS: [TenantLabel; 15] = [
  TenantLabel::Tenant,
  TenantLabel::Manager,
  TenantLabel::Monitoring,
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

struct TenantListAll {}

#[async_trait]
impl CommandExecutor for TenantListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all tenants with their limits");
    let start_instant = context.now();
    let tenant_ids: Vec<String> = client.get_tenant_ids().await?;
    if tenant_ids.is_empty() {
      context.print_outcome("no managed tenants or you are not authorized to manage tenants");
    } else {
      let (managed_tenants, limits) = try_join!(
        try_join_all(tenant_ids.iter().map(|tenant_id| client.get_tenant_configuration(tenant_id))),
        try_join_all(tenant_ids.iter().map(|tenant_id| client.managed_tenant_limits(tenant_id)))
      )?;
      context.print_execution_time(start_instant);
      let managed_tenants_limits: Vec<(ManagedTenant, TenantLimits)> = managed_tenants.into_iter().zip(limits).collect_vec();
      let mut formatter = ListFormatter::new(&TENANT_LABELS, context);
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all tenant ids");
    let start_instant = context.now();
    let tenant_ids: Vec<String> = client.get_tenant_ids().await?;
    context.print_execution_time(start_instant);
    if tenant_ids.is_empty() {
      context.print_outcome("no managed tenants or you are not authorized to manage tenants");
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

static LIST_STREAM_ACCESS_LABELS: [StreamAccessLabel; 7] = [
  StreamAccessLabel::Tenant,
  StreamAccessLabel::StreamId,
  StreamAccessLabel::StreamKind,
  StreamAccessLabel::ReadAccess,
  StreamAccessLabel::WriteAccess,
  StreamAccessLabel::Partitions,
  StreamAccessLabel::ReplicationFactor,
];

struct TenantListStreams {}

#[async_trait]
impl CommandExecutor for TenantListStreams {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all tenants with the managed streams that they are granted access");
    let start_instant = context.now();

    let tenant_ids = client.get_tenant_ids().await?;
    if tenant_ids.is_empty() {
      context.print_outcome("no managed tenants or you are not authorized to manage tenants");
    } else {
      let tenants_granted_streams: Vec<Vec<(ManagedStreamId, Stream, AccessRights)>> =
        try_join_all(tenant_ids.iter().map(|tenant_id| client.managed_tenant_granted_managed_streams(tenant_id))).await?;
      context.print_execution_time(start_instant);
      let mut formatter = ListFormatter::new(&LIST_STREAM_ACCESS_LABELS, context);
      for (tenant_id, granted_streams) in tenant_ids.iter().zip(&tenants_granted_streams) {
        for granted_stream in granted_streams {
          formatter.push_target_id_value(tenant_id.clone(), granted_stream);
        }
      }
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantRevoke {}

#[async_trait]
impl CommandExecutor for TenantRevoke {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let managed_tenant_id = target.unwrap_or_else(|| unreachable!());
    let (managed_stream_id, rights) = get_managed_stream_id(matches, client.tenant_name())?;
    context.print_explanation(format!(
      "revoke {} access to managed stream '{}' from managed tenant '{}'",
      rights, managed_stream_id, managed_tenant_id
    ));
    let kind = client.managed_stream_revoke_access_rights(&managed_stream_id, &managed_tenant_id, &rights).await?;
    context.print_outcome(format!("access revoked from {} managed stream", kind));
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantShow {}

#[async_trait]
impl CommandExecutor for TenantShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show tenant '{}' with its limits", tenant_id));
    let start_instant = context.now();
    let (managed_tenant_with_limits, allocation_status) = join!(
      try_join(client.get_tenant_configuration(&tenant_id), client.managed_tenant_limits(&tenant_id)),
      client.get_tenant_status(&tenant_id)
    );
    match managed_tenant_with_limits {
      Ok((managed_tenant, tenant_limits)) => {
        context.print_execution_time(start_instant);
        context.print_allocation_status(&allocation_status, TENANT_SUBJECT_TARGET);
        UnitFormatter::new(tenant_id, &TENANT_LABELS, context).print(&(managed_tenant, tenant_limits), None)
      }
      Err(error) => DshCliError::accept_not_found(error, || {
        context.print_error(format!("tenant '{}' does not exist or you are not authorized to manage it", tenant_id))
      }),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static STREAM_ACCESS_LABELS: [StreamAccessLabel; 6] = [
  StreamAccessLabel::StreamId,
  StreamAccessLabel::StreamKind,
  StreamAccessLabel::ReadAccess,
  StreamAccessLabel::WriteAccess,
  StreamAccessLabel::Partitions,
  StreamAccessLabel::ReplicationFactor,
];

struct TenantShowStreams {}

#[async_trait]
impl CommandExecutor for TenantShowStreams {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all streams that tenant '{}' has access to", tenant_id));
    let start_instant = context.now();
    let grants: Vec<(ManagedStreamId, Stream, AccessRights)> = client.managed_tenant_granted_managed_streams(&tenant_id).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&STREAM_ACCESS_LABELS, context);
    formatter.push_values(&grants);
    formatter.print(None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct TenantUpdateLimit {}

#[async_trait]
impl CommandExecutor for TenantUpdateLimit {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    let enable_tracing_argument = matches.get_one::<bool>(TRACING_OPTION);
    let enable_vpn_argument = matches.get_one::<bool>(VPN_OPTION);
    let tenant_limits_from_arguments = tenant_limits_try_from_matches(matches)?;

    match (
      enable_tracing_argument.is_some() || enable_vpn_argument.is_some(),
      !tenant_limits_from_arguments.is_empty(),
    ) {
      (false, false) => err!("at least one limit or capability argument must be provided"),

      (false, true) => {
        context.print_explanation(format!("update limits of managed tenant '{}'", tenant_id));
        match client.managed_tenant_limits(&tenant_id).await {
          Ok(current_tenant_limits) => {
            let mut updated_tenant_limits = current_tenant_limits.clone();
            updated_tenant_limits.update(&tenant_limits_from_arguments);
            if current_tenant_limits != updated_tenant_limits {
              if context.dry_run() {
                context.print_warning("dry-run mode, limits not updated");
              } else {
                let limit_values: Vec<LimitValue> = (&updated_tenant_limits).into();
                client.patch_tenant_limit(&tenant_id, &limit_values).await?;
                context.print_outcome(format!("limits for managed tenant '{}' updated", tenant_id));
              }
            } else {
              context.print_outcome("provided limits are equal to the current managed tenant limits, limits not updated");
            }
            Ok(())
          }
          Err(error) => DshCliError::accept_not_found(error, || {
            context.print_error(format!("managed tenant '{}' does not exist or you are not authorized to manage it", tenant_id))
          }),
        }
      }

      (true, false) => {
        context.print_explanation(format!("update capabilities of managed tenant '{}'", tenant_id));
        match client.get_tenant_configuration(&tenant_id).await {
          Ok(mut managed_tenant) => {
            let mut update = false;
            if let Some(eta) = enable_tracing_argument {
              match managed_tenant.services.iter_mut().find(|s| s.name == ManagedTenantServicesName::Tracing) {
                Some(tracing_service) => {
                  if *eta != tracing_service.enabled {
                    tracing_service.enabled = *eta;
                    update = true;
                  }
                }
                None => {
                  managed_tenant
                    .services
                    .push(ManagedTenantServices { enabled: *eta, name: ManagedTenantServicesName::Tracing });
                  update = true;
                }
              }
            }
            if let Some(eva) = enable_vpn_argument {
              match managed_tenant.services.iter_mut().find(|s| s.name == ManagedTenantServicesName::Vpn) {
                Some(vpn_service) => {
                  if *eva != vpn_service.enabled {
                    vpn_service.enabled = *eva;
                    update = true;
                  }
                }
                None => {
                  managed_tenant
                    .services
                    .push(ManagedTenantServices { enabled: *eva, name: ManagedTenantServicesName::Vpn });
                  update = true;
                }
              }
            }
            if update {
              if context.dry_run() {
                context.print_warning("dry-run mode, capabilities not updated");
              } else {
                client.put_tenant_configuration(&tenant_id, &managed_tenant).await?;
                context.print_outcome(format!("capabilities for managed tenant '{}' updated", tenant_id));
              }
            } else {
              context.print_warning("provided arguments match current values, managed tenant not updated")
            }
            Ok(())
          }
          Err(error) => DshCliError::accept_not_found(error, || {
            context.print_error(format!("managed tenant '{}' does not exist or you are not authorized to manage it", tenant_id))
          }),
        }
      }

      (true, true) => err!("provide either limit arguments or capability arguments, but not both"),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn get_managed_stream_id(matches: &ArgMatches, managing_tenant: &str) -> DshCliResult<(ManagedStreamId, AccessRights)> {
  Ok(match matches.get_one::<String>(STREAM_READ_OPTION) {
    Some(stream) => (managed_stream_id(stream, managing_tenant)?, AccessRights::Read),
    None => match matches.get_one::<String>(STREAM_RW_OPTION) {
      Some(stream) => (managed_stream_id(stream, managing_tenant)?, AccessRights::ReadWrite),
      None => match matches.get_one::<String>(STREAM_WRITE_OPTION) {
        Some(stream) => (managed_stream_id(stream, managing_tenant)?, AccessRights::Write),
        None => unreachable!(),
      },
    },
  })
}

fn managed_stream_id(stream_argument: &str, managing_tenant: &str) -> DshCliResult<ManagedStreamId> {
  if stream_argument.starts_with(&format!("{}---", managing_tenant)) {
    ManagedStreamId::try_from(stream_argument).map_err(DshCliError::from)
  } else {
    err!("managed stream id must start with '{}---'", managing_tenant)
  }
}

fn tenant_limits_try_from_matches(matches: &ArgMatches) -> DshCliResult<TenantLimits> {
  Ok(TenantLimits {
    certificate_count: matches.get_one::<NonZeroU64>(CERTIFICATE_COUNT_OPTION).cloned(),
    consumer_rate: matches.get_one::<i64>(CONSUMER_RATE_OPTION).cloned(),
    cpu: match matches.get_one::<f64>(CPU_OPTION).cloned() {
      Some(cpus) => {
        if (0.01..=16.0).contains(&cpus) {
          Some(cpus)
        } else {
          return err!("number of cpus should be greater than or equal to 0.01 and lower than or equal to 16.0");
        }
      }
      None => None,
    },
    kafka_acl_group_count: matches.get_one::<i64>(KAFKA_ACL_GROUP_COUNT_OPTION).cloned(),
    mem: matches.get_one::<NonZeroU64>(MEM_OPTION).cloned(),
    partition_count: matches.get_one::<NonZeroU64>(PARTITION_COUNT_OPTION).cloned(),
    producer_rate: matches.get_one::<i64>(PRODUCER_RATE_OPTION).cloned(),
    request_rate: matches.get_one::<NonZeroU64>(REQUEST_RATE_OPTION).cloned(),
    secret_count: matches.get_one::<NonZeroU64>(SECRET_COUNT_OPTION).cloned(),
    topic_count: matches.get_one::<NonZeroU64>(TOPIC_COUNT_OPTION).cloned(),
  })
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum TenantLabel {
  CertificateCount,
  ConsumerRate,
  Cpu,
  KafkaAclGroupCount,
  Manager,
  Mem,
  Monitoring,
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
      Self::Monitoring => "monitoring",
      Self::PartitionCount => "partition count",
      Self::ProducerRate => "producer rate",
      Self::RequestRate => "request rate",
      Self::SecretCount => "secret count",
      Self::Tenant => "managed tenant",
      Self::TopicCount => "topic count",
      Self::Tracing => "tracing",
      Self::Vpn => "vpn",
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
  fn value(&self, label: &TenantLabel, target_id: &str) -> Value {
    match label {
      TenantLabel::CertificateCount => Value::some_or_hide(self.certificate_count),
      TenantLabel::ConsumerRate => Value::some_or_hide(self.consumer_rate),
      TenantLabel::Cpu => Value::some_or_hide(self.cpu),
      TenantLabel::KafkaAclGroupCount => Value::some_or_hide(self.kafka_acl_group_count),
      TenantLabel::Mem => Value::some_or_hide(self.mem),
      TenantLabel::PartitionCount => Value::some_or_hide(self.partition_count),
      TenantLabel::ProducerRate => Value::some_or_hide(self.producer_rate),
      TenantLabel::RequestRate => Value::some_or_hide(self.request_rate),
      TenantLabel::SecretCount => Value::some_or_hide(self.secret_count),
      TenantLabel::Tenant => Value::target(target_id),
      TenantLabel::TopicCount => Value::some_or_hide(self.topic_count),
      _ => unreachable!(),
    }
  }
}

impl SubjectFormatter<TenantLabel> for ManagedTenant {
  fn value(&self, label: &TenantLabel, _target_id: &str) -> Value {
    match label {
      TenantLabel::Manager => Value::plain(&self.manager),
      TenantLabel::Monitoring => Value::plain(service_enabled(self, ManagedTenantServicesName::Monitoring)),
      TenantLabel::Tracing => Value::plain(service_enabled(self, ManagedTenantServicesName::Tracing)),
      TenantLabel::Vpn => Value::plain(service_enabled(self, ManagedTenantServicesName::Vpn)),
      _ => unreachable!(),
    }
  }
}

impl SubjectFormatter<TenantLabel> for (ManagedTenant, TenantLimits) {
  fn value(&self, label: &TenantLabel, target_id: &str) -> Value {
    match label {
      TenantLabel::Manager | TenantLabel::Monitoring | TenantLabel::Tracing | TenantLabel::Vpn => self.0.value(label, target_id),
      _ => self.1.value(label, target_id),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum StreamAccessLabel {
  ReadAccess,
  StreamId,
  StreamKind,
  Tenant,
  WriteAccess,
  Partitions,
  ReplicationFactor,
}

impl Label for StreamAccessLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::ReadAccess => "read ",
      Self::Partitions => "partitions",
      Self::ReplicationFactor => "replication",
      Self::StreamId => "stream id",
      Self::StreamKind => "kind",
      Self::Tenant => "tenant id",
      Self::WriteAccess => "write",
    }
  }
  fn is_target_label(&self) -> bool {
    matches!(self, Self::Tenant)
  }
}

impl SubjectFormatter<StreamAccessLabel> for (&ManagedStreamId, &str, bool, bool) {
  fn value(&self, label: &StreamAccessLabel, target_id: &str) -> Value {
    match label {
      StreamAccessLabel::ReadAccess => {
        if self.2 {
          Value::plain("granted")
        } else {
          Value::plain("denied")
        }
      }
      StreamAccessLabel::StreamId => Value::plain(self.0),
      StreamAccessLabel::StreamKind => Value::plain(self.1),
      StreamAccessLabel::Tenant => Value::target(target_id),
      StreamAccessLabel::WriteAccess => {
        if self.3 {
          Value::plain("granted")
        } else {
          Value::plain("denied")
        }
      }
      _ => Value::not_applicable(),
    }
  }
}

impl SubjectFormatter<StreamAccessLabel> for (ManagedStreamId, Stream, AccessRights) {
  fn value(&self, label: &StreamAccessLabel, target_id: &str) -> Value {
    match label {
      StreamAccessLabel::Partitions => match &self.1 {
        Stream::Internal { internal_stream } => Value::plain(internal_stream.partitions),
        Stream::Public { public_stream } => Value::plain(public_stream.partitions),
      },
      StreamAccessLabel::ReplicationFactor => match &self.1 {
        Stream::Internal { internal_stream } => Value::plain(internal_stream.replication_factor),
        Stream::Public { public_stream } => Value::plain(public_stream.replication_factor),
      },
      StreamAccessLabel::ReadAccess => {
        if self.2.has_read_access() {
          Value::plain("granted")
        } else {
          Value::plain("denied")
        }
      }
      StreamAccessLabel::StreamId => Value::plain(&self.0),
      StreamAccessLabel::StreamKind => match self.1 {
        Stream::Internal { .. } => Value::plain("internal"),
        Stream::Public { .. } => Value::plain("public"),
      },
      StreamAccessLabel::Tenant => Value::target(target_id),
      StreamAccessLabel::WriteAccess => {
        if self.2.has_write_access() {
          Value::plain("granted")
        } else {
          Value::plain("denied")
        }
      }
    }
  }
}

fn service_enabled(managed_tenant: &ManagedTenant, name: ManagedTenantServicesName) -> String {
  managed_tenant
    .services
    .iter()
    .find_map(|service| if service.name == name { Some(if service.enabled { "enabled".to_string() } else { "disabled".to_string() }) } else { None })
    .unwrap_or_default()
}
