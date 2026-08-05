use crate::arguments::{APP_ID_ARGUMENT, BUCKET_ID_ARGUMENT, PROXY_ID_ARGUMENT, SERVICE_ID_ARGUMENT, TOPIC_ID_ARGUMENT, VENDOR_NAME_ARGUMENT, VHOST_SUBDOMAIN_ARGUMENT};
use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::error::DshCliError;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::vec_to_table;
use crate::subject::Requirements;
use crate::subjects::platform::labels::DshPlatformLabel;
use crate::target_platform::get_target_platform;
use crate::target_tenant::{get_target_tenant, get_target_tenant_non_interactive};
use crate::{err, read_single_line, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::platform::DshPlatform;
use dsh_api::DEFAULT_PLATFORMS;
use itertools::Itertools;
use log::{debug, warn};

pub(crate) struct PlatformExport {}

#[async_trait]
impl CommandExecutor for PlatformExport {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("export the default platforms specification");
    context.println(DEFAULT_PLATFORMS);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

static DSH_PLATFORM_LABELS_LIST: [DshPlatformLabel; 7] = [
  DshPlatformLabel::Parameter,
  DshPlatformLabel::Alias,
  DshPlatformLabel::Realm,
  DshPlatformLabel::IsProduction,
  DshPlatformLabel::Description,
  DshPlatformLabel::PublicDomain,
  DshPlatformLabel::PrivateDomain,
];

pub(crate) struct PLatformList {}

#[async_trait]
impl CommandExecutor for PLatformList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list platforms");
    let mut formatter = ListFormatter::new_override_target_id_label(&DSH_PLATFORM_LABELS_LIST, "platform id", context);
    let full_names = DshPlatform::all().iter().map(|platform| platform.name().to_string()).collect_vec();
    formatter.push_target_ids_and_values(&full_names, DshPlatform::all());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

const OPEN_APP: &str = "app";
const OPEN_CONSOLE: &str = "console";
const OPEN_MONITORING: &str = "monitoring";
const OPEN_SERVICE: &str = "service";
const OPEN_SWAGGER: &str = "swagger";
const OPEN_TENANT: &str = "tenant";
const OPEN_TRACING: &str = "tracing";

pub(crate) struct PlatformOpen {}

#[async_trait]
impl CommandExecutor for PlatformOpen {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    match matches.subcommand() {
      Some((target, arg_matches)) => {
        let platform = get_target_platform(matches, context.settings())?;
        match target {
          OPEN_APP => Self::open_app(&platform, arg_matches, context),
          OPEN_CONSOLE => Self::open_console(&platform, context),
          OPEN_MONITORING => Self::open_monitoring(&platform, arg_matches, context),
          OPEN_SERVICE => Self::open_service(&platform, arg_matches, context),
          OPEN_TENANT => Self::open_tenant(&platform, arg_matches, context),
          OPEN_TRACING => Self::open_tracing(&platform, context),
          _ => unreachable!(),
        }
      }
      None => err!("missing target argument"),
    }
  }

  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    match matches.subcommand() {
      Some((target, _)) => match target {
        OPEN_SWAGGER => {
          let platform = get_target_platform(matches, context.settings())?;
          Self::open_swagger(&platform, client, context).await
        }
        _ => unreachable!(),
      },
      None => err!("missing target argument"),
    }
  }

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(true, None, matches!(sub_matches.subcommand().unwrap_or_else(|| unreachable!()).0, OPEN_SWAGGER))
  }
}

impl PlatformOpen {
  fn open_app(platform: &DshPlatform, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let app = get_app_argument_or_prompt(matches)?;
    context.open_url(
      platform.tenant_app_console_url(&tenant_name, &app),
      format!("console for tenant '{}@{}' and app '{}'", tenant_name, platform, app),
    );
    Ok(())
  }

  fn open_console(platform: &DshPlatform, context: &Context) -> DshCliResult<()> {
    context.open_url(platform.console_url(), format!("console for platform '{}'", platform));
    Ok(())
  }

  fn open_monitoring(platform: &DshPlatform, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let tenant_name = get_target_tenant(matches, context.settings())?;
    context.open_url(
      format!("{}/dashboards", platform.tenant_monitoring_url(&tenant_name)),
      format!("monitoring application for tenant '{}@{}'", tenant_name, platform),
    );
    Ok(())
  }

  fn open_service(platform: &DshPlatform, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let service = get_service_argument_or_prompt(matches)?;
    context.open_url(
      platform.tenant_service_console_url(&tenant_name, &service),
      format!("console for tenant '{}@{}' and service '{}'", tenant_name, platform, service),
    );
    Ok(())
  }

