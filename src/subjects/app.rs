use crate::arguments::{app_id_argument, manifest_id_argument, manifest_version_argument, APP_ID_ARGUMENT};
use crate::capability::{
  Capability, CommandExecutor, DEPLOY_COMMAND, EXPLAIN_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS,
  UNDEPLOY_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::formatter::{hashmap_to_table, Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::subjects::bucket::BUCKET_LABELS;
use crate::subjects::certificate::CERTIFICATE_LABELS_SHOW;
use crate::subjects::manifest::ManifestExplain;
use crate::subjects::service::SERVICE_LABELS_SHOW;
use crate::subjects::topic::TOPIC_LABELS;
use crate::subjects::vhost::VHOST_LABELS;
use crate::subjects::volume::VOLUME_LABELS;
use crate::{get_target_platform, get_target_tenant, DshCliResult};
use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::manifest::{Property, PropertyKind};
use dsh_api::types::AppCatalogAppResourcesValue;
use dsh_api::types::{AppCatalogApp, AppCatalogAppConfiguration};
use dsh_api::version::Version;
use futures::future::try_join;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use serde_json::de::from_str;
use std::collections::HashMap;
use std::str::FromStr;

pub(crate) struct AppSubject {}

const APP_SUBJECT_TARGET: &str = "app";

lazy_static! {
  pub static ref APP_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(AppSubject {});
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
  static ref APP_DEPLOY_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DEPLOY_COMMAND, None, &AppDeploy {}, "Deploy app")
      .set_long_about(DEPLOY_LONG_ABOUT)
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(manifest_version_argument().required(true))
      .add_target_argument(app_id_argument().required(true))
      .add_extra_argument(app_parameter_argument())
      .add_modifier_flag(ModifierFlagType::ImplicitDefaults, None)
  );
  static ref APP_EXPLAIN_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
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
  static ref APP_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &AppListConfiguration {}, "List deployed apps")
      .set_long_about(
        "Lists all apps deployed from the DSH app catalog. If the --ids option is provided \
         only the app identifiers will be listed."
      )
      .add_command_executor(FlagType::Ids, &AppListIds {}, None)
  );
  static ref APP_OPEN_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(OPEN_COMMAND, Some(OPEN_COMMAND_ALIAS), &AppOpen {}, "Open app vhost")
      .set_long_about("Open the vhost of an app deployed from the DSH app catalog.")
      .add_target_argument(app_id_argument().required(true))
  );
  static ref APP_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &AppShowAll {}, "Show deployed app configuration")
      .set_long_about("Show the configuration of an app deployed from the DSH app catalog.")
      .add_target_argument(app_id_argument().required(true))
  );
  static ref APP_UNDEPLOY_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
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

struct AppDeploy {}

#[async_trait]
impl CommandExecutor for AppDeploy {
  async fn execute_with_client(&self, target: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let manifest_version = Version::from_str(sub_argument.unwrap().as_str())?;
    let app_id = matches.get_one::<String>(APP_ID_ARGUMENT).unwrap_or_else(|| unreachable!());
    if client.get_appcatalog_app_configuration(app_id).await.is_ok() {
      return Err(format!("app '{}' already exists", app_id));
    }
    let implicit_defaults = matches.get_flag(ModifierFlagType::ImplicitDefaults.id());
    context.print_explanation(format!("get manifest '{}', version {}", manifest_id, manifest_version));
    let ((gid, uid), manifest) = try_join(client.guid(), client.manifest(manifest_id.as_str(), &manifest_version))
      .await
      .map_err(|_| format!("manifest '{}:{}' does not exist", manifest_id, manifest_version))?;
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
        let app_parameter: Result<String, String> = match command_line_app_parameters.get(property_name) {
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
                None => Err("no default for property".to_string()),
              }
            } else {
              Err("no default for property".to_string())
            }
          }
        };
        match app_parameter.and_then(|a| validate_parameter(&a, property_name, property)) {
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
      return Err("missing or invalid parameter".to_string());
    } else if missing_or_invalid_parameters > 1 {
      return Err(format!("{} missing or invalid parameters", missing_or_invalid_parameters));
    }
    let app_catalog_app_configuration = AppCatalogAppConfiguration {
      configuration: app_configuration,
      manifest_urn: format!("appcatalog/manifest/{}/{}", manifest.id, manifest.version),
      name: app_id.to_string(),
      stopped: false,
    };
    UnitFormatter::new(app_id, &APP_CATALOG_APP_CONFIGURATION_LABELS, Some("app id"), context).print(&app_catalog_app_configuration, None)?;
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

