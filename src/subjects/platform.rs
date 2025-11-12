use crate::arguments::{
  app_id_argument, bucket_id_argument, platform_name_argument, service_id_argument, vendor_name_argument, vhost_id_argument, APP_ID_ARGUMENT, BUCKET_ID_ARGUMENT,
  PLATFORM_NAME_ARGUMENT, SERVICE_ID_ARGUMENT, VENDOR_NAME_ARGUMENT, VHOST_ID_ARGUMENT,
};
use crate::capability::{Capability, CommandExecutor, EXPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::{get_target_platform, get_target_tenant, get_target_tenant_non_interactive, read_single_line, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::platform::DshPlatform;
use dsh_api::DEFAULT_PLATFORMS;
use itertools::Itertools;
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
    CapabilityBuilder::new(EXPORT_COMMAND, None, &PlatformExport {}, "Export default platform configuration").set_long_about(
      "Export the default platform configuration json file from the dsh-api library. \
        This file can be used as a starting point when platform customization is required."
    )
  );
  static ref PLATFORM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &PLatformList {}, "List platforms").set_long_about("Lists all dsh platforms."));
  static ref PLATFORM_OPEN_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
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
      ])
  );
  static ref PLATFORM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &PlatformShow {}, "Show platform data")
      .set_long_about("Show platform data.")
      .add_target_argument(platform_name_argument())
      .add_extra_arguments(vec![
        app_id_argument().long("app"),
        bucket_id_argument().long("bucket"),
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
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("export the default platforms specification");
    context.print(DEFAULT_PLATFORMS);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct PLatformList {}

#[async_trait]
impl CommandExecutor for PLatformList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list platforms");
    let mut formatter = ListFormatter::new(&DSH_PLATFORM_LABELS_LIST, None, context);
    let full_names = DshPlatform::all().iter().map(|platform| platform.name().to_string()).collect_vec();
    formatter.push_target_ids_and_values(&full_names, DshPlatform::all());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct PlatformOpen {}

#[async_trait]
impl CommandExecutor for PlatformOpen {
  async fn execute_without_client(&self, _argument: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((target, arg_matches)) => match target {
        OPEN_APP => Self::open_app(arg_matches, context),
        OPEN_CONSOLE => Self::open_console(arg_matches, context),
        OPEN_MONITORING => Self::open_monitoring(arg_matches, context),
        OPEN_SERVICE => Self::open_service(arg_matches, context),
        OPEN_TENANT => Self::open_tenant(arg_matches, context),
        OPEN_TRACING => Self::open_tracing(arg_matches, context),
        _ => unreachable!(),
      },
      None => Err("missing target argument".to_string()),
    }
  }

  async fn execute_with_client(&self, _argument: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((target, arg_matches)) => match target {
        OPEN_SWAGGER => Self::open_swagger(arg_matches, client, context).await,
        _ => unreachable!(),
      },
      None => Err("missing target argument".to_string()),
    }
  }

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(matches!(sub_matches.subcommand().unwrap_or_else(|| unreachable!()).0, OPEN_SWAGGER))
  }
}

impl PlatformOpen {
  fn open_app(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let app = get_app_argument_or_prompt(matches)?;
    context.open_url(
      platform.tenant_app_console_url(&tenant_name, &app),
      format!("console for tenant '{}@{}' and app '{}'", tenant_name, platform, app),
    )
  }

  fn open_console(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    context.open_url(platform.console_url(), format!("console for platform '{}'", platform))
  }

  fn open_monitoring(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    context.open_url(
      format!("{}/dashboards", platform.tenant_monitoring_url(&tenant_name)),
      format!("monitoring application for tenant '{}@{}'", tenant_name, platform),
    )
  }

  fn open_service(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let service = get_service_argument_or_prompt(matches)?;
    context.open_url(
      platform.tenant_service_console_url(&tenant_name, &service),
      format!("console for tenant '{}@{}' and service '{}'", tenant_name, platform, service),
    )
  }

  fn open_tenant(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    context.open_url(
      platform.tenant_console_url(&tenant_name),
      format!("console for tenant '{}@{}'", tenant_name, platform),
    )
  }

  async fn open_swagger(matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    let token = match client.token().await {
      Ok(token) => {
        debug!("token fetched");
        Some(token)
      }
      Err(_) => {
        context.print_warning("token could not be fetched");
        None
      }
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
    context.open_url(platform.swagger_url(), opening_target)
  }

  fn open_tracing(matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    context.open_url(platform.tracing_url(), format!("tracing application for platform '{}'", platform))
  }
}

struct PlatformShow {}

#[async_trait]
impl CommandExecutor for PlatformShow {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let platform = match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
      Some(platform_name_from_argument) => DshPlatform::try_from(platform_name_from_argument.as_str())?,
      None => get_target_platform(matches, context.settings())?,
    };
    let tenant = get_target_tenant_non_interactive(matches, context.settings())?;
    let app_id = matches.get_one::<String>(APP_ID_ARGUMENT).cloned();
    let bucket_id = matches.get_one::<String>(BUCKET_ID_ARGUMENT).cloned();
    let service_id = matches.get_one::<String>(SERVICE_ID_ARGUMENT).cloned();
    let vendor_id = matches.get_one::<String>(VENDOR_NAME_ARGUMENT).cloned();
    let vhost = matches.get_one::<String>(VHOST_ID_ARGUMENT).cloned();

