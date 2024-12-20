use crate::arguments::{get_platform_argument_or_prompt, platform_argument, service_argument, target_argument, tenant_argument, TENANT_ARGUMENT};
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::Subject;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::platform::{DshPlatform, DSH_PLATFORMS};
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;

pub(crate) struct PlatformSubject {}

const PLATFORM_SUBJECT_TARGET: &str = "platform";

lazy_static! {
  pub static ref PLATFORM_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(PlatformSubject {});
}

#[async_trait]
impl Subject for PlatformSubject {
  fn subject(&self) -> &'static str {
    PLATFORM_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, list and open platform resources.".to_string()
  }

  fn requires_dsh_api_client(&self) -> bool {
    false
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, PLATFORM_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Open, PLATFORM_OPEN_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, PLATFORM_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref PLATFORM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List platforms")
      .set_long_about("Lists all dsh platforms.")
      .set_default_command_executor(&PLatformList {}),
  );
  pub static ref PLATFORM_OPEN_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Open, "Open console or web application")
      .set_long_about("Open the DSH console, monitoring page or the web application for the tenant or a service.")
      .set_default_command_executor(&PlatformOpen {})
      .add_extra_arguments(vec![platform_argument(), service_argument(), tenant_argument()])
  );
  pub static ref PLATFORM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show platform data")
      .set_long_about("Show platform data.")
      .set_default_command_executor(&PlatformShow {})
      .add_target_argument(target_argument(PLATFORM_SUBJECT_TARGET, None))
      .add_extra_argument(tenant_argument())
  );
}

struct PLatformList {}

#[async_trait]
impl CommandExecutor for PLatformList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list platforms");
    let mut formatter = ListFormatter::new(&DSH_PLATFORM_LABELS, None, context);
    let full_names = DSH_PLATFORMS.iter().map(|platform| platform.full_name().to_string()).collect::<Vec<_>>();
    formatter.push_target_ids_and_values(&full_names, &DSH_PLATFORMS);
    formatter.print()?;
    Ok(())
  }
}

struct PlatformOpen {}

#[async_trait]
impl CommandExecutor for PlatformOpen {
  async fn execute(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_platform_argument_or_prompt(matches)?;
    match matches.get_one::<String>(TENANT_ARGUMENT) {
      Some(tenant) => {
        let url = platform.console_url_for_tenant(tenant);
        context.print_explanation(format!("open console for tenant {}@{} at url {}", tenant, platform, url));
        if let Err(error) = open::that(url) {
          context.print_error(format!("could not run browser ({})", error));
        }
      }
      None => {
        let url = platform.console_url();
        context.print_explanation(format!("open console for platform '{}' at url {}", platform, url));
        if let Err(error) = open::that(url) {
          context.print_error(format!("could not run browser ({})", error));
        }
      }
    }
    Ok(())
  }
}

struct PlatformShow {}

#[async_trait]
impl CommandExecutor for PlatformShow {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform_id = target.unwrap_or_else(|| unreachable!());
    let platform = DshPlatform::try_from(platform_id.as_str())?;
    match matches.get_one::<String>(TENANT_ARGUMENT) {
      Some(tenant) => {
        context.print_explanation(format!("show platform '{}' and its parameters for tenant '{}'", platform, tenant));
        UnitFormatter::new(
          platform.full_name(),
          &ALL_DSH_PLATFORM_SERVICE_LABELS,
          Some("platform name"),
          &(platform.clone(), tenant.to_string()),
          context,
        )
        .print()?;
      }
      None => {
        context.print_explanation(format!("show platform '{}' and its parameters", platform));
        UnitFormatter::new(platform.full_name(), &ALL_DSH_PLATFORM_LABELS, Some("platform name"), &platform, context).print()?;
      }
    }
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum DshPlatformLabel {
  Alias,
  ApiRestEndpoint,
  AppDomainForTenant,
  CloudProvider,
  ConsoleDomain,
  ConsoleUrlForTenant,
  Description,
  FullName,
  InternalDomainForTenant,
  KeyCloakUrl,
  MonitoringDomainForTenant,
  MqttTokenRestEndpoint,
  Realm,
  RestApiDomain,
  RestClientId,
  RestClientIdForTenant,
  Target,
  VhostDomain,
}

impl Label for DshPlatformLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Alias => "alias",
      Self::ApiRestEndpoint => "api rest endpoint",
      Self::AppDomainForTenant => "app domain for tenant",
      Self::CloudProvider => "cloud provider",
      Self::ConsoleDomain => "console domain",
      Self::ConsoleUrlForTenant => "console url for tenant",
      Self::Description => "description",
      Self::FullName => "full name",
      Self::InternalDomainForTenant => "internal domain for tenant",
      Self::KeyCloakUrl => "key cloak endpoint",
      Self::MonitoringDomainForTenant => "monitoring domain for tenant",
      Self::MqttTokenRestEndpoint => "mqtt token rest endpoint",
      Self::Realm => "realm",
      Self::RestApiDomain => "rest api domain",
      Self::RestClientId => "rest client id",
      Self::RestClientIdForTenant => "rest client id for tenant",
      Self::Target => "target id",
      Self::VhostDomain => "vhost domain",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::FullName)
  }
}

