use crate::formatters::formatter::{Label, SubjectFormatter};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::KafkaProxy;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::Subject;
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
    "Show, manage and list Kafka proxies used by the applications/services and apps on the DSH.".to_string()
  }

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Delete, PROXY_DELETE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, PROXY_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, PROXY_SHOW_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Update, PROXY_UPDATE_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref PROXY_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Delete, "Delete proxy")
      .set_long_about("Delete a Kafka proxy.")
      .set_default_command_executor(&ProxyDelete {})
      .add_target_argument(target_argument(PROXY_SUBJECT_TARGET, None))
  );
  pub static ref PROXY_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List proxies")
      .set_long_about("Lists all Kafka proxies used by the applications/services and apps on the DSH.")
      .set_default_command_executor(&ProxyListAll {})
      .add_command_executor(FlagType::Ids, &ProxyListIds {}, None)
      .set_run_all_executors(true)
  );
  pub static ref PROXY_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show Kafka proxy configuration")
      .set_default_command_executor(&ProxyShowConfiguration {})
      .add_target_argument(target_argument(PROXY_SUBJECT_TARGET, None))
  );
  pub static ref PROXY_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Update, "Update proxy")
      .set_long_about("Update a Kafka proxy configuration.")
      .set_default_command_executor(&ProxyUpdateConfiguration {})
      .add_target_argument(target_argument(PROXY_SUBJECT_TARGET, None))
  );
}

struct ProxyDelete {}

#[async_trait]
impl CommandExecutor for ProxyDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete proxy '{}'", proxy_id));
    if context.dsh_api_client.as_ref().unwrap().get_proxy(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    if context.confirmed(format!("type 'yes' to delete proxy '{}': ", proxy_id).as_str())? {
      if context.dry_run {
        context.print_warning("dry-run mode, proxy not deleted");
      } else {
        context.dsh_api_client.as_ref().unwrap().delete_proxy(&proxy_id).await?;
        context.print_outcome(format!("proxy {} deleted", proxy_id));
      }
    } else {
      context.print_outcome(format!("cancelled, proxy {} not deleted", proxy_id));
    }
    Ok(())
  }
}

struct ProxyListAll {}

#[async_trait]
impl CommandExecutor for ProxyListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all proxies with parameters");
    let start_instant = Instant::now();
    let proxy_ids = context.dsh_api_client.as_ref().unwrap().get_proxy_ids().await?;
    let proxys = try_join_all(
      proxy_ids
        .iter()
        .map(|proxy_id| context.dsh_api_client.as_ref().unwrap().get_proxy(proxy_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&PROXY_LABELS_LIST, None, context);
    formatter.push_target_ids_and_values(proxy_ids.as_slice(), proxys.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct ProxyListIds {}

#[async_trait]
impl CommandExecutor for ProxyListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all proxy ids");
    let start_instant = Instant::now();
    let proxy_ids = context.dsh_api_client.as_ref().unwrap().get_proxy_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("proxy id", context);
    formatter.push_target_ids(&proxy_ids);
    formatter.print()?;
    Ok(())
  }
}

struct ProxyShowConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show configuration of proxy '{}'", proxy_id));
    let start_instant = Instant::now();
    let proxy = context.dsh_api_client.as_ref().unwrap().get_proxy(proxy_id.as_str()).await?;
    context.print_execution_time(start_instant);
    let formatter = UnitFormatter::new(proxy_id, &PROXY_LABELS_SHOW, None, &proxy, context);
    formatter.print()?;
    Ok(())
  }
}

struct ProxyUpdateConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyUpdateConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("update configuration for proxy '{}'", proxy_id));
    if context.dsh_api_client.as_ref().unwrap().get_proxy(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    // TODO
    context.print_outcome("function not implemented, proxy not updated");
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum ProxyLabel {
  Certificate,
  Cpus,
  Instances,
  KafkaProxyZone,
  Mem,
  Name,
  SchemaStore,
  SchemaStoreEnabled,
  SecretNameCaChain,
  Target,
  Validations,
}

impl Label for ProxyLabel {
  fn as_str(&self) -> &str {
    match self {
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "number of cpus",
      ProxyLabel::Instances => "number of instances",
      ProxyLabel::KafkaProxyZone => "kafka proxy zone",
      ProxyLabel::Mem => "available memory",
      ProxyLabel::Name => "certificate name",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::SchemaStoreEnabled => "schema store enabled",
      ProxyLabel::SecretNameCaChain => "secret name ca chain",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validation",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "cpus",
      ProxyLabel::Instances => "instances",
      ProxyLabel::KafkaProxyZone => "zone",
      ProxyLabel::Mem => "memory",
      ProxyLabel::Name => "certificate name",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::SchemaStoreEnabled => "schema store",
      ProxyLabel::SecretNameCaChain => "secret name",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validation",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<ProxyLabel> for KafkaProxy {
  fn value(&self, label: &ProxyLabel, target_id: &str) -> String {
    match label {
      ProxyLabel::Certificate => self.certificate.clone(),
      ProxyLabel::Cpus => self.cpus.to_string(),
      ProxyLabel::Instances => self.instances.to_string(),
      ProxyLabel::KafkaProxyZone => self.zone.to_string(),
      ProxyLabel::Mem => self.mem.to_string(),
      ProxyLabel::Name => self.name.clone().unwrap_or_default(),
      ProxyLabel::SchemaStore => {
        if self.schema_store.is_some_and(|enabled| enabled) {
          format!(
            "cpus: {}, mem: {}",
            self.schema_store_cpus.unwrap_or_default(),
            self.schema_store_mem.unwrap_or_default()
          )
        } else {
          "NA".to_string()
        }
      }
      ProxyLabel::SchemaStoreEnabled => self.schema_store.map(|enabled| enabled.to_string()).unwrap_or("NA".to_string()),
      ProxyLabel::SecretNameCaChain => self.secret_name_ca_chain.to_string(),
      ProxyLabel::Target => target_id.to_string(),
      ProxyLabel::Validations => {
        if self.validations.is_empty() {
          "none".to_string()
        } else {
          self
            .validations
            .iter()
            .map(|validation| validation.common_name.clone().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
        }
      }
    }
  }

  fn target_label(&self) -> Option<ProxyLabel> {
    Some(ProxyLabel::Target)
  }
}

pub static PROXY_LABELS_LIST: [ProxyLabel; 6] =
  [ProxyLabel::Target, ProxyLabel::Certificate, ProxyLabel::Cpus, ProxyLabel::Mem, ProxyLabel::KafkaProxyZone, ProxyLabel::SchemaStoreEnabled];

pub static PROXY_LABELS_SHOW: [ProxyLabel; 11] = [
  ProxyLabel::Target,
  ProxyLabel::Certificate,
  ProxyLabel::Cpus,
  ProxyLabel::Instances,
  ProxyLabel::KafkaProxyZone,
  ProxyLabel::Mem,
  ProxyLabel::Name,
  ProxyLabel::SchemaStoreEnabled,
  ProxyLabel::SchemaStore,
  ProxyLabel::SecretNameCaChain,
  ProxyLabel::Validations,
];
