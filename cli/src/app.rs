use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::{AppCatalogApp, AppCatalogAppResourcesValue, Application};
use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::app::{app_to_default_vector, default_app_column_labels};
use crate::formatters::application::default_application_table;
use crate::subject::Subject;
use crate::tabular::{make_tabular_with_headers, print_table};
use crate::CommandResult;

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

  fn capabilities(&self) -> HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> {
    let mut capabilities: HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> = HashMap::new();
    capabilities.insert(CapabilityType::List, &APP_LIST_CAPABILITY);
    capabilities.insert(CapabilityType::Show, &APP_SHOW_CAPABILITY);
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
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let app_ids = dsh_api_client.get_app_ids().await?;
    let allocation_statuses = futures::future::join_all(app_ids.iter().map(|app_id| dsh_api_client.get_app_catalog_app_allocation_status(app_id))).await;
    let mut table: Vec<Vec<String>> = vec![];
    for (app_id, allocation_status) in app_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(app_id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(APP_SUBJECT_TARGET), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct AppListConfiguration {}

#[async_trait]
impl CommandExecutor for AppListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let apps = &dsh_api_client.get_app_configurations().await?;
    let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
    app_ids.sort();
    let mut table: Vec<Vec<String>> = vec![];
    for app_id in app_ids {
      let app = apps.get(&app_id).unwrap();
      table.push(app_to_default_vector(app_id.as_str(), app));
    }
    for line in make_tabular_with_headers(&default_app_column_labels(), table) {
      println!("{}", line)
    }
    Ok(())
  }
}

struct AppListIds {}

#[async_trait]
impl CommandExecutor for AppListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let app_ids = dsh_api_client.get_app_ids().await?;
    for app_id in app_ids {
      println!("{}", app_id)
    }
    Ok(())
  }
}

struct AppShowAll {}

#[async_trait]
impl CommandExecutor for AppShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let app_id = target.unwrap_or_else(|| unreachable!());
    let app = dsh_api_client.get_app_configuration(app_id.as_str()).await?;
    println!("name:                 {}", app.name);
    println!("manifest urn:         {}", app.manifest_urn);
    println!("configuration:        {}", app.configuration.clone().unwrap_or("none".to_string()));
    for (resource_name, resource) in &app.resources {
      match resource {
        AppCatalogAppResourcesValue::Application(application) => {
          println!("resource/application: {}", resource_name);
          print_table(default_application_table(app.name.as_str(), application), "  ", "  ", "");
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
          println!("  {:?}", vhost)
        }
        AppCatalogAppResourcesValue::Volume(volume) => {
          println!("resource/volume:      {}", resource_name);
          println!("  {:?}", volume)
        }
      }
    }
    Ok(())
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
