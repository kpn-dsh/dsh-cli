pub(crate) mod bundle_capabilities;
pub(crate) mod labels;
pub(crate) mod options;
pub(crate) mod proxy_capabilities;

use crate::arguments::proxy_id_argument;
use crate::capability::{
  Capability, CODE_COMMAND, CREATE_COMMAND, DELETE_COMMAND, DEPLOY_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS, UNDEPLOY_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::flags::FlagType;
use crate::global_options::expiration_option;
use crate::subject::Subject;
use crate::subjects::proxy::bundle_capabilities::{BundleList, BundleShow, BUNDLE_CODE_CAPABILITY, BUNDLE_CREATE_CAPABILITY, BUNDLE_DELETE_CAPABILITY};
use crate::subjects::proxy::proxy_capabilities::{ProxyList, ProxyListIds, ProxyShow, PROXY_DEPLOY_CAPABILITY, PROXY_UNDEPLOY_CAPABILITY};
use async_trait::async_trait;
use lazy_static::lazy_static;
use std::convert::AsRef;
use std::sync::LazyLock;

struct ProxySubject {}

const PROXY_SUBJECT_TARGET: &str = "proxy";

lazy_static! {
  pub(crate) static ref PROXY_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ProxySubject {});
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
      CODE_COMMAND => Some(BUNDLE_CODE_CAPABILITY.as_ref()),
      CREATE_COMMAND => Some(BUNDLE_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(BUNDLE_DELETE_CAPABILITY.as_ref()),
      DEPLOY_COMMAND => Some(PROXY_DEPLOY_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(PROXY_BUNDLE_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(PROXY_BUNDLE_SHOW_CAPABILITY.as_ref()),
      UNDEPLOY_COMMAND => Some(PROXY_UNDEPLOY_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &PROXY_CAPABILITIES
  }
}

pub(crate) static PROXY_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
    BUNDLE_CODE_CAPABILITY.as_ref(),
    BUNDLE_CREATE_CAPABILITY.as_ref(),
    BUNDLE_DELETE_CAPABILITY.as_ref(),
    PROXY_DEPLOY_CAPABILITY.as_ref(),
    PROXY_BUNDLE_LIST_CAPABILITY.as_ref(),
    PROXY_BUNDLE_SHOW_CAPABILITY.as_ref(),
    PROXY_UNDEPLOY_CAPABILITY.as_ref(),
  ]
});

static PROXY_BUNDLE_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ProxyList {}, "List dsh proxies")
      .set_long_about("Lists all Kafka proxies used by the services and apps on the DSH.")
      .add_command_executor(FlagType::Bundle, &BundleList {}, Some("Lists all local Kafka proxy bundles.".to_string()))
      .add_command_executor(FlagType::Ids, &ProxyListIds {}, None),
  )
});
static PROXY_BUNDLE_SHOW_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ProxyShow {}, "Show Kafka proxy configuration")
      .add_target_argument(proxy_id_argument().required(true))
      .add_command_executor(FlagType::Bundle, &BundleShow {}, Some("Show local Kafka proxy bundle.".to_string()))
      .add_extra_argument(expiration_option()),
  )
});
