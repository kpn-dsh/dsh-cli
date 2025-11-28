use crate::arguments::query_argument;
use crate::capability::{Capability, CommandExecutor, FIND_COMMAND, FIND_COMMAND_ALIAS, LIST_COMMAND, LIST_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::{Label, SubjectFormatter};
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::parse::ImageString;
use dsh_api::query_processor::{DummyQueryProcessor, ExactMatchQueryProcessor, QueryProcessor, RegexQueryProcessor};
use dsh_api::types::Application;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;

struct ImageSubject {}

const IMAGE_SUBJECT_TARGET: &str = "image";

lazy_static! {
  pub(crate) static ref IMAGE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ImageSubject {});
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
    CapabilityBuilder::new(FIND_COMMAND, Some(FIND_COMMAND_ALIAS), &ImageFind {}, "Find used images")
      .set_long_about("Find all services and/or apps that use a given Harbor image.")
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search in all started services.".to_string())),
        (FilterFlagType::Stopped, Some("Search in all stopped services.".to_string()))
      ])
      .add_target_argument(query_argument(None).required(true))
      .add_modifier_flag(ModifierFlagType::Regex, None)
  );
  static ref IMAGE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ImageListAll {}, "List images")
      .set_long_about(
        "Lists all images that are deployed in at least one service. \
        This will also include services that are stopped \
        (deployed with 0 instances)."
      )
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
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let image_query = target.unwrap_or_else(|| unreachable!());
    let query_processor: &dyn QueryProcessor =
      if matches.get_flag(ModifierFlagType::Regex.id()) { &RegexQueryProcessor::create(&*image_query)? } else { &ExactMatchQueryProcessor::create(&image_query)? };
    context.print_explanation(format!("find images that {}", query_processor.describe()));
    let start_instant = context.now();
    let services = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    list_images(services, query_processor, matches, context)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ImageListAll {}

#[async_trait]
impl CommandExecutor for ImageListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all images used in services");
    let start_instant = context.now();
    let services = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    list_images(services, &DummyQueryProcessor::create()?, matches, context)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn list_images(services: HashMap<String, Application>, query_processor: &dyn QueryProcessor, matches: &ArgMatches, context: &Context) -> Result<(), String> {
  let (include_started, include_stopped) = include_started_stopped(matches);
  let mut services = services.iter().collect_vec();
  services.sort_by(|(service_id_a, _), (service_id_b, _)| service_id_a.cmp(service_id_b));
  let mut images: HashMap<String, Vec<ImageUsage>> = HashMap::new();
  for (service_id, service) in services {
    if (service.instances > 0 && include_started) || (service.instances == 0 && include_stopped) && !service.image.is_empty() {
      let image_string = ImageString::from(service.image.as_str());
      images
        .entry(image_string.id())
        .or_default()
        .push(ImageUsage::new(image_string, service_id.to_string(), service.instances));
    }
  }
  let mut images: Vec<(String, Vec<ImageUsage>)> = images.into_iter().collect_vec();
  images.sort_by(|(image_a, _), (image_b, _)| image_a.cmp(image_b));
  let mut formatter = ListFormatter::new(&IMAGE_USAGE_LABELS, None, context);
  for (image, image_usages) in &images {
    if let Some(matching) = query_processor.matching_parts(image) {
      for image_usage in image_usages {
        formatter.push_target_id_value(context.parts_to_string_for_stdout(&matching, None), image_usage);
      }
    }
  }
  if formatter.is_empty() {
    context.print_outcome("no matches found in services");
  } else {
    formatter.print(None)?;
  }
  Ok(())
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum ImageUsageLabel {
  Id,
  Instances,
  Service,
  Source,
  Stage,
  Supplier,
  Tenant,
  Version,
}

impl Label for ImageUsageLabel {
  fn as_str(&self) -> &str {
    match self {
      ImageUsageLabel::Id => "image id",
      ImageUsageLabel::Instances => "#",
      ImageUsageLabel::Service => "service id",
      ImageUsageLabel::Source => "source",
      ImageUsageLabel::Stage => "stage",
      ImageUsageLabel::Supplier => "supplier",
      ImageUsageLabel::Tenant => "tenant",
      ImageUsageLabel::Version => "version",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Id)
  }
}

#[derive(Debug, Hash, PartialEq, Serialize)]
struct ImageUsage {
  image: ImageString,
  service_id: String,
  instances: u64,
}

impl ImageUsage {
  fn new(image: ImageString, service_id: String, instances: u64) -> Self {
    Self { image, service_id, instances }
  }
}

impl SubjectFormatter<ImageUsageLabel> for ImageUsage {
  fn value(&self, label: &ImageUsageLabel, target_id: &str) -> String {
    match label {
      ImageUsageLabel::Id => target_id.to_string(),
      ImageUsageLabel::Instances => self.instances.to_string(),
      ImageUsageLabel::Service => self.service_id.clone(),
      ImageUsageLabel::Source => self.image.source().to_string(),
      ImageUsageLabel::Stage => match &self.image {
        ImageString::App(app) => app.stage.clone(),
        _ => "".to_string(),
      },
      ImageUsageLabel::Supplier => match &self.image {
        ImageString::App(app) => app.supplier.clone(),
        _ => "".to_string(),
      },
      ImageUsageLabel::Tenant => self.image.tenant().clone(),
      ImageUsageLabel::Version => self.image.version().clone(),
    }
  }
}

const IMAGE_USAGE_LABELS: [ImageUsageLabel; 8] = [
  ImageUsageLabel::Id,
  ImageUsageLabel::Service,
  ImageUsageLabel::Instances,
  ImageUsageLabel::Version,
  ImageUsageLabel::Source,
  ImageUsageLabel::Stage,
  ImageUsageLabel::Supplier,
  ImageUsageLabel::Tenant,
];