impl SubjectFormatter<DshPlatformLabel> for DshPlatform {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> String {
    match label {
      DshPlatformLabel::Alias => self.alias().to_string(),
      DshPlatformLabel::ApiRestEndpoint => self.api_rest_endpoint(),
      DshPlatformLabel::CloudProvider => self.cloud_provider().to_string(),
      DshPlatformLabel::ConsoleDomain => self.console_domain(),
      DshPlatformLabel::Description => self.description().to_string(),
      DshPlatformLabel::FullName => self.full_name().to_string(),
      DshPlatformLabel::KeyCloakUrl => self.key_cloak_url().to_string(),
      DshPlatformLabel::MqttTokenRestEndpoint => self.mqtt_token_rest_endpoint(),
      DshPlatformLabel::Realm => self.realm().to_string(),
      DshPlatformLabel::RestApiDomain => self.rest_api_domain(),
      DshPlatformLabel::RestClientId => self.rest_client_id(),
      DshPlatformLabel::Target => target_id.to_string(),
      DshPlatformLabel::VhostDomain => self.vhost_domain().to_string(),
      _ => "".to_string(),
    }
  }

  fn target_label(&self) -> Option<DshPlatformLabel> {
    Some(DshPlatformLabel::FullName)
  }
}

impl SubjectFormatter<DshPlatformLabel> for (DshPlatform, String) {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> String {
    match label {
      DshPlatformLabel::Alias => self.0.alias().to_string(),
      DshPlatformLabel::ApiRestEndpoint => self.0.api_rest_endpoint(),
      DshPlatformLabel::CloudProvider => self.0.cloud_provider().to_string(),
      DshPlatformLabel::ConsoleDomain => self.0.console_domain(),
      DshPlatformLabel::Description => self.0.description().to_string(),
      DshPlatformLabel::FullName => self.0.full_name().to_string(),
      DshPlatformLabel::KeyCloakUrl => self.0.key_cloak_url().to_string(),
      DshPlatformLabel::MqttTokenRestEndpoint => self.0.mqtt_token_rest_endpoint(),
      DshPlatformLabel::Realm => self.0.realm().to_string(),
      DshPlatformLabel::RestApiDomain => self.0.rest_api_domain(),
      DshPlatformLabel::RestClientId => self.0.rest_client_id(),
      DshPlatformLabel::Target => target_id.to_string(),
      DshPlatformLabel::VhostDomain => self.0.vhost_domain().to_string(),
      DshPlatformLabel::AppDomainForTenant => self.0.app_domain_for_tenant(self.1.clone()),
      DshPlatformLabel::ConsoleUrlForTenant => self.0.console_url_for_tenant(self.1.clone()),
      DshPlatformLabel::InternalDomainForTenant => self.0.internal_domain_for_tenant(self.1.clone()),
      DshPlatformLabel::MonitoringDomainForTenant => self.0.monitoring_domain_for_tenant(self.1.clone()),
      DshPlatformLabel::RestClientIdForTenant => self.0.rest_client_id_for_tenant(self.1.clone()),
    }
  }

  fn target_label(&self) -> Option<DshPlatformLabel> {
    Some(DshPlatformLabel::FullName)
  }
}

pub static DSH_PLATFORM_LABELS: [DshPlatformLabel; 4] = [DshPlatformLabel::FullName, DshPlatformLabel::Alias, DshPlatformLabel::Realm, DshPlatformLabel::ConsoleDomain];

pub static ALL_DSH_PLATFORM_LABELS: [DshPlatformLabel; 12] = [
  DshPlatformLabel::Alias,
  DshPlatformLabel::ApiRestEndpoint,
  DshPlatformLabel::CloudProvider,
  DshPlatformLabel::ConsoleDomain,
  DshPlatformLabel::Description,
  DshPlatformLabel::FullName,
  DshPlatformLabel::KeyCloakUrl,
  DshPlatformLabel::MqttTokenRestEndpoint,
  DshPlatformLabel::Realm,
  DshPlatformLabel::RestApiDomain,
  DshPlatformLabel::RestClientId,
  DshPlatformLabel::VhostDomain,
];

pub static ALL_DSH_PLATFORM_SERVICE_LABELS: [DshPlatformLabel; 18] = [
  DshPlatformLabel::Alias,
  DshPlatformLabel::ApiRestEndpoint,
  DshPlatformLabel::AppDomainForTenant,
  DshPlatformLabel::CloudProvider,
  DshPlatformLabel::ConsoleDomain,
  DshPlatformLabel::ConsoleUrlForTenant,
  DshPlatformLabel::Description,
  DshPlatformLabel::FullName,
  DshPlatformLabel::InternalDomainForTenant,
  DshPlatformLabel::KeyCloakUrl,
  DshPlatformLabel::MonitoringDomainForTenant,
  DshPlatformLabel::MqttTokenRestEndpoint,
  DshPlatformLabel::Realm,
  DshPlatformLabel::RestApiDomain,
  DshPlatformLabel::RestClientId,
  DshPlatformLabel::RestClientIdForTenant,
  DshPlatformLabel::Target,
  DshPlatformLabel::VhostDomain,
];