    context.print_explanation(
      [
        Some(format!("list all parameters for platform '{}'", platform)),
        tenant.as_ref().map(|tenant| format!("tenant '{}'", tenant)),
        app_id.as_ref().map(|app_id| format!("app '{}'", app_id)),
        bucket_id.as_ref().map(|bucket_id| format!("bucket '{}'", bucket_id)),
        service_id.as_ref().map(|service_id| format!("service '{}'", service_id)),
        vendor_id.as_ref().map(|vendor_id| format!("vendor '{}'", vendor_id)),
        vhost.as_ref().map(|vhost| format!("vhost '{}'", vhost)),
      ]
      .iter()
      .flatten()
      .join(", "),
    );

    let labels = ALL_DSH_PLATFORM_LABELS
      .iter()
      .filter(|label| {
        let (app_id_required, bucket_id_required, service_id_required, tenant_required, vendor_id_required, vhost_required) = label.requirements();
        (!app_id_required || app_id.is_some())
          && (!bucket_id_required || bucket_id.is_some())
          && (!service_id_required || service_id.is_some())
          && (!tenant_required || tenant.is_some())
          && (!vendor_id_required || vendor_id.is_some())
          && (!vhost_required || vhost.is_some())
      })
      .map(|label| label.to_owned())
      .collect_vec();
    UnitFormatter::new(platform.name(), labels.as_slice(), Some("platform name"), context).print(
      &(
        platform.clone(),
        app_id.unwrap_or_default(),
        bucket_id.unwrap_or_default(),
        service_id.unwrap_or_default(),
        tenant.unwrap_or_default(),
        vendor_id.unwrap_or_default(),
        vhost.unwrap_or_default(),
      ),
      None,
    )
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

fn get_app_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(APP_ID_ARGUMENT) {
    Some(app_argument) => Ok(app_argument.to_string()),
    None => Ok(read_single_line("enter app: ")?),
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
  BucketName,
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
      Self::BucketName => "bucket name",
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

// Subject formatter for (DshPlatform, app, bucket, service, tenant, vendor, vhost) septets
impl SubjectFormatter<DshPlatformLabel> for (DshPlatform, String, String, String, String, String, String) {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> String {
    let (platform, app_id, bucket_id, service_id, tenant, vendor_id, vhost) = self;
    match label {
      DshPlatformLabel::BucketName => platform.bucket_name(tenant, bucket_id, Some("ACCESS_KEY_ID")).unwrap_or_else(|error| error),
      DshPlatformLabel::InternalDomain => platform.internal_domain(tenant),
      DshPlatformLabel::InternalServiceDomain => platform.internal_service_domain(tenant, service_id),
      DshPlatformLabel::PublicVhostDomain => platform.public_vhost_domain(vhost),
      DshPlatformLabel::TenantAppCatalogAppUrl => platform.tenant_app_catalog_app_url(tenant, vendor_id, app_id),
      DshPlatformLabel::TenantAppCatalogUrl => platform.tenant_app_catalog_url(tenant),
      DshPlatformLabel::TenantAppConsoleUrl => platform.tenant_app_console_url(tenant, app_id),
      DshPlatformLabel::TenantClientId => platform.tenant_client_id(tenant),
      DshPlatformLabel::TenantConsoleUrl => platform.tenant_console_url(tenant),
      DshPlatformLabel::TenantDataCatalogUrl => platform.tenant_data_catalog_url(tenant),
      DshPlatformLabel::TenantMonitoringUrl => platform.tenant_monitoring_url(tenant),
      DshPlatformLabel::TenantPrivateVhostDomain => platform
        .tenant_private_vhost_domain(tenant, vhost)
        .unwrap_or("private domain not configured".to_string()),
      DshPlatformLabel::TenantPublicAppDomain => platform.tenant_public_app_domain(tenant, app_id),
      DshPlatformLabel::TenantPublicAppsDomain => platform.tenant_public_domain(tenant),
      DshPlatformLabel::TenantServiceConsoleUrl => platform.tenant_service_console_url(tenant, service_id),
      _ => platform.value(label, target_id),
    }
  }
}

pub static ALL_DSH_PLATFORM_LABELS: [DshPlatformLabel; 32] = [
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
  // Derived items that do depend on tenant et cetera
  DshPlatformLabel::BucketName,
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
// (app_id_required, bucket_id_required, service_id_required, tenant_required, vendor_id_required, vhost_required)
impl DshPlatformLabel {
  fn requirements(&self) -> (bool, bool, bool, bool, bool, bool) {
    match self {
      DshPlatformLabel::BucketName => (false, true, false, true, false, false),
      DshPlatformLabel::TenantAppCatalogAppUrl => (true, false, false, true, true, false),
      DshPlatformLabel::TenantAppConsoleUrl | DshPlatformLabel::TenantPublicAppDomain => (true, false, false, true, false, false),
      DshPlatformLabel::TenantServiceConsoleUrl | DshPlatformLabel::InternalServiceDomain => (false, false, true, true, false, false),
      DshPlatformLabel::InternalDomain
      | DshPlatformLabel::TenantAppCatalogUrl
      | DshPlatformLabel::TenantClientId
      | DshPlatformLabel::TenantConsoleUrl
      | DshPlatformLabel::TenantDataCatalogUrl
      | DshPlatformLabel::TenantMonitoringUrl
      | DshPlatformLabel::TenantPublicAppsDomain => (false, false, false, true, false, false),
      DshPlatformLabel::TenantPrivateVhostDomain => (false, false, false, true, false, true),
      DshPlatformLabel::PublicVhostDomain => (false, false, false, false, false, true),
      _ => (false, false, false, false, false, false),
    }
  }
}
