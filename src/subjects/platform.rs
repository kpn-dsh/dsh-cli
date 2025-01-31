use crate::arguments::{
  app_argument, service_argument, target_argument, tenant_argument, vendor_argument, vhost_argument, APP_ARGUMENT, SERVICE_ARGUMENT, TENANT_ARGUMENT, VENDOR_ARGUMENT,
  VHOST_ARGUMENT,
};
use crate::capability::{
  Capability, CommandExecutor, DEFAULT_COMMAND, DEFAULT_COMMAND_PAIR, LIST_COMMAND, LIST_COMMAND_PAIR, OPEN_COMMAND, OPEN_COMMAND_PAIR, SHOW_COMMAND, SHOW_COMMAND_PAIR,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::{read_single_line, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::{ArgMatches, Command};
use dsh_api::platform::DshPlatform;
use dsh_api::DEFAULT_PLATFORMS;
use lazy_static::lazy_static;
use serde::Serialize;

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

  fn subject_command_alias(&self) -> Option<&str> {
    Some("p")
  }

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements {
    let needs_dsh_api_client = match sub_matches.subcommand() {
      Some((OPEN_COMMAND, subcommand_matches)) => matches!(subcommand_matches.subcommand().unwrap_or_else(|| unreachable!()).0, OPEN_TARGET_SWAGGER),
      _ => false,
    };
    Requirements::new(needs_dsh_api_client, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DEFAULT_COMMAND => Some(PLATFORM_DEFAULT.as_ref()),
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

const OPEN_TARGET_CONSOLE: &str = "console";
const OPEN_TARGET_CONSOLE_TENANT_APP: &str = "app";
const OPEN_TARGET_CONSOLE_TENANT_SERVICE: &str = "service";
const OPEN_TARGET_MONITORING_TENANT: &str = "monitoring";
const OPEN_TARGET_SWAGGER: &str = "swagger";
const OPEN_TARGET_TRACING: &str = "tracing";

lazy_static! {
  static ref PLATFORM_DEFAULT: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DEFAULT_COMMAND_PAIR, "Show default platform configuration")
      .set_long_about(
        "Show the default platform configuration json file from the dsh-api library. \
        This file can be used as a starting point when platform customization is required."
      )
      .set_default_command_executor(&PlatformDefault {})
  );
  static ref PLATFORM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List platforms")
      .set_long_about("Lists all dsh platforms.")
      .set_default_command_executor(&PLatformList {}),
  );
  static ref PLATFORM_OPEN_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(OPEN_COMMAND_PAIR, "Open console or web application")
      .set_long_about("Open the DSH console, monitoring page or the web application for the tenant or a service.")
      .set_default_command_executor(&PlatformOpen {})
      .add_subcommands(vec![
        Command::new(OPEN_TARGET_CONSOLE)
          .about("Open the console for the platform, and optionally the tenant")
          .alias("c")
          .arg(service_argument()),
        Command::new(OPEN_TARGET_CONSOLE_TENANT_APP)
          .about("Open the console for the platform, selected tenant and app")
          .alias("a")
          .args(vec![app_argument(), tenant_argument()]),
        Command::new(OPEN_TARGET_CONSOLE_TENANT_SERVICE)
          .about("Open the console for the platform, selected tenant and service")
          .alias("s")
          .args(vec![service_argument(), tenant_argument()]),
        Command::new(OPEN_TARGET_MONITORING_TENANT)
          .about("Open the monitoring web application for the platform and selected tenant")
          .arg(tenant_argument()),
        Command::new(OPEN_TARGET_SWAGGER).about("Open the swagger web application for the platform"),
        Command::new(OPEN_TARGET_TRACING).about("Open the tracing application for the platform"),
      ])
      .add_extra_arguments(vec![service_argument()])
  );
  static ref PLATFORM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND_PAIR, "Show platform data")
      .set_long_about("Show platform data.")
      .set_default_command_executor(&PlatformShow {})
      .add_target_argument(target_argument(PLATFORM_SUBJECT_TARGET, None))
      .add_extra_arguments(vec![app_argument(), service_argument(), tenant_argument(), vendor_argument(), vhost_argument()])
  );
  static ref PLATFORM__CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![PLATFORM_DEFAULT.as_ref(), PLATFORM_LIST_CAPABILITY.as_ref(), PLATFORM_OPEN_CAPABILITY.as_ref(), PLATFORM_SHOW_CAPABILITY.as_ref()];
}