  fn open_tenant(platform: &DshPlatform, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let tenant_name = get_target_tenant(matches, context.settings())?;
    context.open_url(
      platform.tenant_console_url(&tenant_name),
      format!("console for tenant '{}@{}'", tenant_name, platform),
    );
    Ok(())
  }

  async fn open_swagger(platform: &DshPlatform, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let bearer_token = match client.raw_token().await {
      Ok(token) => {
        debug!("token fetched");
        Some(token)
      }
      Err(_) => {
        context.print_warning("token could not be fetched");
        None
      }
    };
    let opening_target = match bearer_token {
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
      None => format!("swagger application for platform '{}'", platform),
    };
    context.open_url(platform.swagger_url(), opening_target);
    Ok(())
  }

  fn open_tracing(platform: &DshPlatform, context: &Context) -> DshCliResult<()> {
    context.open_url(platform.tracing_url(), format!("tracing application for platform '{}'", platform));
    Ok(())
  }
}

static DSH_PLATFORM_LABELS_CONFIGURATION: [DshPlatformLabel; 10] = [
  DshPlatformLabel::Parameter,
  DshPlatformLabel::Alias,
  DshPlatformLabel::Description,
  DshPlatformLabel::IsProduction,
  DshPlatformLabel::CloudProvider,
  DshPlatformLabel::Region,
  DshPlatformLabel::Realm,
  DshPlatformLabel::IssuerEndpoint,
  DshPlatformLabel::PrivateDomain,
  DshPlatformLabel::PublicDomain,
];

static DSH_PLATFORM_LABELS_DERIVED: [DshPlatformLabel; 13] = [
  DshPlatformLabel::Parameter,
  DshPlatformLabel::RestApiDomain,
  DshPlatformLabel::RestApiEndpoint,
  DshPlatformLabel::RestTokenEndpoint,
  DshPlatformLabel::ConsoleDomain,
  DshPlatformLabel::ConsoleUrl,
  DshPlatformLabel::SwaggerUrl,
  DshPlatformLabel::TracingUrl,
  DshPlatformLabel::AccessTokenEndpoint,
  DshPlatformLabel::RobotClientId,
  DshPlatformLabel::MqttMessagingApiPort,
  DshPlatformLabel::MqttTokenEndpoint,
  DshPlatformLabel::MqttMessagingApiEndpoint,
];

static DSH_PLATFORM_LABELS_DERIVED_TENANT: [DshPlatformLabel; 9] = [
  DshPlatformLabel::Parameter,
  DshPlatformLabel::InternalDomain,
  DshPlatformLabel::ProxyVhostDomain,
  DshPlatformLabel::RobotTenantClientId,
  DshPlatformLabel::TenantAppCatalogUrl,
  DshPlatformLabel::TenantConsoleUrl,
  DshPlatformLabel::TenantDataCatalogUrl,
  DshPlatformLabel::TenantMonitoringUrl,
  DshPlatformLabel::TenantPublicAppsDomain,
];

static DSH_PLATFORM_LABELS_DERIVED_ARGUMENTS: [DshPlatformLabel; 21] = [
  DshPlatformLabel::Parameter,
  DshPlatformLabel::BucketName,
  DshPlatformLabel::InternalServiceDomain,
  DshPlatformLabel::HttpMessagingApiUrlMulti,
  DshPlatformLabel::HttpMessagingApiUrlSingle,
  DshPlatformLabel::TenantPrivateVhostDomain,
  DshPlatformLabel::ProxyBrokerVhost,
  DshPlatformLabel::ProxyCommonName,
  DshPlatformLabel::ProxyConsumerGroup,
  DshPlatformLabel::ProxyConsumerGroupAcl,
  DshPlatformLabel::ProxySchemaStoreVhost,
  DshPlatformLabel::TenantProxyPrivateBootstrapServers,
  DshPlatformLabel::TenantProxyPrivateSchemaStoreHost,
  DshPlatformLabel::TenantProxyPublicBootstrapServers,
  DshPlatformLabel::TenantProxyPublicSchemaStoreHost,
  DshPlatformLabel::PublicVhostDomain,
  DshPlatformLabel::TenantPrivateAppDomain,
  DshPlatformLabel::TenantPublicAppDomain,
  DshPlatformLabel::TenantServiceConsoleUrl,
  DshPlatformLabel::TenantAppConsoleUrl,
  DshPlatformLabel::TenantAppCatalogAppUrl,
];

