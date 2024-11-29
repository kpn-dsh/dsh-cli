use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use lazy_static::lazy_static;

use dsh_api::query_processor::{ExactMatchQueryProcessor, QueryProcessor, RegexQueryProcessor};
use dsh_api::types::AppCatalogApp;

use crate::arguments::query_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::filter_flags::FilterFlagType;
use crate::formatters::formatter::StringTableBuilder;
use crate::formatters::{wrap_vec_parts, TerminalStyle};
use crate::modifier_flags::ModifierFlagType;
use crate::subject::Subject;
use crate::{include_app_application, DcliContext, DcliResult};

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

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Find, IMAGE_FIND_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  // TODO Add list capability
    pub static ref IMAGE_FIND_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Find, "Find used images")
    .set_long_about("Find all applications and/or apps that use a given Harbor image.")
    .set_default_command_executor(&ImageFind {})
    .add_target_argument(query_argument(None))
    .add_filter_flags(vec![(FilterFlagType::App, None), (FilterFlagType::Application, None)])
    .add_modifier_flag(ModifierFlagType::Regex, None)
  );
}

struct ImageFind {}

#[async_trait]
impl CommandExecutor for ImageFind {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    let image_query = target.unwrap_or_else(|| unreachable!());
    let (query_processor, terminal_style): (&dyn QueryProcessor, TerminalStyle) = if matches.get_flag(ModifierFlagType::Regex.id()) {
      (&RegexQueryProcessor::create(image_query.as_str())?, TerminalStyle::Bold)
    } else {
      (&ExactMatchQueryProcessor::create(image_query.as_str())?, TerminalStyle::Normal)
    };
    let (include_app, include_application) = include_app_application(matches);
    if include_app {
      if context.show_capability_explanation() {
        println!("find images in apps that {}", query_processor.describe());
      }
      let apps: &HashMap<String, AppCatalogApp> = &context.dsh_api_client.as_ref().unwrap().get_app_configurations().await?;
      let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<_>>();
      app_ids.sort();
      let mut builder = StringTableBuilder::new(&["app", "application resource", "image"], context);
      let mut found = false;
      for app_id in app_ids {
        let app = apps.get(&app_id).unwrap();
        if let Some((resource_id, application)) = DshApiClient::application_from_app(app) {
          if let Some(image_parts) = query_processor.matching_parts(application.image.as_str()) {
            found = true;
            builder.vec(&vec![
              app_id,
              resource_id.to_string(),
              wrap_vec_parts(terminal_style.clone(), image_parts.as_slice()),
            ]);
          }
        }
      }
      if found {
        builder.print_list();
      } else {
        println!("image not used in apps");
      }
    }
    if include_application {
      if context.show_capability_explanation() {
        println!("find environment variables in applications that {}", query_processor.describe());
      }
      let applications = &context.dsh_api_client.as_ref().unwrap().get_applications().await?;
      let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<_>>();
      application_ids.sort();
      let mut builder = StringTableBuilder::new(&["application", "image"], context);
      let mut found = false;
      for application_id in application_ids {
        let application = applications.get(&application_id).unwrap();
        if let Some(image_parts) = query_processor.matching_parts(application.image.as_str()) {
          found = true;
          builder.vec(&vec![application_id, wrap_vec_parts(terminal_style.clone(), image_parts.as_slice())]);
        }
      }
      if found {
        builder.print_list();
      } else {
        println!("image not used in applications");
      }
    }
    Ok(false)
  }
}
