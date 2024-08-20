use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use trifonius_dsh_api::types::{AppCatalogApp, AppCatalogAppResourcesValue, Application};
use trifonius_dsh_api::DshApiClient;

use crate::arguments::Flag;
use crate::command::SubjectCommand;
use crate::formatters::allocation_status::{allocation_status_table_column_labels, allocation_status_to_table_row};
use crate::formatters::app::{app_to_default_vector, default_app_column_labels};
use crate::formatters::application::default_application_table;
use crate::tabular::{make_tabular_with_headers, print_table};
use crate::CommandResult;

pub(crate) struct AppCommand {}

lazy_static! {
  pub static ref APP_COMMAND: Box<(dyn SubjectCommand + Send + Sync)> = Box::new(AppCommand {});
}

#[async_trait]
impl SubjectCommand for AppCommand {
  fn subject(&self) -> &'static str {
    "app"
  }

  fn subject_first_upper(&self) -> &'static str {
    "App"
  }

  fn alias(&self) -> Option<&str> {
    None
  }

  fn about(&self) -> String {
    "Show, manage and list apps.".to_string()
  }

  fn long_about(&self) -> String {
    "Show, manage and list apps deployed from the DSH App Catalog.".to_string()
  }

  fn list_flags(&self) -> &'static [Flag] {
    &[Flag::All, Flag::AllocationStatus, Flag::Configuration, Flag::Ids]
  }

  fn show_flags(&self) -> &'static [Flag] {
    &[Flag::All]
  }

  async fn list_all(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_configuration(matches, dsh_api_client).await
  }

  async fn list_allocation_status(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let app_ids = dsh_api_client.get_app_ids().await?;
    let allocation_statuses = futures::future::join_all(app_ids.iter().map(|app_id| dsh_api_client.get_app_catalog_app_allocation_status(app_id))).await;
    let mut table: Vec<Vec<String>> = vec![];
    for (app_id, allocation_status) in app_ids.iter().zip(allocation_statuses) {
      table.push(allocation_status_to_table_row(app_id, allocation_status.ok().as_ref()));
    }
    for line in make_tabular_with_headers(&allocation_status_table_column_labels(self.subject()), table) {
      println!("{}", line)
    }
    Ok(())
  }

  async fn list_configuration(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
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

  async fn list_ids(&self, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let app_ids = dsh_api_client.get_app_ids().await?;
    for app_id in app_ids {
      println!("{}", app_id)
    }
    Ok(())
  }

  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_ids(matches, dsh_api_client).await
  }

  async fn show_all(&self, target_id: &str, _matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let app = dsh_api_client.get_app_configuration(target_id).await?;
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

  async fn show_default(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.show_all(target_id, matches, dsh_api_client).await
  }
}

/// ## Returns
/// * (resource_id, application)
pub(crate) fn get_application_from_app(app: &AppCatalogApp) -> Option<(&String, &Application)> {
  app.resources.iter().find_map(|(resource_id, resource)| match resource {
    AppCatalogAppResourcesValue::Application(application) => Some((resource_id, application)),
    _ => None,
  })
}