pub(crate) struct PlatformShow {}

#[async_trait]
impl CommandExecutor for PlatformShow {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;

    context.print_explanation(format!("configured parameters for platform '{}'", platform));
    UnitFormatter::new(platform.name(), &DSH_PLATFORM_LABELS_CONFIGURATION, context).print(&platform, None)?;

    context.print_explanation(format!("derived parameters for platform '{}'", platform));
    UnitFormatter::new(platform.name(), &DSH_PLATFORM_LABELS_DERIVED, context).print(&platform, None)?;

    if let Some(tenant) = get_target_tenant_non_interactive(matches, context.settings())? {
      context.print_explanation(format!("derived parameters for '{}@{}'", tenant, platform));
      UnitFormatter::new(format!("{}@{}", tenant, platform), &DSH_PLATFORM_LABELS_DERIVED_TENANT, context).print_non_serializable(&(&platform, &tenant), None)?;

      let provided_arguments = ProvidedArguments::try_from(matches)?;
      if let Some(description) = provided_arguments.describe() {
        context.print_explanation(format!("derived parameters for '{}@{}' with arguments", tenant, platform));
        let labels = DSH_PLATFORM_LABELS_DERIVED_ARGUMENTS
          .iter()
          .filter(|label| label.all_required_arguments_provided(&provided_arguments))
          .map(|label| label.to_owned())
          .collect_vec();
        UnitFormatter::new(format!("{}@{}\n{}", tenant, platform, description), labels.as_slice(), context)
          .print_non_serializable(&(platform.clone(), &tenant, &provided_arguments), None)?;
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

pub(crate) struct ProvidedArguments {
  pub(crate) app_id: Option<String>,
  pub(crate) bucket_id: Option<String>,
  pub(crate) proxy_id: Option<String>,
  pub(crate) service_id: Option<String>,
  pub(crate) topic_id: Option<String>,
  pub(crate) vendor_id: Option<String>,
  pub(crate) vhost: Option<String>,
}

impl ProvidedArguments {
  fn describe(&self) -> Option<String> {
    let arguments = [
      self.app_id.as_ref().map(|app_id| ("app", vec![app_id])),
      self.bucket_id.as_ref().map(|bucket_id| ("bucket", vec![bucket_id])),
      self.proxy_id.as_ref().map(|proxy_id| ("proxy", vec![proxy_id])),
      self.service_id.as_ref().map(|service_id| ("service", vec![service_id])),
      self.topic_id.as_ref().map(|topic_id| ("topic", vec![topic_id])),
      self.vendor_id.as_ref().map(|vendor_id| ("vendor", vec![vendor_id])),
      self.vhost.as_ref().map(|vhost| ("vhost", vec![vhost])),
    ]
    .into_iter()
    .flatten()
    .collect_vec();
    if arguments.is_empty() {
      None
    } else {
      Some(vec_to_table(&arguments))
    }
  }
}

impl TryFrom<&ArgMatches> for ProvidedArguments {
  type Error = DshCliError;

  fn try_from(matches: &ArgMatches) -> DshCliResult<Self> {
    Ok(Self {
      app_id: matches.get_one::<String>(APP_ID_ARGUMENT).cloned(),
      bucket_id: matches.get_one::<String>(BUCKET_ID_ARGUMENT).cloned(),
      proxy_id: matches.get_one::<String>(PROXY_ID_ARGUMENT).cloned(),
      service_id: matches.get_one::<String>(SERVICE_ID_ARGUMENT).cloned(),
      topic_id: matches.get_one::<String>(TOPIC_ID_ARGUMENT).cloned(),
      vendor_id: matches.get_one::<String>(VENDOR_NAME_ARGUMENT).cloned(),
      vhost: matches.get_one::<String>(VHOST_SUBDOMAIN_ARGUMENT).cloned(),
    })
  }
}

fn get_app_argument_or_prompt(matches: &ArgMatches) -> DshCliResult<String> {
  match matches.get_one::<String>(APP_ID_ARGUMENT) {
    Some(app_argument) => Ok(app_argument.to_string()),
    None => Ok(read_single_line("enter app: ")?),
  }
}

fn get_service_argument_or_prompt(matches: &ArgMatches) -> DshCliResult<String> {
  match matches.get_one::<String>(SERVICE_ID_ARGUMENT) {
    Some(service_argument) => Ok(service_argument.to_string()),
    None => Ok(read_single_line("enter service: ")?),
  }
}
