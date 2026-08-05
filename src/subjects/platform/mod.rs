mod capabilities;
mod labels;

use crate::arguments::{app_id_argument, bucket_id_argument, proxy_id_argument, service_id_argument, topic_id_argument, vendor_name_argument, vhost_subdomain_argument};
use crate::capability::{Capability, EXPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::subject::Subject;
use crate::subjects::platform::capabilities::{PLatformList, PlatformExport, PlatformOpen, PlatformShow};
use async_trait::async_trait;
use clap::Command;
use std::sync::LazyLock;

struct PlatformSubject {}

const PLATFORM_SUBJECT_TARGET: &str = "platform";

pub(crate) static PLATFORM_SUBJECT: LazyLock<Box<dyn Subject + Send + Sync>> = LazyLock::new(|| Box::new(PlatformSubject {}));

#[async_trait]
impl Subject for PlatformSubject {
  fn subject(&self) -> &'static str {
    PLATFORM_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, list and open platform resources.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("p")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      EXPORT_COMMAND => Some(PLATFORM_EXPORT_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(PLATFORM_LIST_CAPABILITY.as_ref()),
      OPEN_COMMAND => Some(PLATFORM_OPEN_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(PLATFORM_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &PLATFORM__CAPABILITIES
  }
}

static PLATFORM_EXPORT_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(EXPORT_COMMAND, None, &PlatformExport {}, "Export default platform configuration").set_long_about(
      "Export the default platform configuration json file from the dsh-api library. \
        This file can be used as a starting point when platform customization is required.",
    ),
  )
});

static PLATFORM_LIST_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> =
  LazyLock::new(|| Box::new(CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &PLatformList {}, "List platforms").set_long_about("Lists all dsh platforms.")));

const OPEN_APP: &str = "app";
const OPEN_CONSOLE: &str = "console";
const OPEN_MONITORING: &str = "monitoring";
const OPEN_SERVICE: &str = "service";
const OPEN_SWAGGER: &str = "swagger";
const OPEN_TENANT: &str = "tenant";
const OPEN_TRACING: &str = "tracing";

static PLATFORM_OPEN_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(OPEN_COMMAND, Some(OPEN_COMMAND_ALIAS), &PlatformOpen {}, "Open console or web application")
      .set_long_about("Open the DSH console, monitoring page or the web application for the tenant or a service.")
      .add_subcommands(vec![
        Command::new(OPEN_APP)
          .about("Open the console for the target platform/tenant and the provided app")
          .alias("a")
          .arg(app_id_argument().required(true)),
        Command::new(OPEN_CONSOLE).about("Open the console for the target platform").alias("c"),
        Command::new(OPEN_MONITORING)
          .about("Open the monitoring web application for the target platform/tenant")
          .alias("m"),
        Command::new(OPEN_SERVICE)
          .about("Open the console for the target platform/tenant and the provided service")
          .alias("s")
          .arg(service_id_argument().required(true)),
        Command::new(OPEN_SWAGGER).about("Open the swagger web application for the target platform and copy a fresh token to the clipboard"),
        Command::new(OPEN_TENANT).about("Open the console for the target platform/tenant").alias("t"),
        Command::new(OPEN_TRACING).about("Open the tracing application for the target platform"),
      ]),
  )
});
static PLATFORM_SHOW_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &PlatformShow {}, "Show platform data")
      .set_long_about("Show platform data.")
      .add_extra_arguments(vec![
        app_id_argument().long("app"),
        bucket_id_argument().long("bucket"),
        proxy_id_argument().long("proxy"),
        service_id_argument().long("service"),
        topic_id_argument().long("topic"),
        vendor_name_argument().long("vendor"),
        vhost_subdomain_argument().long("vhost"),
      ]),
  )
});
static PLATFORM__CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> =
  LazyLock::new(|| vec![PLATFORM_EXPORT_CAPABILITY.as_ref(), PLATFORM_LIST_CAPABILITY.as_ref(), PLATFORM_OPEN_CAPABILITY.as_ref(), PLATFORM_SHOW_CAPABILITY.as_ref()]);
