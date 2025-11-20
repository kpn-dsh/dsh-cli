use crate::formatters::formatter::{Label, SubjectFormatter};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::KafkaProxy;
use futures::future::try_join_all;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::arguments::proxy_id_argument;
use crate::capability::{Capability, CommandExecutor, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;

pub(crate) struct ProxySubject {}

const PROXY_SUBJECT_TARGET: &str = "proxy";

lazy_static! {
  pub static ref PROXY_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ProxySubject {});
}

#[async_trait]
impl Subject for ProxySubject {
  fn subject(&self) -> &'static str {
    PROXY_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH Kafka proxies.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list Kafka proxies used by the services and apps on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(PROXY_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(PROXY_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(PROXY_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &PROXY_CAPABILITIES
  }
}

lazy_static! {
  static ref PROXY_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &ProxyDelete {}, "Delete proxy")
      .set_long_about("Delete a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true))
  );
  static ref PROXY_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ProxyListAll {}, "List proxies")
      .set_long_about("Lists all Kafka proxies used by the services and apps on the DSH.")
      .add_command_executor(FlagType::Ids, &ProxyListIds {}, None)
  );
  static ref PROXY_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ProxyShowConfiguration {}, "Show Kafka proxy configuration")
      .add_target_argument(proxy_id_argument().required(true))
  );
  static ref PROXY_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![PROXY_DELETE_CAPABILITY.as_ref(), PROXY_LIST_CAPABILITY.as_ref(), PROXY_SHOW_CAPABILITY.as_ref()];
}

struct ProxyDelete {}

#[async_trait]
impl CommandExecutor for ProxyDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    if client.get_kafkaproxy_configuration(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    if context.confirmed(format!("delete proxy '{}'?", proxy_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, proxy not deleted");
      } else {
        client.delete_kafkaproxy_configuration(&proxy_id).await?;
        context.print_outcome(format!("proxy '{}' deleted", proxy_id));
      }
    } else {
      context.print_outcome(format!("cancelled, proxy '{}' not deleted", proxy_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyListAll {}

#[async_trait]
impl CommandExecutor for ProxyListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all proxies with parameters");
    let start_instant = context.now();
    let proxy_ids = client.get_kafkaproxy_ids().await?;
    let proxys = try_join_all(proxy_ids.iter().map(|proxy_id| client.get_kafkaproxy_configuration(proxy_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&PROXY_LABELS_LIST, None, context);
    formatter.push_target_ids_and_values(proxy_ids.as_slice(), proxys.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyListIds {}

#[async_trait]
impl CommandExecutor for ProxyListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all proxy ids");
    let start_instant = context.now();
    let proxy_ids = client.get_kafkaproxy_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("proxy id", context);
    formatter.push_target_ids(&proxy_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyShowConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyShowConfiguration {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show configuration of proxy '{}'", proxy_id));
    let start_instant = context.now();
    let proxy = client.get_kafkaproxy_configuration(&proxy_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(proxy_id, &PROXY_LABELS_SHOW, None, context).print(&proxy, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum ProxyLabel {
  AclGroupsEnabled,
  CaChainSecretName,
  Certificate,
  Cpus,
  Instances,
  Mem,
  Name,
  SchemaStore,
  Target,
  Validations,
  Zone,
}

impl Label for ProxyLabel {
  fn as_str(&self) -> &str {
    match self {
      ProxyLabel::AclGroupsEnabled => "acl groups",
      ProxyLabel::CaChainSecretName => "ca chain secret name",
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "cpus",
      ProxyLabel::Instances => "instances",
      ProxyLabel::Mem => "memory",
      ProxyLabel::Name => "certificate",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validation",
      ProxyLabel::Zone => "zone",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      ProxyLabel::AclGroupsEnabled => "acl groups",
      ProxyLabel::CaChainSecretName => "secret name",
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "cpus",
      ProxyLabel::Instances => "instances",
      ProxyLabel::Mem => "memory",
      ProxyLabel::Name => "certificate name",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validation",
      ProxyLabel::Zone => "zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<ProxyLabel> for KafkaProxy {
  fn value(&self, label: &ProxyLabel, target_id: &str) -> String {
    match label {
      ProxyLabel::AclGroupsEnabled => self.enable_kafka_acl_groups.map(|enabled| enabled.to_string()).unwrap_or_default(),
      ProxyLabel::CaChainSecretName => self.secret_name_ca_chain.to_string(),
      ProxyLabel::Certificate => self.certificate.clone(),
      ProxyLabel::Cpus => self.cpus.to_string(),
      ProxyLabel::Instances => self.instances.to_string(),
      ProxyLabel::Mem => self.mem.to_string(),
      ProxyLabel::Name => self.name.clone().unwrap_or_default(),
      ProxyLabel::SchemaStore => self
        .schema_store
        .map(|enabled| {
          if enabled {
            format!(
              "true (cpus: {}, mem: {})",
              self.schema_store_cpus.map(|cpus| cpus.to_string()).unwrap_or("NA".to_string()),
              self.schema_store_mem.map(|mem| mem.to_string()).unwrap_or("NA".to_string())
            )
          } else {
            "false".to_string()
          }
        })
        .unwrap_or("NA".to_string()),
      ProxyLabel::Target => target_id.to_string(),
      ProxyLabel::Validations => {
        if self.validations.is_empty() {
          "none".to_string()
        } else {
          self
            .validations
            .iter()
            .map(|validation| validation.common_name.clone().unwrap_or_default())
            .collect_vec()
            .join("\n")
        }
      }
      ProxyLabel::Zone => self.zone.to_string(),
    }
  }
}

pub static PROXY_LABELS_LIST: [ProxyLabel; 6] = [ProxyLabel::Target, ProxyLabel::Certificate, ProxyLabel::Cpus, ProxyLabel::Mem, ProxyLabel::Zone, ProxyLabel::SchemaStore];

pub static PROXY_LABELS_SHOW: [ProxyLabel; 11] = [
  ProxyLabel::Target,
  ProxyLabel::Certificate,
  ProxyLabel::Cpus,
  ProxyLabel::Instances,
  ProxyLabel::Zone,
  ProxyLabel::Mem,
  ProxyLabel::Name,
  ProxyLabel::SchemaStore,
  ProxyLabel::CaChainSecretName,
  ProxyLabel::Validations,
  ProxyLabel::AclGroupsEnabled,
];
