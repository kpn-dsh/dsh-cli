use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use clap::ArgMatches;
use dsh_api::app_manifest::CONTACT;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::de::from_str;
use serde_json::Value;

use crate::formatters::formatter::{Label, SubjectFormatter};

use dsh_api::types::AppCatalogManifest;

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
      .add_target_argument(version_argument())
  );
  static ref MANIFEST_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ManifestListAll {}, "List manifests")
      .set_long_about("Lists all manifest files from the App Catalog.")
      .add_command_executor(FlagType::Ids, &ManifestListIds {}, None)
  );
  static ref MANIFEST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ManifestShowAll {}, "Show manifest configuration").add_target_argument(manifest_id_argument().required(true))
  );
  static ref MANIFEST_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![MANIFEST_EXPORT_CAPABILITY.as_ref(), MANIFEST_LIST_CAPABILITY.as_ref(), MANIFEST_SHOW_CAPABILITY.as_ref()];
}

struct ManifestExport {}

#[async_trait]
impl CommandExecutor for ManifestExport {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let version_argument = matches.get_one::<String>(VERSION_ARGUMENT);
    if let Some(version) = version_argument {
      context.print_explanation(format!("show app catalog manifest '{}' version {}", manifest_id, version));
    } else {
      context.print_explanation(format!("show app catalog manifests '{}'", manifest_id));
    }
    let start_instant = context.now();
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.client_unchecked().get_appcatalog_manifests().await?;
    context.print_execution_time(start_instant);
    let manifests = app_catalog_manifests
      .iter()
      .map(|acm| Manifest::try_from(acm).unwrap())
      .filter(|manifest| manifest.manifest_id == manifest_id)
      .collect::<Vec<_>>();
    if manifests.is_empty() {
      context.print_outcome(format!("manifest '{}' not found", manifest_id));
    } else {
      match version_argument {
        Some(version) => match manifests.iter().find(|manifest| manifest.version == *version) {
          Some(manifest) => context.print_serializable(manifest.clone().json),
          None => context.print_outcome(format!("manifest '{}' has no version {}", manifest_id, version)),
        },
        None => context.print_serializable(manifests),
      }
    }
    Ok(())
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
    let manifests_with_id = manifests.iter().map(|manifest| (manifest.manifest_id.clone(), manifest)).collect::<Vec<_>>();
    let manifests_grouped = manifests_with_id.clone().into_iter().into_group_map();
    let mut manifest_ids = manifests_grouped.keys().collect::<Vec<_>>();
    manifest_ids.sort();
    let mut formatter = ListFormatter::new(&MANIFEST_LABELS_LIST, None, context);
    for manifest_id in manifest_ids {
      let mut manifests: Vec<&Manifest> = manifests_grouped.get(manifest_id).unwrap().clone();
      manifests.sort_by_key(|m| m.version.clone());
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
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for app catalog manifest '{}'", manifest_id));
    let start_instant = context.now();
    let app_catalog_manifests: Vec<AppCatalogManifest> = context.client_unchecked().get_appcatalog_manifests().await?;
    context.print_execution_time(start_instant);
    let manifests = app_catalog_manifests.iter().map(|acm| Manifest::try_from(acm).unwrap()).collect::<Vec<_>>();
    let manifests_with_id = manifests.iter().map(|manifest| (manifest.manifest_id.clone(), manifest)).collect::<Vec<_>>();
    let manifests_grouped = manifests_with_id.clone().into_iter().into_group_map();
    let mut manifests: Vec<&Manifest> = manifests_grouped.get(&manifest_id).unwrap().clone();
    manifests.sort_by_key(|m| m.version.clone());
    UnitFormatter::new(manifest_id, &MANIFEST_LABELS_SHOW, Some("manifest id"), context).print(*manifests.last().unwrap())
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
  Version,
  Target,
}

const _CONFIGURATION: &str = "configuration";
const ID: &str = "id";
const NAME: &str = "name";
const VENDOR: &str = "vendor";
const VERSION: &str = "version";

impl Label for ManifestLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::_Configuration => _CONFIGURATION,
      Self::Target => "app",
      Self::Contact => "contact",
      Self::Draft => "draft",
      Self::LastModified => "last modified",
      Self::Name => NAME,
      Self::Vendor => VENDOR,
      Self::Version => VERSION,
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct Manifest {
  pub manifest_id: String,
  pub contact: String,
  pub draft: bool,
  pub last_modified: String,
  pub name: String,
  pub vendor: String,
  pub version: String,
  pub json: Value,
}

impl TryFrom<&AppCatalogManifest> for Manifest {
  type Error = String;

  fn try_from(value: &AppCatalogManifest) -> Result<Self, Self::Error> {
    match from_str::<Value>(&value.payload) {
      Ok(payload_value) => match payload_value.as_object() {
        Some(payload_object) => Ok(Manifest {
          manifest_id: payload_object.get(&ID.to_string()).unwrap().as_str().unwrap().to_string(),
          contact: payload_object.get(&CONTACT.to_string()).unwrap().as_str().unwrap().to_string(),
          draft: value.draft,
          last_modified: Utc.timestamp_opt(value.last_modified as i64 / 1000, 0).unwrap().to_string(),
          name: payload_object.get(&NAME.to_string()).unwrap().as_str().unwrap().to_string(),
          vendor: payload_object.get(&VENDOR.to_string()).unwrap().as_str().unwrap().to_string(),
          version: payload_object.get(&VERSION.to_string()).unwrap().as_str().unwrap().to_string(),
          json: payload_value,
        }),
        None => Err("".to_string()),
      },
      Err(_) => Err("".to_string()),
    }
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
      ManifestLabel::Version => self.version.clone(),
      ManifestLabel::Target => target_id.to_string(),
    }
  }
}

pub static MANIFEST_LABELS_LIST: [ManifestLabel; 6] =
  [ManifestLabel::Target, ManifestLabel::Version, ManifestLabel::Name, ManifestLabel::Draft, ManifestLabel::Vendor, ManifestLabel::LastModified];

pub static MANIFEST_LABELS_SHOW: [ManifestLabel; 7] =
  [ManifestLabel::Target, ManifestLabel::Contact, ManifestLabel::Draft, ManifestLabel::LastModified, ManifestLabel::Name, ManifestLabel::Vendor, ManifestLabel::Version];
