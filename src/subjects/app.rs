use crate::arguments::{app_id_argument, manifest_id_argument, manifest_version_argument, APP_ID_ARGUMENT};
use crate::capability::{
  Capability, CommandExecutor, DEPLOY_COMMAND, EXPLAIN_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS,
  UNDEPLOY_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{hashmap_to_table, Value};
use crate::formatters::{Label, SubjectFormatter};
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::subjects::bucket::BucketLabel;
use crate::subjects::certificate::capabilities::CERTIFICATE_LABELS_SHOW;
use crate::subjects::manifest::ManifestExplain;
use crate::subjects::service::SERVICE_LABELS_SHOW;
use crate::subjects::topic::TOPIC_LABELS;
use crate::subjects::vhost::labels::VhostLabel;
use crate::subjects::volume::VOLUME_LABELS;
use crate::target_tenant::get_target_tenant;
use crate::{cli_error, err, get_target_platform, DshCliResult};
use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::manifest::{Property, PropertyKind};
use dsh_api::types::AppCatalogAppResourcesValue;
use dsh_api::types::{AppCatalogApp, AppCatalogAppConfiguration};
use dsh_api::version::Version;
use futures::future::try_join;
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use serde_json::de::from_str;
use std::collections::HashMap;
use std::str::FromStr;

struct AppSubject {}

const APP_SUBJECT_TARGET: &str = "app";

lazy_static! {
  pub(crate) static ref APP_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(AppSubject {});
}

#[async_trait]
impl Subject for AppSubject {
  fn subject(&self) -> &'static str {
    APP_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list apps deployed from the DSH app catalog.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DEPLOY_COMMAND => Some(APP_DEPLOY_CAPABILITY.as_ref()),
      EXPLAIN_COMMAND => Some(APP_EXPLAIN_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(APP_LIST_CAPABILITY.as_ref()),
      OPEN_COMMAND => Some(APP_OPEN_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(APP_SHOW_CAPABILITY.as_ref()),
      UNDEPLOY_COMMAND => Some(APP_UNDEPLOY_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &APP_CAPABILITIES
  }
}

lazy_static! {
  static ref APP_DEPLOY_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(DEPLOY_COMMAND, None, &AppDeploy {}, "Deploy app")
      .set_long_about(DEPLOY_LONG_ABOUT)
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(manifest_version_argument().required(true))
      .add_target_argument(app_id_argument().required(true))
      .add_extra_argument(app_parameter_argument())
      .add_modifier_flag(ModifierFlagType::ImplicitDefaults, None)
  );
  static ref APP_EXPLAIN_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(EXPLAIN_COMMAND, None, &ManifestExplain {}, "Explain manifest configuration")
      .set_long_about(
        "Explain an app manifest from the app catalog. This explanation will describe all \
         parameters that are required to deploy the app. The app manifest must be identified by \
         the app manifest identifier and the (optional) manifest version. If no version is \
         provided, the latest final app manifest will be explained. This function is the same \
         as the \"manifest explain\" function."
      )
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(manifest_version_argument())
  );
  static ref APP_LIST_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &AppList {}, "List deployed apps")
      .set_long_about(
        "Lists all apps deployed from the DSH app catalog. If the --ids option is provided \
         only the app identifiers will be listed."
      )
      .add_command_executor(FlagType::Ids, &AppListIds {}, None)
  );
  static ref APP_OPEN_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(OPEN_COMMAND, Some(OPEN_COMMAND_ALIAS), &AppOpen {}, "Open app vhost")
      .set_long_about("Open the vhost of an app deployed from the DSH app catalog.")
      .add_target_argument(app_id_argument().required(true))
  );
  static ref APP_SHOW_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &AppShow {}, "Show deployed app configuration")
      .set_long_about("Show the configuration of an app deployed from the DSH app catalog.")
      .add_target_argument(app_id_argument().required(true))
  );
  static ref APP_UNDEPLOY_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(UNDEPLOY_COMMAND, None, &AppUndeploy {}, "Undeploy app")
      .set_long_about("Undeploy an app.")
      .add_target_argument(app_id_argument().required(true))
  );
  static ref APP_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    APP_DEPLOY_CAPABILITY.as_ref(),
    APP_EXPLAIN_CAPABILITY.as_ref(),
    APP_LIST_CAPABILITY.as_ref(),
    APP_OPEN_CAPABILITY.as_ref(),
    APP_SHOW_CAPABILITY.as_ref(),
    APP_UNDEPLOY_CAPABILITY.as_ref()
  ];
}

static APP_CATALOG_APP_CONFIGURATION_LABELS: [AppCatalogAppConfigurationLabel; 4] =
  [AppCatalogAppConfigurationLabel::Name, AppCatalogAppConfigurationLabel::ManifestUrn, AppCatalogAppConfigurationLabel::Stopped, AppCatalogAppConfigurationLabel::Configuration];

