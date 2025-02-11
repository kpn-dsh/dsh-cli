use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
use serde_json::de::from_str;
use std::collections::HashMap;
use std::time::Instant;

use crate::formatters::formatter::{Label, SubjectFormatter};
use dsh_api::types::AppCatalogApp;
use serde::Serialize;

use dsh_api::types::AppCatalogAppResourcesValue;

use crate::arguments::target_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR, SHOW_COMMAND, SHOW_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::{Requirements, Subject};
use crate::subjects::application::APPLICATION_LABELS_SHOW;
use crate::subjects::bucket::BUCKET_LABELS;
use crate::subjects::certificate::CERTIFICATE_LABELS_SHOW;
use crate::subjects::topic::TOPIC_LABELS;
use crate::subjects::vhost::VHOST_LABELS;
use crate::subjects::volume::VOLUME_LABELS;
use crate::DshCliResult;

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

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(false, true, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(APP_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(APP_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &APP_CAPABILITIES
  }
}

lazy_static! {
  static ref APP_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List deployed apps")
      .set_long_about("Lists all apps deployed from the DSH app catalog.")
      .set_default_command_executor(&AppListConfiguration {})
      .add_command_executor(FlagType::Ids, &AppListIds {}, None)
      .set_run_all_executors(true)
  );
  static ref APP_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND_PAIR, "Show app configuration")
      .set_long_about("Show the configuration of an app deployed from the DSH app catalog.")
      .set_default_command_executor(&AppShowAll {})
      .add_target_argument(target_argument(APP_SUBJECT_TARGET, None))
  );
  static ref APP_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![APP_LIST_CAPABILITY.as_ref(), APP_SHOW_CAPABILITY.as_ref()];
}

struct AppListConfiguration {}

#[async_trait]
impl CommandExecutor for AppListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all deployed apps and their configurations");
    let start_instant = Instant::now();
    let apps = context.dsh_api_client.as_ref().unwrap().get_appcatalogapp_configuration_map().await?;
    context.print_execution_time(start_instant);
    let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<_>>();
    app_ids.sort();
    let mut formatter = ListFormatter::new(&APP_CATALOG_APP_LABELS, Some("app id"), context);
    for app_id in app_ids {
      let app = apps.get(&app_id).unwrap();
      formatter.push_target_id_value(app_id, app);
    }
    formatter.print()?;
    Ok(())
  }
}

struct AppListIds {}

#[async_trait]
impl CommandExecutor for AppListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all deployed app ids");
    let start_instant = Instant::now();
    let ids = context.dsh_api_client.as_ref().unwrap().list_app_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("app id", context);
    formatter.push_target_ids(&ids);
    formatter.print()?;
    Ok(())
  }
}

struct AppShowAll {}

#[async_trait]
impl CommandExecutor for AppShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let app_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for app '{}'", app_id));
    let start_instant = Instant::now();
    let app = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .get_appcatalogapp_appcatalogappid_configuration(app_id.as_str())
      .await?;
    context.print_execution_time(start_instant);
    for (resource_name, resource) in &app.resources {
      match resource {
        AppCatalogAppResourcesValue::Application(application) => {
          UnitFormatter::new(resource_name, &APPLICATION_LABELS_SHOW, Some("application resource"), application, context).print()?;
        }
        AppCatalogAppResourcesValue::Bucket(bucket) => {
          UnitFormatter::new(resource_name, &BUCKET_LABELS, Some("bucket resource"), bucket, context).print()?;
        }
        AppCatalogAppResourcesValue::Certificate(certificate) => {
          UnitFormatter::new(resource_name, &CERTIFICATE_LABELS_SHOW, Some("certificate resource"), certificate, context).print()?;
        }
        AppCatalogAppResourcesValue::Secret(secret) => {
          UnitFormatter::new(resource_name, &["secret".to_string()], Some("secret"), &secret.name, context).print()?;
        }
        AppCatalogAppResourcesValue::Topic(topic) => {
          UnitFormatter::new(resource_name, &TOPIC_LABELS, Some("topic resource"), topic, context).print()?;
        }
        AppCatalogAppResourcesValue::Vhost(vhost) => {
          UnitFormatter::new(resource_name, &VHOST_LABELS, Some("vhost resource"), vhost, context).print()?;
        }
        AppCatalogAppResourcesValue::Volume(volume) => {
          UnitFormatter::new(resource_name, &VOLUME_LABELS, Some("volume resource"), volume, context).print()?;
        }
      }
    }
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum AppCatalogAppLabel {
  Configuration,
  ManifestUrl,
  Target,
}

impl Label for AppCatalogAppLabel {
  fn as_str(&self) -> &str {
    match self {
      AppCatalogAppLabel::Configuration => "app configuration",
      AppCatalogAppLabel::ManifestUrl => "manifest url",
      AppCatalogAppLabel::Target => "app id",
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
          Ok(map) => {
            let mut keys: Vec<String> = map
              .keys()
              .filter_map(|key| if !key.starts_with("@") { Some(key.to_string()) } else { None })
              .collect();
            keys.sort();
            keys
              .iter()
              .map(|key| format!("{}: {}", key, map.get(key).map(|v| v.to_string()).unwrap_or("".to_string())))
              .collect::<Vec<_>>()
              .join("\n")
          }
          Err(_) => "error".to_string(),
        },
        None => "empty".to_string(),
      },
      AppCatalogAppLabel::ManifestUrl => self.manifest_urn.clone(),
      AppCatalogAppLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<AppCatalogAppLabel> {
    Some(AppCatalogAppLabel::Target)
  }
}

pub static APP_CATALOG_APP_LABELS: [AppCatalogAppLabel; 3] = [AppCatalogAppLabel::Target, AppCatalogAppLabel::ManifestUrl, AppCatalogAppLabel::Configuration];
