use crate::arguments::query_argument;
use crate::capability::{Capability, CommandExecutor, FIND_COMMAND, FIND_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS};
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
use dsh_api::application::parse_image_string;
use dsh_api::query_processor::{DummyQueryProcessor, ExactMatchQueryProcessor, QueryProcessor, RegexQueryProcessor};
use dsh_api::types::Application;
use lazy_static::lazy_static;
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
    CapabilityBuilder::new(FIND_COMMAND, Some(FIND_COMMAND_ALIAS), "Find used images")
      .set_long_about("Find all services and/or apps that use a given Harbor image.")
      .set_default_command_executor(&ImageFind {})
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search in all started services.".to_string())),
        (FilterFlagType::Stopped, Some("Search in all stopped services.".to_string()))
      ])
      .add_target_argument(query_argument(None).required(true))
      .add_modifier_flag(ModifierFlagType::Regex, None)
  );
  static ref IMAGE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List images")
      .set_long_about(
        "Lists all images that are deployed in at least one service. \
        This will also include services that are stopped \
        (deployed with 0 instances)."
      )
      .set_default_command_executor(&ImageListAll {})
      .set_run_all_executors(true)
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search all started services.".to_string())),
        (FilterFlagType::Stopped, Some("Search all stopped services.".to_string()))
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
    let services = context.client_unchecked().get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    list_images(services, query_processor, matches, context)?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ImageListAll {}

#[async_trait]
impl CommandExecutor for ImageListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all images used in services");
    let start_instant = Instant::now();
    let services = context.client_unchecked().get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    list_images(services, &DummyQueryProcessor::create()?, matches, context)?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

fn list_images(services: HashMap<String, Application>, query_processor: &dyn QueryProcessor, matches: &ArgMatches, context: &Context) -> Result<(), String> {
  let (include_started, include_stopped) = include_started_stopped(matches);
  let mut services = services.iter().collect::<Vec<_>>();
  services.sort_by(|(service_id_a, _), (service_id_b, _)| service_id_a.cmp(service_id_b));
  let mut images: HashMap<String, Vec<ImageUsage>> = HashMap::new();
  for (service_id, service) in services {
    if (service.instances > 0 && include_started) || (service.instances == 0 && include_stopped) {
      let image = service.image.clone();
      let (registry, image) = parse_image_string(&image)?;
      images
        .entry(image)
        .or_default()
        .push(ImageUsage::new(registry, service_id.to_string(), service.instances));
    }
  }
  let mut images: Vec<(String, Vec<ImageUsage>)> = images.into_iter().collect::<Vec<_>>();
  images.sort_by(|(image_a, _), (image_b, _)| image_a.cmp(image_b));
  let mut formatter = ListFormatter::new(&IMAGE_USAGE_LABELS, None, context);
  for (image, image_usages) in &images {
    if let Some(image_parts) = query_processor.matching_parts(image) {
      for image_usage in image_usages {
        formatter.push_target_id_value(context.parts_to_string_stdout(&image_parts), image_usage);
      }
    }
  }
  if formatter.is_empty() {
    context.print_outcome("no matches found in services");
  } else {
    formatter.print()?;
  }
  Ok(())
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum ImageUsageLabel {
  Image,
  Instances,
  Registry,
  Service,
}

impl Label for ImageUsageLabel {
  fn as_str(&self) -> &str {
    match self {
      ImageUsageLabel::Image => "image",
      ImageUsageLabel::Instances => "#",
      ImageUsageLabel::Registry => "registry",
      ImageUsageLabel::Service => "service id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Image)
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
struct ImageUsage {
  registry: String,
  service_id: String,
  instances: u64,
}

impl ImageUsage {
  fn new(registry: String, service_id: String, instances: u64) -> Self {
    Self { registry, service_id, instances }
  }
}

impl SubjectFormatter<ImageUsageLabel> for ImageUsage {
  fn value(&self, label: &ImageUsageLabel, target_id: &str) -> String {
    match label {
      ImageUsageLabel::Service => self.service_id.clone(),
      ImageUsageLabel::Image => target_id.to_string(),
      ImageUsageLabel::Instances => self.instances.to_string(),
      ImageUsageLabel::Registry => self.registry.to_string(),
    }
  }
}

const IMAGE_USAGE_LABELS: [ImageUsageLabel; 4] = [ImageUsageLabel::Image, ImageUsageLabel::Registry, ImageUsageLabel::Service, ImageUsageLabel::Instances];