fn validate_parameter(parameter: &String, property_name: &str, property: &Property) -> Result<String, String> {
  match &property.enumeration {
    Some(enumeration) => {
      if enumeration.contains(parameter) {
        Ok(parameter.clone())
      } else {
        Err(format!(
          "property {} has illegal value \"{}\", should be one of {}",
          property_name,
          parameter,
          enumeration
            .iter()
            .map(|enumeration_value| if property.kind == PropertyKind::Number { enumeration_value.to_string() } else { format!("\"{}\"", enumeration_value) })
            .join(", ")
        ))
      }
    }
    None => match property.kind {
      PropertyKind::DnsZone => {
        if parameter == "private" || parameter == "public" {
          Ok(parameter.clone())
        } else {
          Err(format!(
            "dns-zone property {} has illegal value \"{}\", should be \"private\" or \"public\"",
            property_name, parameter
          ))
        }
      }
      PropertyKind::Number => {
        lazy_static! {
          static ref NUMBER_REGEX: Regex = Regex::new(r"^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$").unwrap();
        }
        match NUMBER_REGEX.captures(parameter) {
          Some(_) => Ok(parameter.clone()),
          None => Err(format!("property {} has illegal value \"{}\", should be a number", property_name, parameter)),
        }
      }
      PropertyKind::String => Ok(parameter.clone()),
    },
  }
}

fn parse_app_parameter(app_parameter: &str) -> Result<(String, String), String> {
  lazy_static! {
    static ref APP_PARAMETER_REGEX: Regex = Regex::new(r"^([A-Z][a-zA-Z0-9_]+)=(.*)$").unwrap();
  }
  match APP_PARAMETER_REGEX.captures(app_parameter) {
    Some(captures) => Ok((captures.get(1).unwrap().as_str().to_string(), captures.get(2).unwrap().as_str().to_string())),
    None => Err(format!("illegal app parameter {}", app_parameter)),
  }
}

struct AppListConfiguration {}

