use crate::arguments::query_argument;
use crate::capability::{Capability, CommandExecutor, FIND_COMMAND, FIND_COMMAND_PAIR, LIST_COMMAND, LIST_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::query_processor::{DummyQueryProcessor, ExactMatchQueryProcessor, QueryProcessor, RegexQueryProcessor};
use dsh_api::types::Application;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

pub(crate) struct ImageSubject {}

const IMAGE_SUBJECT_TARGET: &str = "image";

lazy_static! {
  pub static ref IMAGE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ImageSubject {});
}

#[async_trait]
impl Subject for ImageSubject {
  fn subject(&self) -> &'static str {
    IMAGE_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show image usage.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show which DSH components use an image.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("i")
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::new(true, None)
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      FIND_COMMAND => Some(IMAGE_FIND_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(IMAGE_LIST_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &IMAGE_CAPABILITIES
  }
}

lazy_static! {
  static ref IMAGE_FIND_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(FIND_COMMAND_PAIR, "Find used images")
      .set_long_about("Find all applications and/or apps that use a given Harbor image.")
      .set_default_command_executor(&ImageFind {})
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search in all started applications.".to_string())),
        (FilterFlagType::Stopped, Some("Search in all stopped applications.".to_string()))
      ])
      .add_target_argument(query_argument(None))
      .add_modifier_flag(ModifierFlagType::Regex, None)
  );
  static ref IMAGE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List images")
      .set_long_about(
        "Lists all images that are deployed in at least one application. \
        This will also include applications that are stopped \
        (deployed with 0 instances)."
      )
      .set_default_command_executor(&ImageListAll {})
      .set_run_all_executors(true)
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search all started applications.".to_string())),
        (FilterFlagType::Stopped, Some("Search all stopped applications.".to_string()))
      ])
  );
  static ref IMAGE_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![IMAGE_FIND_CAPABILITY.as_ref(), IMAGE_LIST_CAPABILITY.as_ref()];
}

struct ImageFind {}

#[async_trait]
impl CommandExecutor for ImageFind {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let image_query = target.unwrap_or_else(|| unreachable!());
    let query_processor: &dyn QueryProcessor =
      if matches.get_flag(ModifierFlagType::Regex.id()) { &RegexQueryProcessor::create(image_query.as_str())? } else { &ExactMatchQueryProcessor::create(image_query.as_str())? };
    context.print_explanation(format!("find images that {}", query_processor.describe()));
    let start_instant = Instant::now();
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    context.print_execution_time(start_instant);
    list_images(applications, query_processor, matches, context)?;
    Ok(())
  }
}

struct ImageListAll {}

#[async_trait]
impl CommandExecutor for ImageListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all images used in applications");
    let start_instant = Instant::now();
    let applications = context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    context.print_execution_time(start_instant);
    list_images(applications, &DummyQueryProcessor::create()?, matches, context)?;
    Ok(())
  }
}

