use crate::formatters::formatter::{Label, SubjectFormatter};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::app_manifest::Manifest;
use dsh_api::types::AppCatalogManifest;
use dsh_api::DshApiError;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::str::FromStr;

use crate::arguments::{manifest_id_argument, version_argument, VERSION_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, EXPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::subject::{Requirements, Subject};
use crate::version::Version;
use crate::DshCliResult;

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

  fn subject_command_about(&self) -> String {
    "Show App Catalog manifests.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show the manifest files for the apps in the DSH App Catalog.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      EXPORT_COMMAND => Some(MANIFEST_EXPORT_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(MANIFEST_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(MANIFEST_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &MANIFEST_CAPABILITIES
  }
}

lazy_static! {
  static ref MANIFEST_EXPORT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(EXPORT_COMMAND, None, &ManifestExport {}, "Export manifest")
      .set_long_about("Export a manifest file from the App Catalog.")
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(version_argument().required(true))
  );
  static ref MANIFEST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ManifestListAll {}, "List manifests")
      .set_long_about("Lists all manifest files from the App Catalog.")
      .add_command_executor(FlagType::Ids, &ManifestListIds {}, None)
  );
  static ref MANIFEST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ManifestShowAll {}, "Show manifest configuration")
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(version_argument())
  );
  static ref MANIFEST_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![MANIFEST_EXPORT_CAPABILITY.as_ref(), MANIFEST_LIST_CAPABILITY.as_ref(), MANIFEST_SHOW_CAPABILITY.as_ref()];
}

struct ManifestExport {}

#[async_trait]
impl CommandExecutor for ManifestExport {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let manifest_version = matches.get_one::<String>(VERSION_ARGUMENT).unwrap();
    context.print_explanation(format!("export app catalog manifest '{}' version {}", manifest_id, manifest_version));
    let start_instant = context.now();
    let raw_manifest = context.client_unchecked().get_raw_manifest(&manifest_id, manifest_version).await;
    context.print_execution_time(start_instant);
    match raw_manifest {
      Ok(raw_manifest) => {
        context.print(raw_manifest);
        Ok(())
      }
      Err(DshApiError::NotFound) => {
        context.print_outcome(format!("manifest '{}' not found", manifest_id));
        Ok(())
      }
      Err(e) => Err(e.to_string()),
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(Some(OutputFormat::Json))
  }
}

struct ManifestListAll {}

#[async_trait]
impl CommandExecutor for ManifestListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all app catalog manifests");
    let start_instant = context.now();
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.client_unchecked().get_appcatalog_manifests().await?;
    context.print_execution_time(start_instant);
    let manifests = app_catalog_manifests.iter().map(|acm| Manifest::try_from(acm).unwrap()).collect::<Vec<_>>();
    let manifests_with_id = manifests.iter().map(|manifest| (manifest.id.clone(), manifest)).collect::<Vec<_>>();
    let manifests_grouped = manifests_with_id.clone().into_iter().into_group_map();
    let mut manifest_ids = manifests_grouped.keys().collect::<Vec<_>>();
    manifest_ids.sort();
    let mut formatter = ListFormatter::new(&MANIFEST_LABELS_LIST, None, context);
    for manifest_id in manifest_ids {
      let mut manifests: Vec<&Manifest> = manifests_grouped.get(manifest_id).unwrap().clone();
      manifests.sort_by(|manifest_1, manifest_2| manifest_2.version.cmp(&manifest_1.version));
      for manifest in manifests {
        formatter.push_target_id_value(manifest_id.clone(), manifest);
      }
    }
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api_multiple(true, true, None)
  }
}

struct ManifestListWithResources {}

#[async_trait]
impl CommandExecutor for ManifestListWithResources {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all app catalog manifests with resources");
    let start_instant = context.now();
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.client_unchecked().get_appcatalog_manifests().await?;
    context.print_execution_time(start_instant);
    let manifests = app_catalog_manifests.iter().map(|acm| Manifest::try_from(acm).unwrap()).collect::<Vec<_>>();
    let manifests_with_id = manifests.iter().map(|manifest| (manifest.id.clone(), manifest)).collect::<Vec<_>>();
    let manifests_grouped = manifests_with_id.clone().into_iter().into_group_map();
    let mut manifest_ids = manifests_grouped.keys().collect::<Vec<_>>();
    manifest_ids.sort();

