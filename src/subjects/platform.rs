use crate::arguments::{
  app_id_argument, platform_name_argument, service_id_argument, vendor_name_argument, vhost_id_argument, APP_ID_ARGUMENT, PLATFORM_NAME_ARGUMENT, SERVICE_ID_ARGUMENT,
  VENDOR_NAME_ARGUMENT, VHOST_ID_ARGUMENT,
};
use crate::capability::{Capability, CommandExecutor, EXPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
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
use log::{debug, warn};
use serde::Serialize;

pub(crate) struct PlatformSubject {}

const PLATFORM_SUBJECT_TARGET: &str = "platform";

const OPEN_APP: &str = "app";
const OPEN_CONSOLE: &str = "console";
const OPEN_MONITORING: &str = "monitoring";
const OPEN_SERVICE: &str = "service";
const OPEN_SWAGGER: &str = "swagger";
const OPEN_TENANT: &str = "tenant";
const OPEN_TRACING: &str = "tracing";

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

lazy_static! {
  static ref PLATFORM_EXPORT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(EXPORT_COMMAND, None, "Export default platform configuration")
      .set_long_about(
        "Export the default platform configuration json file from the dsh-api library. \
        This file can be used as a starting point when platform customization is required."
      )
      .set_default_command_executor(&PlatformExport {})
  );
  static ref PLATFORM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List platforms")
      .set_long_about("Lists all dsh platforms.")
      .set_default_command_executor(&PLatformList {}),
  );
  static ref PLATFORM_OPEN_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(OPEN_COMMAND, Some(OPEN_COMMAND_ALIAS), "Open console or web application")
      .set_long_about("Open the DSH console, monitoring page or the web application for the tenant or a service.")
      .set_default_command_executor(&PlatformOpen {})
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
      ])
  );
  static ref PLATFORM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), "Show platform data")
      .set_long_about("Show platform data.")
      .set_default_command_executor(&PlatformShow {})
      .add_target_argument(platform_name_argument())
      .add_extra_arguments(vec![
        app_id_argument().long("app"),
        service_id_argument().long("service"),
        vendor_name_argument().long("vendor"),
        vhost_id_argument().long("vhost")
      ])
  );
  static ref PLATFORM__CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![PLATFORM_EXPORT_CAPABILITY.as_ref(), PLATFORM_LIST_CAPABILITY.as_ref(), PLATFORM_OPEN_CAPABILITY.as_ref(), PLATFORM_SHOW_CAPABILITY.as_ref()];
}

struct PlatformExport {}

#[async_trait]
impl CommandExecutor for PlatformExport {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("export the default platforms specification");
    context.print(DEFAULT_PLATFORMS);
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_without_api(None)
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

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_without_api(None)
  }
}

struct PlatformOpen {}

#[async_trait]
impl CommandExecutor for PlatformOpen {
  async fn execute(&self, _argument: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((target, arg_matches)) => match target {
        OPEN_APP => Self::open_app(arg_matches, context),
        OPEN_CONSOLE => Self::open_console(context),
        OPEN_MONITORING => Self::open_monitoring(context),
        OPEN_SERVICE => Self::open_service(arg_matches, context),
        OPEN_SWAGGER => Self::open_swagger(context).await,
        OPEN_TENANT => Self::open_tenant(context),
        OPEN_TRACING => Self::open_tracing(context),
        _ => unreachable!(),
      },
      None => Err("missing target argument".to_string()),
    }
  }

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(
      true,
      matches!(
        sub_matches.subcommand().unwrap_or_else(|| unreachable!()),
        (OPEN_APP, _) | (OPEN_MONITORING, _) | (OPEN_SERVICE, _) | (OPEN_TENANT, _)
      ),
      matches!(sub_matches.subcommand().unwrap_or_else(|| unreachable!()).0, OPEN_SWAGGER),
      None,
    )
  }
}

impl PlatformOpen {
  fn open_app(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    let tenant_name = context.target_tenant_name.clone().unwrap();
    let app = get_app_argument_or_prompt(matches)?;
    Self::open_url(
      platform.tenant_app_console_url(&tenant_name, &app),
      format!("console for tenant '{}@{}' and app '{}'", tenant_name, platform, app),
      context,
    )
  }