struct AppDeploy {}

#[async_trait]
impl CommandExecutor for AppDeploy {
  async fn execute_with_client(&self, target: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let manifest_version = Version::from_str(sub_argument.unwrap_or_else(|| unreachable!()).as_str())?;
    let app_id = matches.get_one::<String>(APP_ID_ARGUMENT).unwrap_or_else(|| unreachable!());
    if client.get_appcatalog_app_configuration(app_id).await.is_ok() {
      return err!("app '{}' already exists", app_id);
    }
    let implicit_defaults = matches.get_flag(ModifierFlagType::ImplicitDefaults.id());
    context.print_explanation(format!("get manifest '{}', version {}", manifest_id, manifest_version));
    let ((gid, uid), manifest) = try_join(client.guid(), client.manifest(manifest_id.as_str(), &manifest_version))
      .await
      .map_err(|_| cli_error!("manifest '{}:{}' does not exist", manifest_id, manifest_version))?;
    let command_line_app_parameters = match matches.get_many::<String>(APP_PARAMETER_ARGUMENT) {
      Some(app_parameters) => app_parameters
        .map(|app_parameter| parse_app_parameter(app_parameter.as_str()))
        .collect::<Result<HashMap<String, String>, _>>()?,
      None => HashMap::new(),
    };
    let mut missing_or_invalid_parameters = 0;
    let mut app_configuration: HashMap<String, String> = HashMap::new();
    app_configuration.insert("@gid".to_string(), gid.to_string());
    app_configuration.insert("@uid".to_string(), uid.to_string());
    if let Some(manifest_configuration) = manifest.configuration {
      let mut property_names = manifest_configuration.properties.keys().collect_vec();
      property_names.sort();
      for property_name in property_names {
        let property = manifest_configuration.properties.get(property_name).unwrap_or_else(|| unreachable!());
        let app_parameter: DshCliResult<String> = match command_line_app_parameters.get(property_name) {
          Some(command_line_app_parameter) => Ok(command_line_app_parameter.clone()),
          None => {
            if context.stdin_is_terminal() {
              if implicit_defaults {
                match property.default.clone() {
                  Some(property_default) => Ok(property_default),
                  None => Ok(context.read_single_line(property_name)?),
                }
              } else {
                match property.default.clone() {
                  Some(property_default) => Ok(context.read_single_line_with_default(property_name, property_default)?),
                  None => Ok(context.read_single_line(property_name)?),
                }
              }
            } else if implicit_defaults {
              match property.default.clone() {
                Some(property_default) => Ok(property_default),
                None => err!("no default for property"),
              }
            } else {
              err!("no default for property")
            }
          }
        };
        match app_parameter.and_then(|app_parameter| validate_parameter(&app_parameter, property_name, property)) {
          Ok(valid_parameter) => {
            app_configuration.insert(property_name.clone(), valid_parameter);
          }
          Err(error_message) => {
            missing_or_invalid_parameters += 1;
            context.print_error(error_message);
          }
        }
      }
    }
    if missing_or_invalid_parameters == 1 {
      return err!("missing or invalid parameter");
    } else if missing_or_invalid_parameters > 1 {
      return err!("{} missing or invalid parameters", missing_or_invalid_parameters);
    }
    let app_catalog_app_configuration = AppCatalogAppConfiguration {
      configuration: app_configuration,
      manifest_urn: format!("appcatalog/manifest/{}/{}", manifest.id, manifest.version),
      name: app_id.to_string(),
      stopped: false,
    };
    UnitFormatter::new(app_id, &APP_CATALOG_APP_CONFIGURATION_LABELS, context).print(&app_catalog_app_configuration, None)?;
    if context.dry_run() {
      context.print_warning("dry-run mode, app not deployed");
    } else {
      client.put_appcatalog_app_configuration(app_id, &app_catalog_app_configuration).await?;
      context.print_outcome(format!("app '{}' deployed", app_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn validate_parameter(parameter: &String, property_name: &str, property: &Property) -> DshCliResult<String> {
  match &property.enumeration {
    Some(enumeration) => {
      if enumeration.contains(parameter) {
        Ok(parameter.clone())
      } else {
        err!(
          "property '{}' has illegal value \"{}\", should be one of {}",
          property_name,
          parameter,
          enumeration
            .iter()
            .map(|enumeration_value| if property.kind == PropertyKind::Number { enumeration_value.to_string() } else { format!("\"{}\"", enumeration_value) })
            .join(", ")
        )
      }
    }
    None => match property.kind {
      PropertyKind::DnsZone => {
        if parameter == "private" || parameter == "public" {
          Ok(parameter.clone())
        } else {
          err!(
            "dns-zone property '{}' has illegal value \"{}\", should be \"private\" or \"public\"",
            property_name,
            parameter
          )
        }
      }
      PropertyKind::Number => {
        lazy_static! {
          static ref NUMBER_REGEX: Regex = Regex::new(r"^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$").unwrap_or_else(|_| unreachable!());
        }
        match NUMBER_REGEX.captures(parameter) {
          Some(_) => Ok(parameter.clone()),
          None => err!("property '{}' has illegal value \"{}\", should be a number", property_name, parameter),
        }
      }
      PropertyKind::String => Ok(parameter.clone()),
    },
  }
}

fn parse_app_parameter(app_parameter: &str) -> DshCliResult<(String, String)> {
  lazy_static! {
    static ref APP_PARAMETER_REGEX: Regex = Regex::new(r"^([A-Z][a-zA-Z0-9_]+)=(.*)$").unwrap_or_else(|_| unreachable!());
  }
  match APP_PARAMETER_REGEX.captures(app_parameter) {
    Some(captures) => Ok((
      captures.get(1).unwrap_or_else(|| unreachable!()).as_str().to_string(),
      captures.get(2).unwrap_or_else(|| unreachable!()).as_str().to_string(),
    )),
    None => err!("illegal app parameter {}", app_parameter),
  }
}

static APP_CATALOG_APP_LABELS: [AppCatalogAppLabel; 3] = [AppCatalogAppLabel::Target, AppCatalogAppLabel::ManifestUrn, AppCatalogAppLabel::Configuration];

struct AppList {}

#[async_trait]
impl CommandExecutor for AppList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all deployed apps and their configurations");
    let start_instant = context.now();
    let apps = client.get_appcatalogapp_configuration_map().await?;
    context.print_execution_time(start_instant);
    let mut app_ids = apps.keys().map(|k| k.to_string()).collect_vec();
    app_ids.sort();
    let mut formatter = ListFormatter::new(&APP_CATALOG_APP_LABELS, context);
    for app_id in app_ids {
      let app = apps.get(&app_id).unwrap_or_else(|| unreachable!());
      formatter.push_target_id_value(app_id, app);
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AppListIds {}

#[async_trait]
impl CommandExecutor for AppListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all deployed app ids");
    let start_instant = context.now();
    let ids = client.app_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("app id", context);
    formatter.push_target_ids(&ids);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AppOpen {}

#[async_trait]
impl CommandExecutor for AppOpen {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let app_id = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let app: AppCatalogApp = client.get_appcatalogapp_configuration(&app_id).await?;
    context.print_execution_time(start_instant);
    for resource in app.resources.values() {
      if let AppCatalogAppResourcesValue::Vhost(vhost) = resource {
        lazy_static! {
          static ref VHOST_REGEX: Regex = Regex::new(r"^([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_-]+)@([a-zA-Z0-9_-]+)$").unwrap_or_else(|_| unreachable!());
        }
        match VHOST_REGEX.captures(vhost.value.as_str()) {
          Some(captures) => {
            if let Some(zone) = captures.get(3) {
              match zone.as_str() {
                "private" => context.open_url(
                  format!(
                    "https://{}",
                    client
                      .platform()
                      .tenant_private_vhost_domain(client.tenant().name(), captures.get(1).unwrap_or_else(|| unreachable!()).as_str())?
                  ),
                  format!("private vhost for tenant '{}@{}' and app '{}'", tenant_name, platform, app_id),
                ),
                "public" => context.open_url(
                  format!(
                    "https://{}",
                    client.platform().public_vhost_domain(captures.get(1).unwrap_or_else(|| unreachable!()).as_str())
                  ),
                  format!("public vhost for tenant '{}@{}' and app '{}'", tenant_name, platform, app_id),
                ),
                illegal_zone => context.print_warning(format!("illegal zone in vhost resource {}", illegal_zone)),
              }
            }
          }
          None => context.print_warning(format!("illegal vhost string in resource {}", vhost.value)),
        }
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static BUCKET_LABELS: [BucketLabel; 3] = [BucketLabel::Target, BucketLabel::Encrypted, BucketLabel::Versioned];
static VHOST_LABELS: [VhostLabel; 2] = [VhostLabel::Target, VhostLabel::Value];

struct AppShow {}

#[async_trait]
impl CommandExecutor for AppShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let app_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for app '{}'", app_id));
    let start_instant = context.now();
    let (app_catalog_app, allocation_status) = join!(client.get_appcatalogapp_configuration(&app_id), client.get_appcatalog_app_status(&app_id));
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, APP_SUBJECT_TARGET);
    let app = app_catalog_app?;
    UnitFormatter::new(app_id, &APP_CATALOG_APP_LABELS, context).print(&app, None)?;
    for (resource_name, resource) in &app.resources {
      match resource {
        AppCatalogAppResourcesValue::Application(service) => {
          UnitFormatter::new(resource_name, &SERVICE_LABELS_SHOW, context).print(service, None)?;
        }
        AppCatalogAppResourcesValue::Bucket(bucket) => {
          UnitFormatter::new(resource_name, &BUCKET_LABELS, context).print(bucket, None)?;
        }
        AppCatalogAppResourcesValue::Certificate(certificate) => {
          UnitFormatter::new(resource_name, &CERTIFICATE_LABELS_SHOW, context).print(certificate, None)?;
        }
        AppCatalogAppResourcesValue::Secret(secret) => {
          UnitFormatter::new(resource_name, &["secret".to_string()], context).print(&secret.name, None)?;
        }
        AppCatalogAppResourcesValue::Topic(topic) => {
          UnitFormatter::new(resource_name, &TOPIC_LABELS, context).print(topic, None)?;
        }
        AppCatalogAppResourcesValue::Vhost(vhost) => {
          UnitFormatter::new(resource_name, &VHOST_LABELS, context).print(vhost, None)?;
        }
        AppCatalogAppResourcesValue::Volume(volume) => {
          UnitFormatter::new(resource_name, &VOLUME_LABELS, context).print(volume, None)?;
        }
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AppUndeploy {}

#[async_trait]
impl CommandExecutor for AppUndeploy {
  async fn execute_with_client(&self, target: Option<String>, _sub_argument: Option<String>, _matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let app_id = target.unwrap_or_else(|| unreachable!());
    if client.get_appcatalog_app_configuration(&app_id).await.is_err() {
      return err!("app '{}' does not exist", app_id);
    }
    if context.confirmed(format!("undeploy app '{}'?", app_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, app not undeployed");
      } else {
        client.delete_appcatalog_app_configuration(&app_id).await?;
        context.print_outcome(format!("app '{}' undeployed", app_id));
      }
    } else {
      context.print_outcome(format!("cancelled, app '{}' not deleted", app_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

const APP_PARAMETER_ARGUMENT: &str = "app-parameter-argument";

fn app_parameter_argument() -> Arg {
  Arg::new(APP_PARAMETER_ARGUMENT)
    .long("app-parameter")
    .short('a')
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("APP_PAR=value")
    .help("Set app configuration parameter")
    .long_help(
      "This option allows specifying the app configuration parameters. \
          The app configuration parameter must be specified as APP_PAR=value. \
          When the value contains spaces or some other special characters, specify the app \
          parameter as --app-parameter \"APP_VAR=value with space\". This option can be provided multiple times.",
    )
    .hide_short_help(false)
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum AppCatalogAppLabel {
  Configuration,
  ManifestUrn,
  Target,
}

impl Label for AppCatalogAppLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Configuration => "app configuration",
      Self::ManifestUrn => "manifest urn",
      Self::Target => "app id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<AppCatalogAppLabel> for AppCatalogApp {
  fn value(&self, label: &AppCatalogAppLabel, target_id: &str) -> Value {
    match label {
      AppCatalogAppLabel::Configuration => match &self.configuration {
        Some(configuration) => match from_str::<HashMap<String, String>>(configuration) {
          Ok(map) => Value::plain(hashmap_to_table(&map.iter().filter(|(key, _)| !key.starts_with("@")).collect())),
          Err(_) => Value::error("error"),
        },
        None => Value::Empty,
      },
      AppCatalogAppLabel::ManifestUrn => Value::plain(&self.manifest_urn),
      AppCatalogAppLabel::Target => Value::target(target_id),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum AppCatalogAppConfigurationLabel {
  Configuration,
  ManifestUrn,
  Name,
  Stopped,
}

impl Label for AppCatalogAppConfigurationLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Configuration => "app configuration",
      Self::ManifestUrn => "manifest urn",
      Self::Name => "name",
      Self::Stopped => "stopped",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Name)
  }
}

impl SubjectFormatter<AppCatalogAppConfigurationLabel> for AppCatalogAppConfiguration {
  fn value(&self, label: &AppCatalogAppConfigurationLabel, target_id: &str) -> Value {
    match label {
      AppCatalogAppConfigurationLabel::Configuration => Value::plain(hashmap_to_table(&self.configuration)),
      AppCatalogAppConfigurationLabel::ManifestUrn => Value::plain(&self.manifest_urn),
      AppCatalogAppConfigurationLabel::Name => Value::target(target_id),
      AppCatalogAppConfigurationLabel::Stopped => Value::plain(self.stopped),
    }
  }
}

const DEPLOY_LONG_ABOUT: &str = "Deploy an app from the app catalog. \
  The app to deploy must be identified by the app manifest identifier and version. \
  The deployed app must be given a unique app identifier, which is also used to \
  construct the vhost name if applicable. The app configuration parameters can be specified as \
  command line arguments using the --app-parameter option.";
