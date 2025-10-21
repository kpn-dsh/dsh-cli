use crate::arguments::{manifest_id_argument, version_argument, VERSION_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, EXPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, MANUAL_COMMAND, MANUAL_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::formatter::{hashmap_to_table, hashmap_to_vec, vec_to_table, Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::app_manifest::{Manifest, Numerical, Property, Resource};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::version::Version;
use dsh_api::DshApiError;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

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
      MANUAL_COMMAND => Some(MANIFEST_MANUAL_CAPABILITY.as_ref()),
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
      .add_command_executor(FlagType::AllVersions, &ManifestListAllVersions {}, None)
      .add_command_executor(FlagType::Ids, &ManifestListIds {}, None)
  );
  static ref MANIFEST_MANUAL_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(MANUAL_COMMAND, Some(MANUAL_COMMAND_ALIAS), &ManifestManual {}, "Show manifest manual")
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(version_argument())
  );
  static ref MANIFEST_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ManifestShowAll {}, "Show manifest configuration")
      .add_target_argument(manifest_id_argument().required(true))
      .add_target_argument(version_argument())
      .add_command_executor(FlagType::AllVersions, &ManifestShowAllVersions {}, None)
      .add_filter_flag(FilterFlagType::Complete, None)
  );
  static ref MANIFEST_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![MANIFEST_EXPORT_CAPABILITY.as_ref(), MANIFEST_LIST_CAPABILITY.as_ref(), MANIFEST_MANUAL_CAPABILITY.as_ref(), MANIFEST_SHOW_CAPABILITY.as_ref()];
}

struct ManifestExport {}

