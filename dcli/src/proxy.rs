use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::proxy::{PROXY_LABELS_LIST, PROXY_LABELS_SHOW};
use crate::subject::Subject;
use crate::{confirmed, DcliContext, DcliResult};

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

  fn subject_first_upper(&self) -> &'static str {
    "Proxy"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH Kafka proxies.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list Kafka proxies used by the applications/services and apps on the DSH.".to_string()
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
  pub static ref PROXY_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Delete,
    command_about: "Delete proxy".to_string(),
    command_long_about: Some("Delete a Kafka proxy.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&ProxyDelete {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![]
  });
  pub static ref PROXY_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List proxies".to_string(),
    command_long_about: Some("Lists all Kafka proxies used by the applications/services and apps on the DSH.".to_string()),
    command_executors: vec![(FlagType::All, &ProxyListAll {}, None), (FlagType::Ids, &ProxyListIds {}, None)],
    default_command_executor: Some(&ProxyListAll {}),
    run_all_executors: true,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![]
  });
  pub static ref PROXY_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show Kafka proxy configuration".to_string(),
    command_long_about: None,
    command_executors: vec![(FlagType::Configuration, &ProxyShowConfiguration {}, None)],
    default_command_executor: Some(&ProxyShowConfiguration {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![]
  });
  pub static ref PROXY_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Update,
    command_about: "Update proxy".to_string(),
    command_long_about: Some("Update a Kafka proxy configuration.".to_string()),
    command_executors: vec![],
    default_command_executor: Some(&ProxyUpdateConfiguration {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![]
  });
}

struct ProxyDelete {}

#[async_trait]
impl CommandExecutor for ProxyDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("delete proxy '{}'", proxy_id);
    }
    if dsh_api_client.get_proxy(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    println!("type 'yes' and Enter to delete proxy '{}'", proxy_id);
    if confirmed()? {
      dsh_api_client.delete_proxy(&proxy_id).await?;
      println!("ok");
    } else {
      println!("cancelled");
    }
    Ok(false)
  }
}

struct ProxyListAll {}

#[async_trait]
impl CommandExecutor for ProxyListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all proxies with parameters");
    }
    let proxy_ids = dsh_api_client.get_proxy_ids().await?;
    let proxys = try_join_all(proxy_ids.iter().map(|proxy_id| dsh_api_client.get_proxy(proxy_id.as_str()))).await?;
    let mut builder = TableBuilder::list(&PROXY_LABELS_LIST, context);
    for (proxy_id, proxy) in proxy_ids.iter().zip(proxys) {
      builder.value(proxy_id.to_string(), &proxy);
    }
    builder.print();
    Ok(false)
  }
}

struct ProxyListIds {}

#[async_trait]
impl CommandExecutor for ProxyListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all proxy ids");
    }
    print_vec("proxy ids".to_string(), dsh_api_client.get_proxy_ids().await?, context);
    Ok(false)
  }
}

struct ProxyShowConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show configuration of proxy '{}'", proxy_id);
    }
    let proxy = dsh_api_client.get_proxy(proxy_id.as_str()).await?;
    let mut builder = TableBuilder::show(&PROXY_LABELS_SHOW, context);
    builder.value(proxy_id, &proxy);
    builder.print();
    Ok(false)
  }
}

struct ProxyUpdateConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyUpdateConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("update configuration for proxy '{}'", proxy_id);
    }
    if dsh_api_client.get_proxy(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    // TODO
    println!("ok");
    Ok(true)
  }
}
