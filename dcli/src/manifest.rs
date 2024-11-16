use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use itertools::Itertools;
use lazy_static::lazy_static;

use dsh_api::types::AppCatalogManifest;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::formatter::print_vec;
use crate::formatters::list_table::ListTable;
use crate::formatters::manifest::{Manifest, MANIFEST_LABELS_LIST, MANIFEST_LABELS_SHOW};
use crate::formatters::show_table::ShowTable;
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

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

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, MANIFEST_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, MANIFEST_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref MANIFEST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List manifests".to_string(),
    command_long_about: Some("Lists all manifest files from the App Catalog.".to_string()),
    command_executors: vec![(FlagType::All, &ManifestListAll {}, None), (FlagType::Ids, &ManifestListIds {}, None),],
    default_command_executor: Some(&ManifestListAll {}),
    run_all_executors: true,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![]
  });
  pub static ref MANIFEST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show manifest configuration".to_string(),
    command_long_about: None,
    command_executors: vec![(FlagType::All, &ManifestShowAll {}, None)],
    default_command_executor: Some(&ManifestShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![]
  });
}

struct ManifestListAll {}

#[async_trait]
impl CommandExecutor for ManifestListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all app catalog manifests");
    }
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.dsh_api_client.as_ref().unwrap().list_app_catalog_manifests().await?;
    let manifests = app_catalog_manifests.iter().map(|acm| Manifest::try_from(acm).unwrap()).collect::<Vec<_>>();
    let manifests_with_id = manifests.iter().map(|manifest| (manifest.manifest_id.clone(), manifest)).collect::<Vec<_>>();
    let manifests_grouped = manifests_with_id.clone().into_iter().into_group_map();
    let mut manifest_ids = manifests_grouped.keys().collect::<Vec<_>>();
    manifest_ids.sort();
    let mut table = ListTable::new(&MANIFEST_LABELS_LIST, context);
    for manifest_id in manifest_ids {
      let mut manifests: Vec<&Manifest> = manifests_grouped.get(manifest_id).unwrap().clone();
      manifests.sort_by_key(|m| m.version.clone());
      let mut first = true;
      for manifest in manifests {
        if first {
          table.value(manifest_id.as_str(), manifest);
          first = false;
        } else {
          table.value("", manifest);
        }
      }
    }
    table.print();
    Ok(false)
  }
}

struct ManifestListIds {}

#[async_trait]
impl CommandExecutor for ManifestListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all app catalog manifest ids");
    }
    print_vec(
      "manifest ids".to_string(),
      context
        .dsh_api_client
        .as_ref()
        .unwrap()
        .list_app_catalog_manifest_ids_with_versions()
        .await?
        .iter()
        .map(|p| p.0.clone())
        .collect::<Vec<String>>(),
      context,
    );
    Ok(false)
  }
}

struct ManifestShowAll {}

#[async_trait]
impl CommandExecutor for ManifestShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for app catalog manifest '{}'", manifest_id);
    }
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.dsh_api_client.as_ref().unwrap().list_app_catalog_manifests().await?;
    let manifests = app_catalog_manifests.iter().map(|acm| Manifest::try_from(acm).unwrap()).collect::<Vec<_>>();
    let manifests_with_id = manifests.iter().map(|manifest| (manifest.manifest_id.clone(), manifest)).collect::<Vec<_>>();
    let manifests_grouped = manifests_with_id.clone().into_iter().into_group_map();
    let mut manifests: Vec<&Manifest> = manifests_grouped.get(manifest_id.as_str()).unwrap().clone();
    manifests.sort_by_key(|m| m.version.clone());
    let table = ShowTable::new(manifest_id.as_str(), *manifests.last().unwrap(), &MANIFEST_LABELS_SHOW, context);
    table.print();
    Ok(false)
  }
}
