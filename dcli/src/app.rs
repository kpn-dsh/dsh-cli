use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use dsh_api::types::{AppCatalogApp, AppCatalogAppResourcesValue, Application};

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::flags::FlagType;
use crate::formatters::app::APP_CATALOG_APP_LABELS;
use crate::formatters::application::APPLICATION_LABELS_SHOW;
use crate::formatters::bucket::BUCKET_LABELS;
use crate::formatters::certificate::CERTIFICATE_LABELS_SHOW;
use crate::formatters::formatter::{print_vec, StringTableBuilder, TableBuilder};
use crate::formatters::show_table::ShowTable;
use crate::formatters::topic::TOPIC_LABELS;
use crate::formatters::volume::VOLUME_LABELS;
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

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

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, APP_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, APP_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref APP_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List deployed apps")
      .set_long_about("Lists all apps deployed from the DSH app catalog.")
      .set_default_command_executor(&AppListConfiguration {})
      .add_command_executor(FlagType::Ids, &AppListIds {}, None)
      .set_run_all_executors(true)
  );
  pub static ref APP_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show app configuration")
      .set_long_about("Show the configuration of an app deployed from the DSH app catalog.")
      .set_default_command_executor(&AppShowAll {})
      .add_target_argument(target_argument(APP_SUBJECT_TARGET, None))
  );
}

struct AppListConfiguration {}

#[async_trait]
impl CommandExecutor for AppListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all deployed apps and their configurations");
    }
    let apps = context.dsh_api_client.as_ref().unwrap().get_app_configurations().await?;
    let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    app_ids.sort();
    let mut builder = TableBuilder::list(&APP_CATALOG_APP_LABELS, context);
    for app_id in app_ids {
      let app = apps.get(&app_id).unwrap();
      builder.value(app_id, app);
    }
    builder.print();
    Ok(false)
  }
}

struct AppListIds {}

#[async_trait]
impl CommandExecutor for AppListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all deployed app ids");
    }
    print_vec("app ids".to_string(), context.dsh_api_client.as_ref().unwrap().list_app_ids().await?, context);
    Ok(false)
  }
}

struct AppShowAll {}

#[async_trait]
impl CommandExecutor for AppShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let app_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for app '{}'", app_id);
    }
    let app = context.dsh_api_client.as_ref().unwrap().get_app_configuration(app_id.as_str()).await?;
    ShowTable::new(app_id.as_str(), &app, &APP_CATALOG_APP_LABELS, context).print();
    for (resource_name, resource) in &app.resources {
      match resource {
        AppCatalogAppResourcesValue::Application(application) => {
          ShowTable::new(resource_name.as_str(), application, &APPLICATION_LABELS_SHOW, context).print();
        }
        AppCatalogAppResourcesValue::Bucket(bucket) => {
          ShowTable::new(resource_name.as_str(), bucket, &BUCKET_LABELS, context).print();
        }
        AppCatalogAppResourcesValue::Certificate(certificate) => {
          ShowTable::new(resource_name.as_str(), certificate, &CERTIFICATE_LABELS_SHOW, context).print();
        }
        AppCatalogAppResourcesValue::Secret(secret) => {
          let mut builder = StringTableBuilder::new(&["resource", "secret"], context);
          builder.vec(&vec![resource_name.to_string(), secret.to_string()]);
          builder.print_show();
        }
        AppCatalogAppResourcesValue::Topic(topic) => {
          ShowTable::new(resource_name.as_str(), topic, &TOPIC_LABELS, context).print();
        }
        AppCatalogAppResourcesValue::Vhost(vhost) => {
          let mut builder = StringTableBuilder::new(&["resource", "vhost"], context);
          builder.vec(&vec![resource_name.to_string(), vhost.to_string()]);
          builder.print_show();
        }
        AppCatalogAppResourcesValue::Volume(volume) => {
          ShowTable::new(resource_name.as_str(), volume, &VOLUME_LABELS, context).print();
        }
      }
    }
    Ok(false)
  }
}

/// Get application for this app
///
/// ## Returns
/// * (resource_id, application)
pub(crate) fn get_application_from_app(app: &AppCatalogApp) -> Option<(&String, &Application)> {
  app.resources.iter().find_map(|(resource_id, resource)| match resource {
    AppCatalogAppResourcesValue::Application(application) => Some((resource_id, application)),
    _ => None,
  })
}

pub(crate) fn _apps_that_use_env_value(value: &str, apps: &HashMap<String, AppCatalogApp>) -> Vec<(String, Vec<String>)> {
  let mut app_ids: Vec<String> = apps.keys().map(|p| p.to_string()).collect();
  app_ids.sort();
  let mut pairs: Vec<(String, Vec<String>)> = vec![];
  for app_id in app_ids {
    let app = apps.get(&app_id).unwrap();
    if let Some((application_id, application)) = get_application_from_app(app) {
      if !application.env.is_empty() {
        let envs_that_contain_value: Vec<String> = application.env.clone().into_iter().filter(|(_, v)| v.contains(value)).map(|(k, _)| k).collect();
        pairs.push((application_id.clone(), envs_that_contain_value));
      }
    }
  }
  pairs
}

// Returns vector with pairs (application_id, instances, secret -> environment variables)
pub(crate) fn apps_with_secret_injections(secrets: &[String], apps: &HashMap<String, AppCatalogApp>) -> Vec<(String, u64, HashMap<String, Vec<String>>)> {
  let mut app_ids: Vec<String> = apps.keys().map(|p| p.to_string()).collect();
  app_ids.sort();
  let mut pairs: Vec<(String, u64, HashMap<String, Vec<String>>)> = vec![];
  for app_id in app_ids {
    let app = apps.get(&app_id).unwrap();
    if let Some((application_id, application)) = get_application_from_app(app) {
      if !application.secrets.is_empty() {
        let mut injections = HashMap::<String, Vec<String>>::new();
        for application_secret in &application.secrets {
          if secrets.contains(&application_secret.name) {
            let mut env_injections = vec![];
            for application_secret_injection in &application_secret.injections {
              if let Some(env_injection) = application_secret_injection.get("env") {
                env_injections.push(env_injection.to_string());
              }
            }
            if !env_injections.is_empty() {
              injections.insert(application_secret.name.clone(), env_injections);
            }
          }
        }
        if !injections.is_empty() {
          pairs.push((application_id.clone(), application.instances, injections));
        }
      }
    }
  }
  pairs
}

pub(crate) fn apps_that_use_volume(volume_id: &str, apps: &HashMap<String, AppCatalogApp>) -> Vec<(String, u64, String)> {
  let mut app_ids: Vec<String> = apps.keys().map(|p| p.to_string()).collect();
  app_ids.sort();
  let mut pairs: Vec<(String, u64, String)> = vec![];
  for app_id in app_ids {
    let app = apps.get(&app_id).unwrap();
    if let Some((application_id, application)) = get_application_from_app(app) {
      for (path, volume) in application.volumes.clone() {
        if volume.name.contains(&format!("volume('{}')", volume_id)) {
          pairs.push((application_id.clone(), application.instances, path))
        }
      }
    }
  }
  pairs
}