    let mut formatter = ListFormatter::new(&MANIFEST_LABELS_LIST, None, context);
    for manifest_id in manifest_ids {
      let mut manifests: Vec<&Manifest> = manifests_grouped.get(manifest_id).unwrap().clone();
      // let highest_version = manifests.iter().max_by_key(|m| m.version)

      manifests.sort_by(|manifest_1, manifest_2| manifest_2.version.cmp(&manifest_1.version));
      for manifest in manifests {
        formatter.push_target_id_value(manifest_id.clone(), manifest);
      }
    }
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api_multiple(true, true, None)
  }
}

struct ManifestListIds {}

#[async_trait]
impl CommandExecutor for ManifestListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all app catalog manifest ids");
    let start_instant = context.now();
    let manifest_ids: Vec<String> = context.client_unchecked().list_app_catalog_manifest_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("manifest id", context);
    formatter.push_target_ids(&manifest_ids);
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api_multiple(true, true, Some(OutputFormat::Plain))
  }
}

struct ManifestShowAll {}

#[async_trait]
impl CommandExecutor for ManifestShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let version_argument = match matches.get_one::<String>(VERSION_ARGUMENT) {
      Some(argument) => Some(Version::from_str(argument)?),
      None => None,
    };
    match &version_argument {
      Some(version) => context.print_explanation(format!("show all parameters for app catalog manifest '{}' version {}", manifest_id, version)),
      None => context.print_explanation(format!("show all parameters for app catalog manifest '{}' latest version", manifest_id)),
    }
    let start_instant = context.now();
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.client_unchecked().get_appcatalog_manifests().await?;
    context.print_execution_time(start_instant);
    let manifests = app_catalog_manifests
      .iter()
      .map(|acm| Manifest::try_from(acm).unwrap())
      .filter(|manifest| manifest.id == manifest_id)
      .collect::<Vec<_>>();
    if manifests.is_empty() {
      context.print_outcome(format!("manifest '{}' not found", manifest_id));
    } else {
      match version_argument {
        Some(version) => match manifests.iter().find(|manifest| manifest.version == version.to_string()) {
          Some(manifest) => UnitFormatter::new(manifest_id, &MANIFEST_LABELS_SHOW, Some("manifest id"), context).print(manifest)?,
          None => context.print_outcome(format!("manifest '{}' has no version {}", manifest_id, version)),
        },
        None => {
          let highest_version = manifests.iter().max_by_key(|manifest| &manifest.version).unwrap();
          UnitFormatter::new(manifest_id, &MANIFEST_LABELS_SHOW, Some("manifest id"), context).print(highest_version)?;
        }
      }
    };
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api_multiple(true, true, None)
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ManifestLabel {
  _Configuration,
  Contact,
  Draft,
  LastModified,
  Name,
  Vendor,
  ManifestVersion,
  Target,
}

impl Label for ManifestLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::_Configuration => "configuration",
      Self::Target => "app",
      Self::Contact => "contact",
      Self::Draft => "draft",
      Self::LastModified => "last modified",
      Self::Name => "name",
      Self::Vendor => "vendor",
      Self::ManifestVersion => "version",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<ManifestLabel> for Manifest {
  fn value(&self, label: &ManifestLabel, target_id: &str) -> String {
    match label {
      ManifestLabel::_Configuration => "".to_string(),
      ManifestLabel::Contact => self.contact.clone(),
      ManifestLabel::Draft => self.draft.to_string(),
      ManifestLabel::LastModified => self.last_modified.clone(),
      ManifestLabel::Name => self.name.clone(),
      ManifestLabel::Vendor => self.vendor.clone(),
      ManifestLabel::ManifestVersion => self.version.to_string(),
      ManifestLabel::Target => target_id.to_string(),
    }
  }
}

pub static MANIFEST_LABELS_LIST: [ManifestLabel; 6] =
  [ManifestLabel::Target, ManifestLabel::ManifestVersion, ManifestLabel::Name, ManifestLabel::Draft, ManifestLabel::Vendor, ManifestLabel::LastModified];

pub static MANIFEST_LABELS_SHOW: [ManifestLabel; 7] =
  [ManifestLabel::Target, ManifestLabel::Contact, ManifestLabel::Draft, ManifestLabel::LastModified, ManifestLabel::Name, ManifestLabel::Vendor, ManifestLabel::ManifestVersion];
