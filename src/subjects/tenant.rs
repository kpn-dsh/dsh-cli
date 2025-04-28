use crate::arguments::managed_tenant_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS, UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::limits_flags::{
  certificate_count_flag, consumer_rate_flag, cpu_flag, kafka_acl_group_flag, mem_flag, partition_count_flag, producer_rate_flag, request_rate_flag, secret_count_flag,
  topic_count_flag, tracing_flag, vpn_flag, CERTIFICATE_COUNT_FLAG, CONSUMER_RATE_FLAG, CPU_FLAG, KAFKA_ACL_GROUP_COUNT_FLAG, MEM_FLAG, PARTITION_COUNT_FLAG, PRODUCER_RATE_FLAG,
  REQUEST_RATE_FLAG, SECRET_COUNT_FLAG, TOPIC_COUNT_FLAG, TRACING_FLAG, VPN_FLAG,
};
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::tenant::TenantLimits;
use dsh_api::types::{
  LimitValue, LimitValueCertificateCount, LimitValueCertificateCountName, LimitValueConsumerRate, LimitValueConsumerRateName, LimitValueCpu, LimitValueCpuName,
  LimitValueKafkaAclGroupCount, LimitValueKafkaAclGroupCountName, LimitValueMem, LimitValueMemName, LimitValuePartitionCount, LimitValuePartitionCountName, LimitValueProducerRate,
  LimitValueProducerRateName, LimitValueRequestRate, LimitValueRequestRateName, LimitValueSecretCount, LimitValueSecretCountName, LimitValueTopicCount, LimitValueTopicCountName,
  ManagedTenant, ManagedTenantServices, ManagedTenantServicesName, PutTenantLimitByManagerByTenantByKindKind,
};
use dsh_api::DshApiError;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;
use serde::Serialize;

pub(crate) struct TenantSubject {}

const TENANT_SUBJECT_TARGET: &str = "tenant";

lazy_static! {
  pub static ref TENANT_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TenantSubject {});
}

const HELP_HEADING: &str = "Tenant options";

lazy_static! {
  static ref TENANT_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &TenantCreate {}, "Create managed tenant")
      .set_long_about("Create a configured managed tenant.")
      .add_target_argument(managed_tenant_argument().required(true))
      .add_extra_argument(tracing_flag().help_heading(HELP_HEADING))
      .add_extra_argument(vpn_flag().help_heading(HELP_HEADING))
  );
  static ref TENANT_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &TenantDelete {}, "Delete managed tenant")
      .set_long_about("Delete a managed tenant and its configuration.")
      .add_target_argument(managed_tenant_argument().required(true))
  );
  static ref TENANT_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TenantListAll {}, "List managed tenants")
      .set_long_about("Lists all managed tenants.")
      .add_target_argument(managed_tenant_argument())
      .add_command_executor(FlagType::Ids, &TenantListIds {}, None)
  );
  static ref TENANT_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &TenantShowAll {}, "Show managed tenant configuration")
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
      .add_extra_argument(kafka_acl_group_flag().help_heading(HELP_HEADING))
      .add_extra_argument(mem_flag().help_heading(HELP_HEADING))
      .add_extra_argument(partition_count_flag().help_heading(HELP_HEADING))
      .add_extra_argument(producer_rate_flag().help_heading(HELP_HEADING))
      .add_extra_argument(request_rate_flag().help_heading(HELP_HEADING))
      .add_extra_argument(secret_count_flag().help_heading(HELP_HEADING))
      .add_extra_argument(topic_count_flag().help_heading(HELP_HEADING))
      .add_extra_argument(tracing_flag().help_heading(HELP_HEADING))
      .add_extra_argument(vpn_flag().help_heading(HELP_HEADING))
  );
  static ref TENANT_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    TENANT_CREATE_CAPABILITY.as_ref(),
    TENANT_DELETE_CAPABILITY.as_ref(),
    TENANT_LIST_CAPABILITY.as_ref(),
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

