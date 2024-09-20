use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{AppCatalogApp, AppCatalogAppResourcesValue, Application};

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::print_allocation_statuses;
use crate::formatters::app::APP_CATALOG_APP_LABELS;
use crate::formatters::application::APPLICATION_LABELS_SHOW;
use crate::formatters::formatter::{print_ids, TableBuilder};
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

  fn subject_first_upper(&self) -> &'static str {
    "App"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH App Catalog apps.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list apps deployed from the DSH App Catalog.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    None
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, APP_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, APP_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref APP_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List apps".to_string(),
    command_long_about: Some("Lists all available App Catalog apps.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &AppListConfiguration {}, None),
      (FlagType::AllocationStatus, &AppListAllocationStatus {}, None),
      (FlagType::Configuration, &AppListConfiguration {}, None),
      (FlagType::Ids, &AppListIds {}, None),
    ],
    default_command_executor: Some(&AppListIds {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref APP_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show app configuration".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: Some("".to_string()),
    command_executors: vec![(FlagType::All, &AppShowAll {}, None)],
    default_command_executor: Some(&AppShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

struct AppListAllocationStatus {}

#[async_trait]
impl CommandExecutor for AppListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all apps and their application status");
    }
    let app_ids = dsh_api_client.get_app_ids().await?;
    let allocation_statuses = try_join_all(app_ids.iter().map(|app_id| dsh_api_client.get_app_catalog_app_allocation_status(app_id))).await?;
    print_allocation_statuses(app_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct AppListConfiguration {}

#[async_trait]
impl CommandExecutor for AppListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all apps and their configurations");
    }
    let apps = &dsh_api_client.get_app_configurations().await?;
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all app ids");
    }
    print_ids("app ids".to_string(), dsh_api_client.get_app_ids().await?, context);
    Ok(false)
  }
}

struct AppShowAll {}

#[async_trait]
impl CommandExecutor for AppShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let app_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for app '{}'", app_id);
    }
    let app = dsh_api_client.get_app_configuration(app_id.as_str()).await?;
    println!("name:                 {}", app.name);
    println!("manifest urn:         {}", app.manifest_urn);
    println!("configuration:        {}", app.configuration.clone().unwrap_or("none".to_string()));
    for (resource_name, resource) in &app.resources {
      match resource {
        AppCatalogAppResourcesValue::Application(application) => {
          println!("resource/application: {}", resource_name);
          let mut builder = TableBuilder::show(&APPLICATION_LABELS_SHOW, context);
          builder.value("".to_string(), application);
          builder.print();
        }
        AppCatalogAppResourcesValue::Bucket(bucket) => {
          println!("resource/bucket:      {}", resource_name);
          println!("  {:?}", bucket)
        }
        AppCatalogAppResourcesValue::Certificate(certificate) => {
          println!("resource/certificate: {}", resource_name);
          println!("  {:?}", certificate)
        }
        AppCatalogAppResourcesValue::Secret(secret) => {
          println!("resource/secret:      {}", resource_name);
          println!("  {:?}", secret)
        }
        AppCatalogAppResourcesValue::Topic(topic) => {
          println!("resource/topic:       {}", resource_name);
          println!("  {:?}", topic)
        }
        AppCatalogAppResourcesValue::Vhost(vhost) => {
          println!("resource/vhost:       {}", resource_name);
          println!("                      {:?}", vhost)
        }
        AppCatalogAppResourcesValue::Volume(volume) => {
          println!("resource/volume:      {}", resource_name);
          println!("  {:?}", volume)
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

pub(crate) fn _apps_that_use_value(value: &str, apps: &HashMap<String, AppCatalogApp>) -> Vec<(String, Vec<String>)> {
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

// Returns vector with pairs (application_id, secret -> environment variables)
pub(crate) fn apps_with_secret_injections(secrets: &[String], apps: &HashMap<String, AppCatalogApp>) -> Vec<(String, HashMap<String, Vec<String>>)> {
  let mut app_ids: Vec<String> = apps.keys().map(|p| p.to_string()).collect();
  app_ids.sort();
  let mut pairs: Vec<(String, HashMap<String, Vec<String>>)> = vec![];
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
          pairs.push((application_id.clone(), injections));
        }
      }
    }
  }
  pairs
}
