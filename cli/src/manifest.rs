use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
use serde_json::{from_str, Value};

use trifonius_dsh_api::types::{AppCatalogAppResourcesValue, AppCatalogManifest};
use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::app::{app_to_default_vector, default_app_column_labels};
use crate::formatters::application::default_application_table;
use crate::subject::Subject;
use crate::tabular::{make_tabular_with_headers, print_table};
use crate::CommandResult;

pub(crate) struct ManifestSubject {}

const MANIFEST_SUBJECT_TARGET: &str = "manifest";

lazy_static! {
  pub static ref MANIFEST_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ManifestSubject {});
}

#[async_trait]
impl Subject for ManifestSubject {
  fn subject(&self) -> &'static str {
    MANIFEST_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Manifest"
  }

  fn subject_command_about(&self) -> String {
    "Show App Catalog manifests.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show the manifest files for the apps in the DSH App Catalog.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> {
    let mut capabilities: HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>> = HashMap::new();
    capabilities.insert(CapabilityType::List, &MANIFEST_LIST_CAPABILITY);
    capabilities.insert(CapabilityType::Show, &MANIFEST_SHOW_CAPABILITY);
    capabilities
  }
}

lazy_static! {
  pub static ref MANIFEST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List manifests".to_string(),
    command_long_about: Some("Lists all manifest files from the App Catalog.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::All, &ManifestListAll {}, None), (FlagType::Configuration, &ManifestListConfiguration {}, None), (FlagType::Ids, &ManifestListIds {}, None),],
    default_command_executor: Some(&ManifestListAll {}),
    run_all_executors: true,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
  pub static ref MANIFEST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show manifest configuration".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::All, &ManifestShowAll {}, None)],
    default_command_executor: Some(&ManifestShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    extra_flags: vec![],
  });
}

const API_VERSION: &str = "apiVersion";
const _CONFIGURATION: &str = "configuration";
const _CONTACT: &str = "contact";
const _DESCRIPTION: &str = "description";
const ID: &str = "id";
const _KIND: &str = "kind";
const _MORE_INFO: &str = "moreInfo";
const _NAME: &str = "name";
const _RESOURCES: &str = "resources";
const _VENDOR: &str = "vendor";
const VERSION: &str = "version";

struct ManifestListAll {}

#[async_trait]
impl CommandExecutor for ManifestListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let manifests: Vec<AppCatalogManifest> = dsh_api_client.get_app_catalog_manifests().await?;

    for (index, manifest) in manifests.iter().enumerate() {
      let payload = &manifest.payload;
      let des = from_str::<Value>(payload.as_str()).unwrap();
      let object = des.as_object().unwrap();

      // println!("--------------------------------------- {}", index);
      // println!("api version    {}", object.get(API_VERSION).unwrap());
      // println!("configuration  {}", object.get(CONFIGURATION).unwrap());
      // println!("contact        {}", object.get(CONTACT).unwrap());
      // println!("description    {}", object.get(DESCRIPTION).unwrap());
      println!(
        "{}  {}  {}  {}",
        index,
        object.get(ID).unwrap().as_str().unwrap(),
        object.get(VERSION).unwrap().as_str().unwrap(),
        object.get(API_VERSION).unwrap().as_str().unwrap()
      );
      // println!("kind           {}", object.get(KIND).unwrap());
      // println!("more info      {}", object.get(MORE_INFO).unwrap());
      // println!("name           {}", object.get(NAME).unwrap());
      // println!("resources      {}", object.get(RESOURCES).unwrap());
      // println!("vendor         {}", object.get(VENDOR).unwrap());
      // println!("version        {}", object.get(VERSION).unwrap());
    }
    Ok(())
  }
}

struct ManifestListConfiguration {}

#[async_trait]
impl CommandExecutor for ManifestListConfiguration {
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

struct ManifestListIds {}

#[async_trait]
impl CommandExecutor for ManifestListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let id_versions_pairs: Vec<(String, Vec<String>)> = dsh_api_client.get_app_catalog_manifest_ids_with_versions().await?;
    for (id, _) in id_versions_pairs {
      println!("{}", id)
    }
    Ok(())
  }
}

struct ManifestShowAll {}

#[async_trait]
impl CommandExecutor for ManifestShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let app = dsh_api_client.get_app_configuration(manifest_id.as_str()).await?;
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
