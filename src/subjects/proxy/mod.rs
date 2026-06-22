pub(crate) mod bundle_capabilities;
pub(crate) mod labels;
pub(crate) mod options;
pub(crate) mod proxy_capabilities;

use crate::arguments::proxy_id_argument;
use crate::capability::{
  Capability, CODE_COMMAND, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, DEPLOY_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND,
  SHOW_COMMAND_ALIAS, UNDEPLOY_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::flags::FlagType;
use crate::global_options::expiration_option;
use crate::subject::Subject;
use crate::subjects::aclgroup::options::acl_group_name_option;
use crate::subjects::proxy::bundle_capabilities::{BundleCode, BundleCodeConfiguration, BundleCreate, BundleDelete, BundleList, BundleShow};
use crate::subjects::proxy::options::{ca_common_name_option, enable_schema_store_option, language_argument, number_of_dns_records_option, vhost_zone_option};
use crate::subjects::proxy::proxy_capabilities::{ProxyDeploy, ProxyList, ProxyListIds, ProxyShow, ProxyUndeploy};
use crate::subjects::service::{cpus_option, instances_option, mem_option};
use crate::COMMAND_OPTIONS_HEADING;
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
    PROXY_BUNDLE_LIST_CAPABILITY.as_ref(),
    PROXY_BUNDLE_SHOW_CAPABILITY.as_ref(),
    PROXY_DEPLOY_CAPABILITY.as_ref(),
    PROXY_UNDEPLOY_CAPABILITY.as_ref(),
  ]
});

static BUNDLE_CODE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CODE_COMMAND, None, &BundleCode {}, "Generate example client code")
      .add_target_argument(proxy_id_argument().required(true))
      .add_target_argument(language_argument())
      .add_command_executor(FlagType::Configuration, &BundleCodeConfiguration {}, None),
  )
});

static BUNDLE_CREATE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(
      CREATE_COMMAND,
      Some(CREATE_COMMAND_ALIAS),
      &BundleCreate {},
      "Create local proxy certificates bundle",
    )
    .add_target_argument(proxy_id_argument().required(true))
    .add_extra_argument(acl_group_name_option())
    .add_extra_argument(ca_common_name_option())
    .add_extra_argument(enable_schema_store_option())
    .add_extra_argument(number_of_dns_records_option())
    .add_extra_argument(vhost_zone_option()),
  )
});
static BUNDLE_DELETE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(
      DELETE_COMMAND,
      Some(DELETE_COMMAND_ALIAS),
      &BundleDelete {},
      "Delete local proxy certificates bundle",
    )
    .add_target_argument(proxy_id_argument().required(true)),
  )
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
static PROXY_DEPLOY_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DEPLOY_COMMAND, None, &ProxyDeploy {}, "Deploy local proxy on dsh")
      .set_long_about("Deploy a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true))
      .add_extra_argument(cpus_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(instances_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(mem_option().help_heading(COMMAND_OPTIONS_HEADING)),
  )
});
pub(crate) static PROXY_UNDEPLOY_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UNDEPLOY_COMMAND, None, &ProxyUndeploy {}, "Undeploy proxy from dsh")
      .set_long_about("Undeploy a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true)),
  )
});