  fn open_console(context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    Self::open_url(platform.console_url(), format!("console for platform '{}'", platform), context)
  }

  fn open_monitoring(context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    let tenant_name = context.target_tenant_name.clone().unwrap();
    Self::open_url(
      format!("{}/dashboards", platform.tenant_monitoring_url(&tenant_name)),
      format!("monitoring application for tenant '{}@{}'", tenant_name, platform),
      context,
    )
  }

  fn open_service(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    let tenant_name = context.target_tenant_name.clone().unwrap();
    let service = get_service_argument_or_prompt(matches)?;
    Self::open_url(
      platform.tenant_service_console_url(&tenant_name, &service),
      format!("console for tenant '{}@{}' and service '{}'", tenant_name, platform, service),
      context,
    )
  }

  fn open_tenant(context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    let tenant_name = context.target_tenant_name.clone().unwrap();
    Self::open_url(
      platform.tenant_console_url(&tenant_name),
      format!("console for tenant '{}@{}'", tenant_name, platform),
      context,
    )
  }

  async fn open_swagger(context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    let token = match context.client() {
      Some(client) => match client.token().await {
        Ok(token) => {
          debug!("token fetched");
          Some(token)
        }
        Err(_) => {
          context.print_warning("token could not be fetched");
          None
        }
      },
      None => None,
    };
    let opening_target = match token {
      Some(token) => match token.strip_prefix("Bearer ") {
        Some(token) => match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(token)) {
          Ok(_) => {
            debug!("token copied to clipboard");
            format!("swagger application for platform '{}' (token on clipboard)", platform)
          }
          Err(_) => {
            warn!("could not copy token to clipboard");
            format!("swagger application for platform '{}'", platform)
          }
        },
        None => return Err("token has incorrect format".to_string()),
      },
      None => format!("swagger application for platform '{}'", platform),
    };
    Self::open_url(platform.swagger_url(), opening_target, context)
  }

  fn open_tracing(context: &Context) -> DshCliResult {
    let platform = context.target_platform.clone().unwrap();
    Self::open_url(platform.tracing_url(), format!("tracing application for platform '{}'", platform), context)
  }

  fn open_url(url: String, opening_target: String, context: &Context) -> DshCliResult {
    if context.dry_run {
      debug!("url (dry-run enabled) '{}'", url);
      context.print_warning(format!("dry-run mode, opening {} canceled", opening_target));
      Ok(())
    } else {
      debug!("open url '{}'", url);
      context.print_explanation(format!("open {}", opening_target));
      open::that(url).map_err(|error| format!("could not open browser ({})", error))
    }
  }
}

struct PlatformShow {}

#[async_trait]
impl CommandExecutor for PlatformShow {
  async fn execute(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
      Some(platform_name_from_argument) => DshPlatform::try_from(platform_name_from_argument.as_str())?,
      None => context.target_platform.clone().unwrap(),
    };
    let tenant = context.target_tenant_name.clone();
    let app = matches.get_one::<String>(APP_ID_ARGUMENT).cloned();
    let service = matches.get_one::<String>(SERVICE_ID_ARGUMENT).cloned();
    let vendor = matches.get_one::<String>(VENDOR_NAME_ARGUMENT).cloned();
    let vhost = matches.get_one::<String>(VHOST_ID_ARGUMENT).cloned();
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
    UnitFormatter::new(platform.name(), labels.as_slice(), Some("platform name"), context).print(&(
      platform.clone(),
      app.unwrap_or_default(),
      service.unwrap_or_default(),
      tenant.unwrap_or_default(),
      vendor.unwrap_or_default(),
      vhost.unwrap_or_default(),
    ))
  }

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(sub_matches.get_one::<String>(PLATFORM_NAME_ARGUMENT).is_none(), false, false, None)
  }
}

fn get_app_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(APP_ID_ARGUMENT) {
    Some(app_argument) => Ok(app_argument.to_string()),
    None => Ok(read_single_line("enter ap: ")?),
  }
}