struct PlatformDefault {}

#[async_trait]
impl CommandExecutor for PlatformDefault {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("print the default platforms specification");
    context.print(DEFAULT_PLATFORMS);
    Ok(())
  }
}

struct PLatformList {}

#[async_trait]
impl CommandExecutor for PLatformList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list platforms");
    let mut formatter = ListFormatter::new(&DSH_PLATFORM_LABELS_LIST, None, context);
    let full_names = DshPlatform::all().iter().map(|platform| platform.name().to_string()).collect::<Vec<_>>();
    formatter.push_target_ids_and_values(&full_names, DshPlatform::all());
    formatter.print()?;
    Ok(())
  }
}

struct PlatformOpen {}

#[async_trait]
impl CommandExecutor for PlatformOpen {
  async fn execute(&self, _argument: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((target, arg_matches)) => {
        let platform = context.platform.clone().ok_or("platform undefined")?;
        match target {
          OPEN_TARGET_CONSOLE => Self::open_target_console(platform, arg_matches, context),
          OPEN_TARGET_CONSOLE_TENANT_APP => Self::open_target_console_tenant_app(platform, arg_matches, context),
          OPEN_TARGET_CONSOLE_TENANT_SERVICE => Self::open_target_console_tenant_service(platform, arg_matches, context),
          OPEN_TARGET_MONITORING_TENANT => Self::open_target_monitoring_tenant(platform, arg_matches, context),
          OPEN_TARGET_SWAGGER => Self::open_target_swagger(platform, context).await,
          OPEN_TARGET_TRACING => Self::open_target_tracing(platform, context),
          _ => (),
        }
      }
      None => context.print_error("missing target argument"),
    }
    Ok(())
  }
}

impl PlatformOpen {
  fn open_target_console(platform: DshPlatform, matches: &ArgMatches, context: &Context) {
    match get_tenant_argument(matches, context) {
      Some(tenant_name) => {
        context.print_explanation(format!("open console for tenant '{}@{}'", tenant_name, platform));
        Self::open_url(platform.tenant_console_url(tenant_name), context);
      }
      None => {
        context.print_explanation(format!("open console for platform '{}'", platform));
        Self::open_url(platform.console_url(), context);
      }
    }
  }

  fn open_target_console_tenant_app(platform: DshPlatform, matches: &ArgMatches, context: &Context) {
    match get_tenant_argument_or_prompt(matches, context) {
      Ok(tenant_name) => match get_app_argument_or_prompt(matches) {
        Ok(app) => {
          context.print_explanation(format!("open console for tenant '{}@{}' and app '{}'", tenant_name, platform, app));
          Self::open_url(platform.tenant_app_console_url(tenant_name, app), context);
        }
        Err(error) => context.print_error(error),
      },
      Err(error) => context.print_error(error),
    }
  }

  fn open_target_console_tenant_service(platform: DshPlatform, matches: &ArgMatches, context: &Context) {
    match get_tenant_argument_or_prompt(matches, context) {
      Ok(tenant_name) => match get_service_argument_or_prompt(matches) {
        Ok(service) => {
          context.print_explanation(format!("open console for tenant '{}@{}' and service '{}'", tenant_name, platform, service));
          Self::open_url(platform.tenant_service_console_url(tenant_name, service), context);
        }
        Err(error) => context.print_error(error),
      },
      Err(error) => context.print_error(error),
    }
  }

  fn open_target_monitoring_tenant(platform: DshPlatform, matches: &ArgMatches, context: &Context) {
    match get_tenant_argument_or_prompt(matches, context) {
      Ok(tenant_name) => {
        context.print_explanation(format!("open monitoring application for tenant '{}@{}'", tenant_name, platform));
        Self::open_url(format!("{}/dashboards", platform.tenant_monitoring_url(tenant_name)), context);
      }
      Err(error) => context.print_error(error),
    }
  }

  async fn open_target_swagger(platform: DshPlatform, context: &Context<'_>) {
    let token = match context.dsh_api_client.as_ref() {
      Some(client) => match client.token().await {
        Ok(token) => Some(token),
        Err(_) => None,
      },
      None => None,
    };
    if let Some(token) = token {
      context.print_explanation(format!("open swagger application for platform '{}' and copy token to clipboard", platform));
      if let Some(token) = token.strip_prefix("Bearer ") {
        if Clipboard::new().and_then(|mut clipboard| clipboard.set_text(token)).is_err() {
          context.print_error("could not copy token to clipboard");
        }
      }
    } else {
      context.print_explanation(format!("open swagger application for platform '{}'", platform));
    }
    Self::open_url(platform.swagger_url(), context);
  }