#[async_trait]
impl CommandExecutor for AppListConfiguration {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all deployed apps and their configurations");
    let start_instant = context.now();
    let apps = client.get_appcatalogapp_configuration_map().await?;
    context.print_execution_time(start_instant);
    let mut app_ids = apps.keys().map(|k| k.to_string()).collect_vec();
    app_ids.sort();
    let mut formatter = ListFormatter::new(&APP_CATALOG_APP_LABELS, Some("app id"), context);
    for app_id in app_ids {
      let app = apps.get(&app_id).unwrap();
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all deployed app ids");
    let start_instant = context.now();
    let ids = client.app_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("app id", context);
    formatter.push_target_ids(&ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AppOpen {}

#[async_trait]
impl CommandExecutor for AppOpen {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let app_id = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let app: AppCatalogApp = client.get_appcatalogapp_configuration(&app_id).await?;
    context.print_execution_time(start_instant);
    for resource in app.resources.values() {
      if let AppCatalogAppResourcesValue::Vhost(vhost) = resource {
        lazy_static! {
          static ref VHOST_REGEX: Regex = Regex::new(r"^([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_-]+)@([a-zA-Z0-9_-]+)$").unwrap();
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
                      .tenant_private_vhost_domain(client.tenant().name(), captures.get(1).unwrap().as_str())?
                  ),
                  format!("private vhost for tenant '{}@{}' and app '{}'", tenant_name, platform, app_id),
                ),
                "public" => context.open_url(
                  format!("https://{}", client.platform().public_vhost_domain(captures.get(1).unwrap().as_str())),
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

struct AppShowAll {}

#[async_trait]
impl CommandExecutor for AppShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let app_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for app '{}'", app_id));
    let start_instant = context.now();
    let app: AppCatalogApp = client.get_appcatalogapp_configuration(&app_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(app_id, &APP_CATALOG_APP_LABELS, Some("app id"), context).print(&app, None)?;
    for (resource_name, resource) in &app.resources {
      match resource {
        AppCatalogAppResourcesValue::Application(service) => {
          UnitFormatter::new(resource_name, &SERVICE_LABELS_SHOW, Some("service resource"), context).print(service, None)?;
        }
        AppCatalogAppResourcesValue::Bucket(bucket) => {
          UnitFormatter::new(resource_name, &BUCKET_LABELS, Some("bucket resource"), context).print(bucket, None)?;
        }
        AppCatalogAppResourcesValue::Certificate(certificate) => {
          UnitFormatter::new(resource_name, &CERTIFICATE_LABELS_SHOW, Some("certificate resource"), context).print(certificate, None)?;
        }
        AppCatalogAppResourcesValue::Secret(secret) => {
          UnitFormatter::new(resource_name, &["secret".to_string()], Some("secret"), context).print(&secret.name, None)?;
        }
        AppCatalogAppResourcesValue::Topic(topic) => {
          UnitFormatter::new(resource_name, &TOPIC_LABELS, Some("topic resource"), context).print(topic, None)?;
        }
        AppCatalogAppResourcesValue::Vhost(vhost) => {
          UnitFormatter::new(resource_name, &VHOST_LABELS, Some("vhost resource"), context).print(vhost, None)?;
        }
        AppCatalogAppResourcesValue::Volume(volume) => {
          UnitFormatter::new(resource_name, &VOLUME_LABELS, Some("volume resource"), context).print(volume, None)?;
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
  async fn execute_with_client(&self, target: Option<String>, _sub_argument: Option<String>, _matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let app_id = target.unwrap_or_else(|| unreachable!());
    if client.get_appcatalog_app_configuration(&app_id).await.is_err() {
      return Err(format!("app '{}' does not exist", app_id));
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

pub(crate) fn app_parameter_argument() -> Arg {
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
pub(crate) enum AppCatalogAppLabel {
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
  fn value(&self, label: &AppCatalogAppLabel, target_id: &str) -> String {
    match label {
      AppCatalogAppLabel::Configuration => match &self.configuration {
        Some(configuration) => match from_str::<HashMap<String, String>>(configuration) {
          Ok(map) => hashmap_to_table(&map.iter().filter(|(key, _)| !key.starts_with("@")).collect()),
          Err(_) => "error".to_string(),
        },
        None => "empty".to_string(),
      },
      AppCatalogAppLabel::ManifestUrn => self.manifest_urn.clone(),
      AppCatalogAppLabel::Target => target_id.to_string(),
    }
  }
}

pub static APP_CATALOG_APP_LABELS: [AppCatalogAppLabel; 3] = [AppCatalogAppLabel::Target, AppCatalogAppLabel::ManifestUrn, AppCatalogAppLabel::Configuration];

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum AppCatalogAppConfigurationLabel {
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
  fn value(&self, label: &AppCatalogAppConfigurationLabel, target_id: &str) -> String {
    match label {
      AppCatalogAppConfigurationLabel::Configuration => hashmap_to_table(&self.configuration),
      AppCatalogAppConfigurationLabel::ManifestUrn => self.manifest_urn.clone(),
      AppCatalogAppConfigurationLabel::Name => target_id.to_string(),
      AppCatalogAppConfigurationLabel::Stopped => self.stopped.to_string(),
    }
  }
}

pub static APP_CATALOG_APP_CONFIGURATION_LABELS: [AppCatalogAppConfigurationLabel; 4] =
  [AppCatalogAppConfigurationLabel::Name, AppCatalogAppConfigurationLabel::ManifestUrn, AppCatalogAppConfigurationLabel::Stopped, AppCatalogAppConfigurationLabel::Configuration];

const DEPLOY_LONG_ABOUT: &str = "Deploy an app from the app catalog. \
  The app to deploy must be identified by the app manifest identifier and version. \
  The deployed app must be given a unique app identifier, which is also used to \
  construct the vhost name if applicable. The app configuration parameters can be specified as \
  command line arguments using the --app-parameter option.";