fn get_service_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(SERVICE_ID_ARGUMENT) {
    Some(service_argument) => Ok(service_argument.to_string()),
    None => Ok(read_single_line("enter service: ")?),
  }
}

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Debug)]
pub(crate) enum DshPlatformLabel {
  AccessTokenEndpoint,
  Alias,
  ClientId,
  CloudProvider,
  ConsoleDomain,
  ConsoleUrl,
  Description,
  InternalDomain,
  InternalServiceDomain,
  IsProduction,
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
      Self::MqttTokenEndpoint => "mqtt token endpoint",
      Self::Name => "name",
      Self::PrivateDomain => "private domain",
      Self::PublicDomain => "public domain",
      Self::Realm => "realm",
      Self::RestApiDomain => "rest api domain",
      Self::RestApiEndpoint => "rest api endpoint",
      Self::SwaggerUrl => "swagger url",
      Self::TracingUrl => "tracing url",

      Self::InternalDomain => "internal domain",
      Self::InternalServiceDomain => "internal domain (service)",
      Self::PublicVhostDomain => "public vhost domain",
      Self::TenantAppCatalogAppUrl => "app catalog url (app/tenant)",
      Self::TenantAppCatalogUrl => "app catalog url (tenant)",
      Self::TenantAppConsoleUrl => "console url (app/tenant)",
      Self::TenantClientId => "client id (tenant)",
      Self::TenantConsoleUrl => "console url (tenant)",
      Self::TenantDataCatalogUrl => "data catalog url (tenant)",
      Self::TenantMonitoringUrl => "monitoring url (tenant)",
      Self::TenantPrivateVhostDomain => "private domain (tenant/vhost)",
      Self::TenantPublicAppDomain => "public domain (app/tenant)",
      Self::TenantPublicAppsDomain => "public apps domain (tenant)",
      Self::TenantServiceConsoleUrl => "console url (service/tenant)",
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
      DshPlatformLabel::AccessTokenEndpoint => self.access_token_endpoint().to_string(),
      DshPlatformLabel::Alias => self.alias().to_string(),
      DshPlatformLabel::ClientId => self.client_id(),
      DshPlatformLabel::CloudProvider => self.cloud_provider().to_string(),
      DshPlatformLabel::ConsoleDomain => self.console_domain(),
      DshPlatformLabel::ConsoleUrl => self.console_url(),
      DshPlatformLabel::Description => self.description().to_string(),
      DshPlatformLabel::IsProduction => self.is_production().to_string(),
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
}

// Subject formatter for (DshPlatform/app/service/tenant/vendor/vhost) sextets
impl SubjectFormatter<DshPlatformLabel> for (DshPlatform, String, String, String, String, String) {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> String {
    let (platform, app, service, tenant, vendor, vhost) = self;
    match label {
      DshPlatformLabel::InternalDomain => platform.internal_domain(tenant),
      DshPlatformLabel::InternalServiceDomain => platform.internal_service_domain(tenant, service),
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
}

pub static ALL_DSH_PLATFORM_LABELS: [DshPlatformLabel; 31] = [
  // Items from platform configuration file
  DshPlatformLabel::Name,
  DshPlatformLabel::Description,
  DshPlatformLabel::Alias,
  DshPlatformLabel::IsProduction,
  DshPlatformLabel::CloudProvider,
  DshPlatformLabel::Realm,
  DshPlatformLabel::AccessTokenEndpoint,
  DshPlatformLabel::PublicDomain,
  DshPlatformLabel::PrivateDomain,
  // Derived items that do not depend on tenant et cetera
  DshPlatformLabel::ConsoleDomain,
  DshPlatformLabel::ConsoleUrl,
  DshPlatformLabel::ClientId,
  DshPlatformLabel::RestApiDomain,
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
  DshPlatformLabel::InternalDomain,
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
      DshPlatformLabel::TenantServiceConsoleUrl | DshPlatformLabel::InternalServiceDomain => (false, true, true, false, false),
      DshPlatformLabel::InternalDomain
      | DshPlatformLabel::TenantAppCatalogUrl
      | DshPlatformLabel::TenantClientId
      | DshPlatformLabel::TenantConsoleUrl
      | DshPlatformLabel::TenantDataCatalogUrl
      | DshPlatformLabel::TenantMonitoringUrl
      | DshPlatformLabel::TenantPublicAppsDomain => (false, false, true, false, false),
      DshPlatformLabel::TenantPrivateVhostDomain => (false, false, true, false, true),
      DshPlatformLabel::PublicVhostDomain => (false, false, false, false, true),
      _ => (false, false, false, false, false),
    }
  }
}