  fn open_target_tracing(platform: DshPlatform, context: &Context) {
    context.print_explanation(format!("open tracing application for platform '{}'", platform));
    Self::open_url(platform.tracing_url(), context);
  }

  fn open_url(url: String, context: &Context) {
    if let Err(error) = open::that(url) {
      context.print_error(format!("could not open browser ({})", error));
    }
  }
}

struct PlatformShow {}

#[async_trait]
impl CommandExecutor for PlatformShow {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform_id = target.unwrap_or_else(|| unreachable!());
    let platform = DshPlatform::try_from(platform_id.as_str())?;
    let app = get_app_argument(matches);
    let service = get_service_argument(matches);
    let tenant = get_tenant_argument(matches, context);
    let vendor = get_vendor_argument(matches);
    let vhost = get_vhost_argument(matches);

    let labels = ALL_DSH_PLATFORM_LABELS
      .iter()
      .filter(|label| {
        let (app_required, service_required, tenant_required, vendor_required, vhost_required) = label.requirements();
        (!app_required || app.is_some())
          && (!service_required || service.is_some())
          && (!tenant_required || tenant.is_some())
          && (!vendor_required || vendor.is_some())
          && (!vhost_required || vhost.is_some())
      })
      .map(|label| label.to_owned())
      .collect::<Vec<_>>();
    UnitFormatter::new(
      platform.name(),
      labels.as_slice(),
      Some("platform name"),
      &(
        platform.clone(),
        app.unwrap_or_default(),
        service.unwrap_or_default(),
        tenant.unwrap_or_default(),
        vendor.unwrap_or_default(),
        vhost.unwrap_or_default(),
      ),
      context,
    )
    .print()?;
    Ok(())
  }
}

fn get_app_argument(matches: &ArgMatches) -> Option<String> {
  matches.get_one::<String>(APP_ARGUMENT).cloned()
}

fn get_vhost_argument(matches: &ArgMatches) -> Option<String> {
  matches.get_one::<String>(VHOST_ARGUMENT).cloned()
}

fn get_vendor_argument(matches: &ArgMatches) -> Option<String> {
  matches.get_one::<String>(VENDOR_ARGUMENT).cloned()
}

fn get_app_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(APP_ARGUMENT) {
    Some(app_argument) => Ok(app_argument.to_string()),
    None => Ok(read_single_line("enter ap: ")?),
  }
}

fn get_service_argument(matches: &ArgMatches) -> Option<String> {
  matches.get_one::<String>(SERVICE_ARGUMENT).cloned()
}

fn get_service_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(SERVICE_ARGUMENT) {
    Some(service_argument) => Ok(service_argument.to_string()),
    None => Ok(read_single_line("enter service: ")?),
  }
}

fn get_tenant_argument(matches: &ArgMatches, context: &Context) -> Option<String> {
  match matches.get_one::<String>(TENANT_ARGUMENT) {
    Some(tenant_argument) => Some(tenant_argument.to_string()),
    None => context.tenant_name.clone(),
  }
}

fn get_tenant_argument_or_prompt(matches: &ArgMatches, context: &Context) -> Result<String, String> {
  match get_tenant_argument(matches, context) {
    Some(tenant) => Ok(tenant),
    None => Ok(read_single_line("enter service: ")?),
  }
}

#[derive(Clone, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum DshPlatformLabel {
  AccessTokenEndpoint,
  Alias,
  ClientId,
  CloudProvider,
  ConsoleDomain,
  ConsoleUrl,
  Description,
  InternalServiceDomain,
  IsProduction,
  KeyCloakUrl,
  MqttTokenEndpoint,
  Name,
  PrivateDomain,
  PublicDomain,
  PublicVhostDomain,
  Realm,
  RestApiDomain,
  RestApiEndpoint,
  SwaggerUrl,
  TenantAppCatalogAppUrl,
  TenantAppCatalogUrl,
  TenantAppConsoleUrl,
  TenantClientId,
  TenantConsoleUrl,
  TenantDataCatalogUrl,
  TenantMonitoringUrl,
  TenantPrivateVhostDomain,
  TenantPublicAppDomain,
  TenantPublicAppsDomain,
  TenantServiceConsoleUrl,
  TracingUrl,
}