fn list_images(applications: HashMap<String, Application>, query_processor: &dyn QueryProcessor, matches: &ArgMatches, context: &Context) -> Result<(), String> {
  let (include_started, include_stopped) = include_started_stopped(matches);
  let mut applications = applications.iter().collect::<Vec<_>>();
  applications.sort_by(|(application_id_a, _), (application_id_b, _)| application_id_a.cmp(application_id_b));
  let mut images: HashMap<String, Vec<ImageUsage>> = HashMap::new();
  for (application_id, application) in applications {
    if (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped) {
      let image = application.image.clone();
      let (registry, image) = parse_image_string(&image)?;
      images
        .entry(image)
        .or_default()
        .push(ImageUsage::new(registry, application_id.to_string(), application.instances));
    }
  }
  let mut images: Vec<(String, Vec<ImageUsage>)> = images.into_iter().collect::<Vec<_>>();
  images.sort_by(|(image_a, _), (image_b, _)| image_a.cmp(image_b));
  let mut formatter = ListFormatter::new(&IMAGE_USAGE_LABELS, None, context);
  for (image, image_usages) in &images {
    if let Some(image_parts) = query_processor.matching_parts(image) {
      let mut first = true;
      for image_usage in image_usages {
        if first {
          formatter.push_target_id_value(context.parts_to_string_stdout(&image_parts), image_usage);
        } else {
          formatter.push_target_id_value("".to_string(), image_usage);
        }
        first = false;
      }
    }
  }
  if formatter.is_empty() {
    context.print_outcome("no matches found in applications");
  } else {
    formatter.print()?;
  }
  Ok(())
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum ImageUsageLabel {
  Application,
  Image,
  Instances,
  Registry,
}

impl Label for ImageUsageLabel {
  fn as_str(&self) -> &str {
    match self {
      ImageUsageLabel::Application => "application_id",
      ImageUsageLabel::Image => "image",
      ImageUsageLabel::Instances => "#",
      ImageUsageLabel::Registry => "registry",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Image)
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
struct ImageUsage {
  registry: String,
  application_id: String,
  instances: u64,
}

impl ImageUsage {
  fn new(registry: String, application_id: String, instances: u64) -> Self {
    Self { registry, application_id, instances }
  }
}

impl SubjectFormatter<ImageUsageLabel> for ImageUsage {
  fn value(&self, label: &ImageUsageLabel, target_id: &str) -> String {
    match label {
      ImageUsageLabel::Application => self.application_id.clone(),
      ImageUsageLabel::Image => target_id.to_string(),
      ImageUsageLabel::Instances => self.instances.to_string(),
      ImageUsageLabel::Registry => self.registry.to_string(),
    }
  }

  fn target_label(&self) -> Option<ImageUsageLabel> {
    Some(ImageUsageLabel::Image)
  }
}

static IMAGE_USAGE_LABELS: [ImageUsageLabel; 4] = [ImageUsageLabel::Image, ImageUsageLabel::Registry, ImageUsageLabel::Application, ImageUsageLabel::Instances];

lazy_static! {
  static ref APP_CATALOG_IMAGE_REGEX: Regex =
    Regex::new(r"APPCATALOG_REGISTRY/dsh-appcatalog/tenant/([a-z0-9-_]+)/([0-9]+)/([0-9]+)/(release|draft)/(klarrio|kpn)/([a-zA-Z0-9-_:.]+)").unwrap();
  static ref REGISTRY_IMAGE_REGEX: Regex = Regex::new(r"registry.cp.kpn-dsh.com/([a-z0-9-_]+)/([a-zA-Z0-9-_:.]+)").unwrap();
}

/// # Parses an image string
///
/// # Returns
/// When the provided string is valid, the method returns a 2-tuple containing:
/// * registry of the image
/// * image id
// TODO Move to dsh-api
pub(crate) fn parse_image_string(image_string: &str) -> Result<(String, String), String> {
  match APP_CATALOG_IMAGE_REGEX.captures(image_string) {
    Some(app_catalog_captures) => Ok((
      format!(
        "app:{}:{}",
        app_catalog_captures.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
        app_catalog_captures.get(5).map(|m| m.as_str().to_string()).unwrap_or_default()
      ),
      app_catalog_captures.get(6).map(|m| m.as_str().to_string()).unwrap_or_default(),
    )),
    None => match REGISTRY_IMAGE_REGEX.captures(image_string) {
      Some(registry_captures) => Ok(("registry".to_string(), registry_captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default())),
      None => Err(format!("unrecognized image string {}", image_string)),
    },
  }
}

#[test]
fn test_app_catalog_image_draft_kpn() {
  const APP_CATALOG_IMAGE: &str = "APPCATALOG_REGISTRY/dsh-appcatalog/tenant/greenbox-dev/1903/1903/draft/kpn/schema-store-proxy:0.2.3-0";
  assert_eq!(
    parse_image_string(APP_CATALOG_IMAGE).unwrap(),
    ("app:draft:kpn".to_string(), "schema-store-proxy:0.2.3-0".to_string())
  );
}

#[test]
fn test_app_catalog_image_release_klarrio() {
  const APP_CATALOG_IMAGE: &str = "APPCATALOG_REGISTRY/dsh-appcatalog/tenant/greenbox-dev/1903/1903/release/klarrio/whoami:1.6.1";
  assert_eq!(
    parse_image_string(APP_CATALOG_IMAGE).unwrap(),
    ("app:release:klarrio".to_string(), "whoami:1.6.1".to_string())
  );
}

#[test]
fn test_registry_image() {
  const REGISTRY_IMAGE: &str = "registry.cp.kpn-dsh.com/greenbox-dev/cck-ingestor:0.0.18";
  assert_eq!(
    parse_image_string(REGISTRY_IMAGE).unwrap(),
    ("registry".to_string(), "cck-ingestor:0.0.18".to_string())
  );
}