struct TenantCreate {}

#[async_trait]
impl CommandExecutor for TenantCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    if client.get_tenant_configuration(&tenant_id).await.is_ok() {
      return Err(format!("managed tenant '{}' already exists", tenant_id));
    }
    let enable_tracing = matches.get_one::<bool>(TRACING_FLAG);
    let enable_vpn = matches.get_one::<bool>(VPN_FLAG);
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
    if context.dry_run {
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
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete tenant '{}'", tenant_id));
    if client.get_tenant_configuration(&tenant_id).await.is_err() {
      return Err(format!("tenant '{}' does not exist or you are not authorized to manage it", tenant_id));
    }
    if context.confirmed(format!("delete tenant '{}'?", tenant_id))? {
      if context.dry_run {
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

struct TenantListAll {}

#[async_trait]
impl CommandExecutor for TenantListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all tenants with their limits");
    let start_instant = context.now();
    let tenant_ids: Vec<String> = client.get_tenant_ids().await?;
    if tenant_ids.is_empty() {
      context.print_outcome("no managed tenants or you are not authorized to manage tenants");
    } else {
      let (managed_tenants, limits) = try_join!(
        try_join_all(tenant_ids.iter().map(|tenant_id| client.get_tenant_configuration(tenant_id))),
        try_join_all(tenant_ids.iter().map(|tenant_id| client.get_tenantlimits(tenant_id)))
      )?;
      context.print_execution_time(start_instant);
      let managed_tenants_limits: Vec<(ManagedTenant, TenantLimits)> = managed_tenants.into_iter().zip(limits).collect::<Vec<_>>();
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

struct TenantShowAll {}

#[async_trait]
impl CommandExecutor for TenantShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let tenant_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all limits for tenant '{}'", tenant_id));
    let start_instant = context.now();
    match try_join!(client.get_tenant_configuration(&tenant_id), client.get_tenantlimits(&tenant_id)) {
      Ok((managed_tenant, tenant_limits)) => {
        context.print_execution_time(start_instant);
        UnitFormatter::new(tenant_id, &TENANT_LABELS, Some("tenant id"), context).print(&(managed_tenant, tenant_limits), None)
      }
      Err(error) => match error {
        DshApiError::NotFound(None) => {
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
    let enable_tracing_argument = matches.get_one::<bool>(TRACING_FLAG);
    let enable_vpn_argument = matches.get_one::<bool>(VPN_FLAG);
    let tenant_limits_from_arguments = tenant_limits_try_from_matches(matches)?;

    match (
      enable_tracing_argument.is_some() || enable_vpn_argument.is_some(),
      !tenant_limits_from_arguments.is_empty(),
    ) {
      (false, false) => Err("at least one limit or capability argument must be provided".to_string()),
      (false, true) => {
        // TODO This code must be replaced once PATCH /manage/{manager}/tenant/{tenant}/limit is fixed
        context.print_explanation(format!("update limit of managed tenant '{}'", tenant_id));
        if client.get_tenant_limits(&tenant_id).await.is_err() {
          return Err(format!("tenant '{}' does not exist or you are not authorized to manage it", tenant_id));
        }
        if let Some(certificate_count) = tenant_limits_from_arguments.certificate_count {
          if context.dry_run {
            context.print_warning("dry-run mode, certificate count limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Certificatecount,
                &LimitValue::CertificateCount(LimitValueCertificateCount { name: LimitValueCertificateCountName::CertificateCount, value: certificate_count }),
              )
              .await?;
          }
        }
        if let Some(consumer_rate) = tenant_limits_from_arguments.consumer_rate {
          if context.dry_run {
            context.print_warning("dry-run mode, consumer rate limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Consumerrate,
                &LimitValue::ConsumerRate(LimitValueConsumerRate { name: LimitValueConsumerRateName::ConsumerRate, value: consumer_rate }),
              )
              .await?;
          }
        }
        if let Some(cpu) = tenant_limits_from_arguments.cpu {
          if context.dry_run {
            context.print_warning("dry-run mode, cpu limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Cpu,
                &LimitValue::Cpu(LimitValueCpu { name: LimitValueCpuName::Cpu, value: cpu }),
              )
              .await?;
          }
        }
        if let Some(kafka_acl_group_count) = tenant_limits_from_arguments.kafka_acl_group_count {
          if context.dry_run {
            context.print_warning("dry-run mode, kafka acl group count limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Kafkaaclgroupcount,
                &LimitValue::KafkaAclGroupCount(LimitValueKafkaAclGroupCount { name: LimitValueKafkaAclGroupCountName::KafkaAclGroupCount, value: kafka_acl_group_count }),
              )
              .await?;
          }
        }
        if let Some(mem) = tenant_limits_from_arguments.mem {
          if context.dry_run {
            context.print_warning("dry-run mode, mem limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Mem,
                &LimitValue::Mem(LimitValueMem { name: LimitValueMemName::Mem, value: mem }),
              )
              .await?;
          }
        }
        if let Some(partition_count) = tenant_limits_from_arguments.partition_count {
          if context.dry_run {
            context.print_warning("dry-run mode, partition count limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Partitioncount,
                &LimitValue::PartitionCount(LimitValuePartitionCount { name: LimitValuePartitionCountName::PartitionCount, value: partition_count }),
              )
              .await?;
          }
        }
        if let Some(producer_rate) = tenant_limits_from_arguments.producer_rate {
          if context.dry_run {
            context.print_warning("dry-run mode, producer rate limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Producerrate,
                &LimitValue::ProducerRate(LimitValueProducerRate { name: LimitValueProducerRateName::ProducerRate, value: producer_rate }),
              )
              .await?;
          }
        }
        if let Some(request_rate) = tenant_limits_from_arguments.request_rate {
          if context.dry_run {
            context.print_warning("dry-run mode, request rate limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Requestrate,
                &LimitValue::RequestRate(LimitValueRequestRate { name: LimitValueRequestRateName::RequestRate, value: request_rate }),
              )
              .await?;
          }
        }
        if let Some(secret_count) = tenant_limits_from_arguments.secret_count {
          if context.dry_run {
            context.print_warning("dry-run mode, secret count limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Secretcount,
                &LimitValue::SecretCount(LimitValueSecretCount { name: LimitValueSecretCountName::SecretCount, value: secret_count }),
              )
              .await?;
          }
        }
        if let Some(topic_count) = tenant_limits_from_arguments.topic_count {
          if context.dry_run {
            context.print_warning("dry-run mode, topic count limit not updated");
          } else {
            client
              .put_tenant_limit(
                &tenant_id,
                PutTenantLimitByManagerByTenantByKindKind::Topiccount,
                &LimitValue::TopicCount(LimitValueTopicCount { name: LimitValueTopicCountName::TopicCount, value: topic_count }),
              )
              .await?;
          }
        }
        Ok(())
      }
      // (false, true) => {
      //   // TODO This code must be used once PATCH /manage/{manager}/tenant/{tenant}/limit is fixed
      //   context.print_explanation(format!("update limits of managed tenant '{}'", tenant_id));
      //   match client.get_tenantlimits(&tenant_id).await {
      //     Ok(current_tenant_limits) => {
      //       let mut updated_tenant_limits = current_tenant_limits.clone();
      //       updated_tenant_limits.update(tenant_limits_from_arguments);
      //       if current_tenant_limits != updated_tenant_limits {
      //         if context.dry_run {
      //           context.print_warning("dry-run mode, limits not updated");
      //         } else {
      //           let limit_values: Vec<LimitValue> = updated_tenant_limits.try_into()?;
      //           client.patch_tenant_limit(&tenant_id, &limit_values).await?;
      //           context.print_outcome(format!("limits for managed tenant '{}' updated", tenant_id));
      //         }
      //       } else {
      //         context.print_outcome("provided limits are equal to the current managed tenant limits, limits not updated");
      //       }
      //       Ok(())
      //     }
      //     Err(error) => match error {
      //       DshApiError::NotFound(None) => {
      //         context.print_error(format!("managed tenant '{}' does not exist or you are not authorized to manage it", tenant_id));
      //         Ok(())
      //       }
      //       error => Err(String::from(error)),
      //     },
      //   }
      // }
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
              if context.dry_run {
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
          Err(error) => match error {
            DshApiError::NotFound(None) => {
              context.print_error(format!("managed tenant '{}' does not exist or you are not authorized to manage it", tenant_id));
              Ok(())
            }
            error => Err(String::from(error)),
          },
        }
      }
      (true, true) => Err("provide either limit arguments or capability arguments, but not both".to_string()),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn tenant_limits_try_from_matches(matches: &ArgMatches) -> Result<TenantLimits, String> {
  Ok(TenantLimits {
    certificate_count: matches.get_one::<i64>(CERTIFICATE_COUNT_FLAG).cloned(),
    consumer_rate: matches.get_one::<i64>(CONSUMER_RATE_FLAG).cloned(),
    cpu: match matches.get_one::<f64>(CPU_FLAG).cloned() {
      Some(cpus) => {
        if (0.01..=16.0).contains(&cpus) {
          Some(cpus)
        } else {
          return Err("number of cpus should be greater than or equal to 0.01 and lower than or equal to 16.0".to_string());
        }
      }
      None => None,
    },
    kafka_acl_group_count: matches.get_one::<i64>(KAFKA_ACL_GROUP_COUNT_FLAG).cloned(),
    mem: matches.get_one::<i64>(MEM_FLAG).cloned(),
    partition_count: matches.get_one::<i64>(PARTITION_COUNT_FLAG).cloned(),
    producer_rate: matches.get_one::<i64>(PRODUCER_RATE_FLAG).cloned(),
    request_rate: matches.get_one::<i64>(REQUEST_RATE_FLAG).cloned(),
    secret_count: matches.get_one::<i64>(SECRET_COUNT_FLAG).cloned(),
    topic_count: matches.get_one::<i64>(TOPIC_COUNT_FLAG).cloned(),
  })
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
  _Name,
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
      Self::_Name => "name",
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
      Self::_Name => "name",
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

pub static TENANT_LABELS: [TenantLabel; 15] = [
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

impl SubjectFormatter<TenantLabel> for ManagedTenant {
  fn value(&self, label: &TenantLabel, _target_id: &str) -> String {
    match label {
      TenantLabel::Manager => self.manager.to_string(),
      TenantLabel::Monitoring => service_enabled(self, ManagedTenantServicesName::Monitoring),
      TenantLabel::_Name => self.name.to_string(),
      TenantLabel::Tracing => service_enabled(self, ManagedTenantServicesName::Tracing),
      TenantLabel::Vpn => service_enabled(self, ManagedTenantServicesName::Vpn),
      _ => unreachable!(),
    }
  }
}

impl SubjectFormatter<TenantLabel> for (ManagedTenant, TenantLimits) {
  fn value(&self, label: &TenantLabel, target_id: &str) -> String {
    match label {
      TenantLabel::Manager | TenantLabel::Monitoring | TenantLabel::_Name | TenantLabel::Tracing | TenantLabel::Vpn => self.0.value(label, target_id),
      _ => self.1.value(label, target_id),
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

pub static _MANAGED_TENANT_LABELS: [TenantLabel; 5] = [TenantLabel::_Name, TenantLabel::Manager, TenantLabel::Monitoring, TenantLabel::Tracing, TenantLabel::Vpn];
