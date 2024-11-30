use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use lazy_static::lazy_static;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::DcliContext;
use crate::flags::FlagType;
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::proxy::{PROXY_LABELS_LIST, PROXY_LABELS_SHOW};
use crate::subject::Subject;
use crate::{confirmed, DcliResult};

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
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_capability_explanation(format!("delete proxy '{}'", proxy_id));
    if context.dsh_api_client.as_ref().unwrap().get_proxy(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    if confirmed(format!("type 'yes' to delete proxy '{}': ", proxy_id).as_str())? {
      context.dsh_api_client.as_ref().unwrap().delete_proxy(&proxy_id).await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    context.print_capability_explanation("list all proxies with parameters");
    let proxy_ids = context.dsh_api_client.as_ref().unwrap().get_proxy_ids().await?;
    let proxys = try_join_all(
      proxy_ids
        .iter()
        .map(|proxy_id| context.dsh_api_client.as_ref().unwrap().get_proxy(proxy_id.as_str())),
    )
    .await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    context.print_capability_explanation("list all proxy ids");
    print_vec("proxy ids".to_string(), context.dsh_api_client.as_ref().unwrap().get_proxy_ids().await?, context);
    Ok(false)
  }
}

struct ProxyShowConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_capability_explanation(format!("show configuration of proxy '{}'", proxy_id));
    let proxy = context.dsh_api_client.as_ref().unwrap().get_proxy(proxy_id.as_str()).await?;
    let mut builder = TableBuilder::show(&PROXY_LABELS_SHOW, context);
    builder.value(proxy_id, &proxy);
    builder.print();
    Ok(false)
  }
}

struct ProxyUpdateConfiguration {}

#[async_trait]
impl CommandExecutor for ProxyUpdateConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_capability_explanation(format!("update configuration for proxy '{}'", proxy_id));
    if context.dsh_api_client.as_ref().unwrap().get_proxy(&proxy_id).await.is_err() {
      return Err(format!("proxy '{}' does not exists", proxy_id));
    }
    // TODO
    println!("ok");
    Ok(true)
  }
}
