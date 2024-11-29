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

pub(crate) struct EnvSubject {}

const ENV_SUBJECT_TARGET: &str = "env";

lazy_static! {
  pub static ref ENV_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(EnvSubject {});
}

#[async_trait]
impl Subject for EnvSubject {
  fn subject(&self) -> &'static str {
    ENV_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Find values used in configurations.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Find values used in environment variables used to configure applications/services and apps deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("e")
  }

  fn requires_dsh_api_client(&self) -> bool {
    true
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Find, ENV_FIND_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref ENV_FIND_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Find, "Find environment variable values")
      .set_long_about("Find values used in environment variables used to configure applications/services and apps deployed on the DSH.")
      .set_default_command_executor(&EnvFind {})
      .add_target_argument(query_argument(None))
      .add_filter_flags(vec![(FilterFlagType::App, None), (FilterFlagType::Application, None)])
      .add_modifier_flag(ModifierFlagType::Regex, None)
  );
}

struct EnvFind {}

#[async_trait]
impl CommandExecutor for EnvFind {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    let query = target.unwrap_or_else(|| unreachable!());
    let (query_processor, terminal_style): (&dyn QueryProcessor, TerminalStyle) = if matches.get_flag(ModifierFlagType::Regex.id()) {
      (&RegexQueryProcessor::create(query.as_str())?, TerminalStyle::Bold)
    } else {
      (&ExactMatchQueryProcessor::create(query.as_str())?, TerminalStyle::Normal)
    };
    let (include_app, include_application) = include_app_application(matches);
    if include_app {
      if context.show_capability_explanation() {
        println!("find environment variables in apps that {}", query_processor.describe());
      }
      let apps: &HashMap<String, AppCatalogApp> = &context.dsh_api_client.as_ref().unwrap().get_app_configurations().await?;
      let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<_>>();
      app_ids.sort();
      let mut builder = StringTableBuilder::new(&["app", "application resource", "#", "environment variable", "value"], context);
      let mut found = false;
      for app_id in app_ids {
        let app = apps.get(&app_id).unwrap();
        if let Some((resource_id, application)) = DshApiClient::application_from_app(app) {
          let mut envs: Vec<(String, String)> = application
            .env
            .iter()
            .filter_map(|(key, value)| {
              query_processor
                .matching_parts(value)
                .map(|ps| (key.to_string(), wrap_vec_parts(terminal_style.clone(), ps.as_slice())))
            })
            .collect();
          if !envs.is_empty() {
            found = true;
            envs.sort_by_key(|env| env.0.clone());
            builder.vec(&vec![
              app_id,
              resource_id.to_string(),
              application.instances.to_string(),
              envs.iter().map(|env| env.0.clone()).collect::<Vec<_>>().join("\n"),
              envs.iter().map(|env| env.1.clone()).collect::<Vec<_>>().join("\n"),
            ]);
          }
        }
      }
      if found {
        builder.print_list();
      } else {
        println!("no matches found in apps");
      }
    }
    if include_application {
      if context.show_capability_explanation() {
        println!("find environment variables in applications that {}", query_processor.describe());
      }
      let applications = &context.dsh_api_client.as_ref().unwrap().get_applications().await?;
      let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<_>>();
      application_ids.sort();
      let mut builder = StringTableBuilder::new(&["application", "#", "environment variable", "value"], context);
      let mut found = false;
      for application_id in application_ids {
        let application = applications.get(&application_id).unwrap();
        let mut envs: Vec<(String, String)> = application
          .env
          .iter()
          .filter_map(|(key, value)| {
            query_processor
              .matching_parts(value)
              .map(|ps| (key.to_string(), wrap_vec_parts(terminal_style.clone(), ps.as_slice())))
          })
          .collect();
        if !envs.is_empty() {
          found = true;
          envs.sort_by_key(|env| env.0.clone());
          builder.vec(&vec![
            application_id,
            application.instances.to_string(),
            envs.iter().map(|env| env.0.clone()).collect::<Vec<_>>().join("\n"),
            envs.iter().map(|env| env.1.clone()).collect::<Vec<_>>().join("\n"),
          ]);
        }
      }
      if found {
        builder.print_list();
      } else {
        println!("no matches found in applications");
      }
    }
    Ok(false)
  }
}