impl Label for DshPlatformLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AccessTokenEndpoint => "access token endpoint",
      Self::Alias => "alias",
      Self::ClientId => "client id",
      Self::CloudProvider => "cloud provider",
      Self::ConsoleDomain => "console domain",
      Self::ConsoleUrl => "console url",
      Self::Description => "description",
      Self::IsProduction => "production",
      Self::KeyCloakUrl => "key cloak url",
      Self::MqttTokenEndpoint => "mqtt token endpoint",
      Self::Name => "name",
      Self::PrivateDomain => "private domain",
      Self::PublicDomain => "public domain",
      Self::Realm => "realm",
      Self::RestApiDomain => "rest api domain",
      Self::RestApiEndpoint => "rest api endpoint",
      Self::SwaggerUrl => "swagger url",
      Self::TracingUrl => "tracing url",

      Self::InternalServiceDomain => "internal domain (service)",
      Self::PublicVhostDomain => "public vhost domain",
      Self::TenantAppCatalogAppUrl => "app catalog url (tenant, app)",
      Self::TenantAppCatalogUrl => "app catalog url (tenant)",
      Self::TenantAppConsoleUrl => "console url (tenant, app)",
      Self::TenantClientId => "client id (tenant)",
      Self::TenantConsoleUrl => "console url (tenant)",
      Self::TenantDataCatalogUrl => "data catalog url (tenant)",
      Self::TenantMonitoringUrl => "monitoring url (tenant)",
      Self::TenantPrivateVhostDomain => "private domain (tenant, vhost)",
      Self::TenantPublicAppDomain => "public domain (tenant, app)",
      Self::TenantPublicAppsDomain => "public apps domain (tenant)",
      Self::TenantServiceConsoleUrl => "console url (tenant, service)",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Name)
  }
}

// Subject formatter for DshPlatform only
impl SubjectFormatter<DshPlatformLabel> for DshPlatform {
  fn value(&self, label: &DshPlatformLabel, _target_id: &str) -> String {
    match label {
      DshPlatformLabel::AccessTokenEndpoint => self.access_token_endpoint(),
      DshPlatformLabel::Alias => self.alias().to_string(),
      DshPlatformLabel::ClientId => self.client_id(),
      DshPlatformLabel::CloudProvider => self.cloud_provider().to_string(),
      DshPlatformLabel::ConsoleDomain => self.console_domain(),
      DshPlatformLabel::ConsoleUrl => self.console_url(),
      DshPlatformLabel::Description => self.description().to_string(),
      DshPlatformLabel::IsProduction => self.is_production().to_string(),
      DshPlatformLabel::KeyCloakUrl => self.key_cloak_url().to_string(),
      DshPlatformLabel::MqttTokenEndpoint => self.mqtt_token_endpoint(),
      DshPlatformLabel::Name => self.name().to_string(),
      DshPlatformLabel::PrivateDomain => self.private_domain().unwrap_or("not configured").to_string(),
      DshPlatformLabel::PublicDomain => self.public_domain().to_string(),
      DshPlatformLabel::Realm => self.realm().to_string(),
      DshPlatformLabel::RestApiDomain => self.rest_api_domain(),
      DshPlatformLabel::RestApiEndpoint => self.rest_api_endpoint(),
      DshPlatformLabel::SwaggerUrl => self.swagger_url(),
      DshPlatformLabel::TracingUrl => self.tracing_url(),
      _ => unreachable!(),
    }
  }

  fn target_label(&self) -> Option<DshPlatformLabel> {
    Some(DshPlatformLabel::Name)
  }
}

// Subject formatter for (DshPlatform/app/service/tenant/vendor/vhost) sextets
impl SubjectFormatter<DshPlatformLabel> for (DshPlatform, String, String, String, String, String) {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> String {
    let (platform, app, service, tenant, vendor, vhost) = self;
    match label {
      DshPlatformLabel::InternalServiceDomain => platform.internal_service_domain(service),
      DshPlatformLabel::PublicVhostDomain => platform.public_vhost_domain(vhost),
      DshPlatformLabel::TenantAppCatalogAppUrl => platform.tenant_app_catalog_app_url(tenant, vendor, app),
      DshPlatformLabel::TenantAppCatalogUrl => platform.tenant_app_catalog_url(tenant),
      DshPlatformLabel::TenantAppConsoleUrl => platform.tenant_app_console_url(tenant, app),
      DshPlatformLabel::TenantClientId => platform.tenant_client_id(tenant),
      DshPlatformLabel::TenantConsoleUrl => platform.tenant_console_url(tenant),
      DshPlatformLabel::TenantDataCatalogUrl => platform.tenant_data_catalog_url(tenant),
      DshPlatformLabel::TenantMonitoringUrl => platform.tenant_monitoring_url(tenant),
      DshPlatformLabel::TenantPrivateVhostDomain => platform
        .tenant_private_vhost_domain(tenant, vhost)
        .unwrap_or("private domain not configured".to_string()),
      DshPlatformLabel::TenantPublicAppDomain => platform.tenant_public_app_domain(tenant, app),
      DshPlatformLabel::TenantPublicAppsDomain => platform.tenant_public_apps_domain(tenant),
      DshPlatformLabel::TenantServiceConsoleUrl => platform.tenant_service_console_url(tenant, service),
      _ => platform.value(label, target_id),
    }
  }