#[async_trait]
impl CommandExecutor for ManifestExport {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let manifest_version = matches.get_one::<String>(VERSION_ARGUMENT).map(|v| Version::from_str(v.as_str())).unwrap()?;
    context.print_explanation(format!("export app catalog manifest '{}' version {}", manifest_id, manifest_version));
    let start_instant = context.now();
    let raw_manifest = client.get_raw_manifest(&manifest_id, &manifest_version).await;
    context.print_execution_time(start_instant);
    match raw_manifest {
      Ok(raw_manifest) => {
        context.print(raw_manifest);
        Ok(())
      }
      Err(DshApiError::NotFound(None)) => Err(format!("manifest '{}' not found", manifest_id)),
      Err(e) => Err(e.to_string()),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ManifestListAll {}

#[async_trait]
impl CommandExecutor for ManifestListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all app catalog manifests");
    let start_instant = context.now();
    let manifests_by_id: Vec<(String, Vec<(Version, Manifest)>)> = client.list_app_catalog_manifests().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&MANIFEST_LABELS_LIST, None, context);
    for (manifest_id, manifests) in &manifests_by_id {
      formatter.push_target_id_value(manifest_id.clone(), &manifests.last().unwrap().1);
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ManifestListAllVersions {}

#[async_trait]
impl CommandExecutor for ManifestListAllVersions {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all versions of all app catalog manifests");
    let start_instant = context.now();
    let manifests_by_id: Vec<(String, Vec<(Version, Manifest)>)> = client.list_app_catalog_manifests().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&MANIFEST_LABELS_LIST, None, context);
    for (manifest_id, manifests) in &manifests_by_id {
      for (_, manifest) in manifests {
        formatter.push_target_id_value(manifest_id.clone(), manifest);
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ManifestListIds {}

#[async_trait]
impl CommandExecutor for ManifestListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all app catalog manifest ids");
    let start_instant = context.now();
    let manifest_ids: Vec<String> = client.list_app_catalog_manifest_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("manifest id", context);
    formatter.push_target_ids(&manifest_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ManifestManual {}

#[async_trait]
impl CommandExecutor for ManifestManual {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let version_argument = match matches.get_one::<String>(VERSION_ARGUMENT) {
      Some(version) => Some(Version::from_str(version.as_str())?),
      None => None,
    };
    match &version_argument {
      Some(version) => context.print_explanation(format!("show configuration manual for app catalog manifest '{}', version {}", manifest_id, version)),
      None => context.print_explanation(format!("show configuration manual for app catalog manifest '{}', latest version", manifest_id)),
    }
    let start_instant = context.now();
    context.print_execution_time(start_instant);
    let manifest: Manifest = match version_argument {
      Some(version_argument) => client.get_app_catalog_manifest(manifest_id.as_str(), &version_argument).await?,
      None => client.get_app_catalog_manifest_latest(manifest_id.as_str()).await?,
    };
    println!("{}", manifest.name);
    if let Some(description) = manifest.description {
      println!("{}", description);
    }
    println!("{}:{}", manifest.id, manifest.version);
    if let Some(more_info) = manifest.more_info {
      println!();
      println!("{}", termimad::text(more_info.as_str()));
    }
    if let Some(configuration) = &manifest.configuration {
      let mut property_ids = configuration.properties.iter().map(|(id, _)| id.to_string()).collect_vec();
      property_ids.sort();
      let mut formatter = ListFormatter::new(&PROPERTY_LABELS_LIST, None, context);
      for property_id in property_ids {
        let property = configuration.properties.get(&property_id).unwrap();
        formatter.push_target_id_value(property_id, property);
      }
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ManifestShowAll {}

#[async_trait]
impl CommandExecutor for ManifestShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    let version_argument = matches.get_one::<String>(VERSION_ARGUMENT).map(|version| Version::from_str(version)).transpose()?;
    let complete = matches.get_flag(FilterFlagType::Complete.id());
    match &version_argument {
      Some(version) => context.print_explanation(format!("show all parameters for app catalog manifest '{}', version {}", manifest_id, version)),
      None => context.print_explanation(format!("show all parameters for app catalog manifest '{}', latest version", manifest_id)),
    }
    let start_instant = context.now();
    let manifests: Vec<(Version, Manifest)> = client.get_app_catalog_manifests(manifest_id.as_str()).await?;
    context.print_execution_time(start_instant);
    if manifests.is_empty() {
      return Err(format!("manifest '{}' not found", manifest_id));
    } else {
      let labels: &[ManifestLabel] = if complete { &MANIFEST_LABELS_SHOW_FULL } else { &MANIFEST_LABELS_SHOW };
      match version_argument {
        Some(version_argument) => match manifests.iter().find(|(version, _)| version == &version_argument) {
          Some((_, manifest)) => UnitFormatter::new(manifest_id, labels, Some("manifest id"), context).print(manifest, None)?,
          None => return Err(format!("manifest '{}' has no version {}", manifest_id, version_argument)),
        },
        None => UnitFormatter::new(manifest_id, labels, Some("manifest id"), context).print(&manifests.last().unwrap().clone().1, None)?,
      }
    };
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ManifestShowAllVersions {}

#[async_trait]
impl CommandExecutor for ManifestShowAllVersions {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let manifest_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("list all versions of app catalog manifest '{}'", manifest_id));
    let start_instant = context.now();
    let manifests_by_id: Vec<(Version, Manifest)> = client.get_app_catalog_manifests(manifest_id.as_str()).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&MANIFEST_LABELS_LIST, None, context);
    for (_, manifest) in &manifests_by_id {
      formatter.push_target_id_value(manifest_id.clone(), manifest);
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ManifestLabel {
  ApiVersion,
  Configuration,
  Contact,
  Description,
  Draft,
  Id,
  Kind,
  LastModified,
  ManifestVersion,
  MoreInfo,
  Name,
  Resources,
  Vendor,
}

impl Label for ManifestLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::ApiVersion => "api version",
      Self::Configuration => "configuration",
      Self::Contact => "contact",
      Self::Description => "description",
      Self::Draft => "draft",
      Self::Id => "app",
      Self::Kind => "kind",
      Self::LastModified => "last modified",
      Self::ManifestVersion => "version",
      Self::MoreInfo => "more info",
      Self::Name => "name",
      Self::Resources => "resources",
      Self::Vendor => "vendor",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Id)
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum PropertyLabel {
  _Default,
  Description,
  Enum,
  Id,
  _Type,
}

impl Label for PropertyLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::_Default => "default",
      Self::Description => "description",
      Self::Enum => "value",
      Self::Id => "property",
      Self::_Type => "type",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Id)
  }
}

impl SubjectFormatter<PropertyLabel> for Property {
  fn value(&self, label: &PropertyLabel, target_id: &str) -> String {
    match label {
      PropertyLabel::_Default => self.default.clone().map(|default| default.to_string()).unwrap_or_default(),
      PropertyLabel::Description => self.description.to_string(),
      PropertyLabel::Enum => match &self.r#enum {
        Some(enum_values) => match &self.default {
          Some(default_value) => enum_values
            .iter()
            .map(|enum_value| if enum_value == default_value { format!("{}*", enum_value) } else { enum_value.to_string() })
            .join(", "),
          None => enum_values.iter().join(", "),
        },
        None => match &self.default {
          Some(default_value) => format!("default: {}", default_value),
          None => "".to_string(),
        },
      },
      PropertyLabel::Id => target_id.to_string(),
      PropertyLabel::_Type => self.r#type.to_string(),
    }
  }
}

pub static PROPERTY_LABELS_LIST: [PropertyLabel; 3] = [PropertyLabel::Id, PropertyLabel::Description, PropertyLabel::Enum];

fn property_to_string(property: &Property) -> String {
  format!(
    "{}{}: {}{}",
    property.r#type,
    if property.default.is_some() { "" } else { "*" },
    property.description,
    match property.r#enum {
      Some(ref enumeration) => format!(
        "\noptions: {}",
        enumeration
          .iter()
          .map(|value| if property.default.clone().is_some_and(|ref default| default == value) { format!("{}*", value) } else { value.to_string() })
          .join(", ")
      ),
      None => match property.default {
        Some(ref default) =>
          if default.is_empty() {
            "".to_string()
          } else {
            format!("\ndefault: {}", default)
          },
        None => "".to_string(),
      },
    },
  )
}

fn resource_to_key(resource: &Resource) -> String {
  match resource {
    Resource::Application(application) => format!("application: {}", application.name),
    Resource::Bucket(bucket) => format!("bucket: {}", bucket.name),
    Resource::Certificate(certificate) => format!("certificate: {}", certificate),
    Resource::Database(database) => format!("database: {}", database.name),
    Resource::Secret(secret) => format!("secret: {}", secret),
    Resource::Topic(topic) => format!("topic: {}", topic.name),
    Resource::Vhost(vhost) => format!("vhost: {}", vhost),
    Resource::Volume(volume) => format!("volume: {}", volume.name),
  }
}

fn resource_to_strings(resource: &Resource) -> Vec<String> {
  match resource {
    Resource::Application(application) => {
      let mut strings = vec![];
      strings.push(format!("image: {}", application.image));
      strings.push(format!("cpus: {}", numerical_to_string(&application.cpus)));
      strings.push(format!("mem: {} (MB)", numerical_to_string(&application.mem)));
      strings.push(format!("instances: {}", numerical_to_string(&application.instances)));
      strings.push(format!("single instance: {}", application.single_instance));
      strings.push(format!("needs token: {}", application.needs_token));
      if let Some(ref secrets) = application.secrets {
        for secret in secrets {
          strings.push(format!("secret: {}", secret.name));
        }
      }
      if let Some(ref exposed_ports) = application.exposed_ports {
        for (port, exposed_port) in exposed_ports {
          strings.push(format!(
            "port: {}, vhost: {}{}{}",
            port,
            exposed_port.vhost,
            exposed_port.auth.clone().map(|ref auth| format!(", auth: {}", auth)).unwrap_or_default(),
            exposed_port.tls.clone().map(|ref tls| format!(", tls: {}", tls)).unwrap_or_default()
          ));
        }
      }
      if let Some(ref metrics) = application.metrics {
        strings.push(format!("metrics: {}:{}", metrics.path, metrics.port));
      }
      strings.push(format!("user: {}", application.user));
      if let Some(ref image_console) = application.image_console {
        strings.push(format!("image console: {}", image_console));
      }
      if !application.env.is_empty() {
        hashmap_to_vec(&application.env).into_iter().for_each(|a| strings.push(a));
      }
      strings
    }
    Resource::Bucket(bucket) => {
      let mut strings = vec![];
      strings.push(format!("encrypted: {}", &bucket.encrypted));
      strings.push(format!("versioned: {}", &bucket.versioned));
      strings
    }
    Resource::Certificate(certificate) => vec![certificate.to_string()],
    Resource::Database(database) => {
      let mut strings = vec![];
      strings.push(format!("cpus: {}", numerical_to_string(&database.cpus),));
      strings.push(format!("mem: {}", numerical_to_string(&database.mem),));
      strings.push(format!("instances: {}", numerical_to_string(&database.instances),));
      strings.push(format!("version: {}", &database.version));
      strings.push(format!("extensions: {}", database.extensions.join(", ")));
      strings.push(format!("snapshot interval: {}", numerical_to_string(&database.snapshot_interval)));
      strings.push(format!("volume size: {}", numerical_to_string(&database.volume_size),));
      strings
    }
    Resource::Secret(secret) => vec![secret.to_string()],
    Resource::Topic(topic) => {
      let mut strings = vec![];
      strings.push(format!("partitions: {}", topic.partitions));
      strings.push(format!("replication factor: {}", topic.replication_factor));
      if let Some(ref kafka_properties) = topic.kafka_properties {
        strings.push(hashmap_to_table(kafka_properties));
      }
      strings
    }
    Resource::Vhost(vhost) => vec![vhost.to_string()],
    Resource::Volume(volume) => {
      let mut strings = vec![];
      strings.push(format!("size: {} (GB)", numerical_to_string(&volume.size)));
      strings
    }
  }
}

fn numerical_to_string(numerical: &Numerical) -> String {
  match numerical {
    Numerical::Float(float) => float.to_string(),
    Numerical::Integer(integer) => integer.to_string(),
    Numerical::Template(template) => template.to_string(),
  }
}

impl SubjectFormatter<ManifestLabel> for Manifest {
  fn value(&self, label: &ManifestLabel, target_id: &str) -> String {
    match label {
      ManifestLabel::ApiVersion => self.api_version.clone().unwrap_or_default(),
      ManifestLabel::Configuration => match self.configuration {
        Some(ref configuration) => hashmap_to_table(
          &configuration
            .properties
            .iter()
            .map(|(key, property)| (key, property_to_string(property)))
            .collect::<HashMap<_, _>>(),
        ),
        None => "".to_string(),
      },
      ManifestLabel::Contact => self.contact.clone(),
      ManifestLabel::Description => self.description.clone().unwrap_or_default(),
      ManifestLabel::Draft => self.draft.to_string(),
      ManifestLabel::Id => target_id.to_string(),
      ManifestLabel::Kind => self.kind.clone().unwrap_or_default(),
      ManifestLabel::LastModified => self.last_modified.clone(),
      ManifestLabel::ManifestVersion => self.version.to_string(),
      ManifestLabel::MoreInfo => self
        .more_info
        .clone()
        .map(|more_info| termimad::text(more_info.as_str()).to_string())
        .unwrap_or_default(),
      ManifestLabel::Name => self.name.clone(),
      ManifestLabel::Resources => vec_to_table(
        &self
          .resources
          .values()
          .map(|resource| (resource_to_key(resource), resource_to_strings(resource)))
          .collect_vec(),
      ),
      ManifestLabel::Vendor => self.vendor.clone(),
    }
  }
}

pub static MANIFEST_LABELS_LIST: [ManifestLabel; 6] =
  [ManifestLabel::Id, ManifestLabel::ManifestVersion, ManifestLabel::Name, ManifestLabel::Draft, ManifestLabel::Vendor, ManifestLabel::LastModified];

pub static MANIFEST_LABELS_SHOW: [ManifestLabel; 9] = [
  ManifestLabel::Id,
  ManifestLabel::Name,
  ManifestLabel::Draft,
  ManifestLabel::Description,
  ManifestLabel::LastModified,
  ManifestLabel::Vendor,
  ManifestLabel::ManifestVersion,
  ManifestLabel::Configuration,
  ManifestLabel::Resources,
];

pub static MANIFEST_LABELS_SHOW_FULL: [ManifestLabel; 13] = [
  ManifestLabel::Id,
  ManifestLabel::Name,
  ManifestLabel::Kind,
  ManifestLabel::Draft,
  ManifestLabel::ApiVersion,
  ManifestLabel::Description,
  ManifestLabel::Contact,
  ManifestLabel::LastModified,
  ManifestLabel::Vendor,
  ManifestLabel::ManifestVersion,
  ManifestLabel::Configuration,
  ManifestLabel::Resources,
  ManifestLabel::MoreInfo,
];