  fn target_label(&self) -> Option<DshPlatformLabel> {
    Some(DshPlatformLabel::Name)
  }
}

pub static ALL_DSH_PLATFORM_LABELS: [DshPlatformLabel; 31] = [
  // Items from platform configuration file
  DshPlatformLabel::Name,
  DshPlatformLabel::Description,
  DshPlatformLabel::Alias,
  DshPlatformLabel::IsProduction,
  DshPlatformLabel::CloudProvider,
  DshPlatformLabel::KeyCloakUrl,
  DshPlatformLabel::Realm,
  DshPlatformLabel::PublicDomain,
  DshPlatformLabel::PrivateDomain,
  // Derived items that do not depend on tenant et cetera
  DshPlatformLabel::ConsoleDomain,
  DshPlatformLabel::ConsoleUrl,
  DshPlatformLabel::ClientId,
  DshPlatformLabel::RestApiDomain,
  DshPlatformLabel::AccessTokenEndpoint,
  DshPlatformLabel::MqttTokenEndpoint,
  DshPlatformLabel::RestApiEndpoint,
  DshPlatformLabel::SwaggerUrl,
  DshPlatformLabel::TracingUrl,
  // Derived itens that do depend on tenant et cetera
  DshPlatformLabel::PublicVhostDomain,
  DshPlatformLabel::TenantPublicAppsDomain,
  DshPlatformLabel::TenantPublicAppDomain,
  DshPlatformLabel::TenantConsoleUrl,
  DshPlatformLabel::TenantAppCatalogUrl,
  DshPlatformLabel::TenantAppCatalogAppUrl,
  DshPlatformLabel::TenantAppConsoleUrl,
  DshPlatformLabel::TenantServiceConsoleUrl,
  DshPlatformLabel::TenantDataCatalogUrl,
  DshPlatformLabel::TenantMonitoringUrl,
  DshPlatformLabel::TenantClientId,
  DshPlatformLabel::TenantPrivateVhostDomain,
  DshPlatformLabel::InternalServiceDomain,
];

pub static DSH_PLATFORM_LABELS_LIST: [DshPlatformLabel; 5] =
  [DshPlatformLabel::Name, DshPlatformLabel::Alias, DshPlatformLabel::IsProduction, DshPlatformLabel::Description, DshPlatformLabel::ConsoleUrl];

// Returns the required parameters
// (app_required, service_required, tenant_required, vendor_required, vhost_required)
impl DshPlatformLabel {
  fn requirements(&self) -> (bool, bool, bool, bool, bool) {
    match self {
      DshPlatformLabel::TenantAppCatalogAppUrl => (true, false, true, true, false),
      DshPlatformLabel::TenantAppConsoleUrl | DshPlatformLabel::TenantPublicAppDomain => (true, false, true, false, false),
      DshPlatformLabel::TenantServiceConsoleUrl => (false, true, true, false, false),
      DshPlatformLabel::InternalServiceDomain => (false, true, false, false, false),
      DshPlatformLabel::TenantPrivateVhostDomain => (false, false, true, false, true),
      DshPlatformLabel::TenantAppCatalogUrl
      | DshPlatformLabel::TenantClientId
      | DshPlatformLabel::TenantConsoleUrl
      | DshPlatformLabel::TenantDataCatalogUrl
      | DshPlatformLabel::TenantMonitoringUrl
      | DshPlatformLabel::TenantPublicAppsDomain => (false, false, true, false, false),
      DshPlatformLabel::PublicVhostDomain => (false, false, false, false, true),
      _ => (false, false, false, false, false),
    }
  }
}
