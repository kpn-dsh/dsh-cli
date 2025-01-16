use crate::arguments::query_argument;
use crate::capability::{Capability, CommandExecutor, FIND_COMMAND, FIND_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::formatters::formatter::Label;
use crate::formatters::list_formatter::ListFormatter;
use crate::modifier_flags::ModifierFlagType;
use crate::subject::Subject;
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::query_processor::{ExactMatchQueryProcessor, QueryProcessor, RegexQueryProcessor};
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Instant;

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
  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      FIND_COMMAND => Some(ENV_FIND_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &ENV_CAPABILITIES
  }
}

lazy_static! {
  static ref ENV_FIND_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(FIND_COMMAND_PAIR, "Find environment variable values")
      .set_long_about("Find values in environment variables in the configurations of applications and apps deployed on the DSH.")
      .set_default_command_executor(&EnvFind {})
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search in all started applications.".to_string())),
        (FilterFlagType::Stopped, Some("Search in all stopped applications.".to_string()))
      ])
      .add_target_argument(query_argument(None))
      .add_modifier_flag(ModifierFlagType::Regex, None)
  );
  static ref ENV_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![ENV_FIND_CAPABILITY.as_ref()];
}

struct EnvFind {}

#[async_trait]
impl CommandExecutor for EnvFind {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let query = target.unwrap_or_else(|| unreachable!());
    let query_processor: &dyn QueryProcessor =
      if matches.get_flag(ModifierFlagType::Regex.id()) { &RegexQueryProcessor::create(query.as_str())? } else { &ExactMatchQueryProcessor::create(query.as_str())? };
    let (include_started, include_stopped) = include_started_stopped(matches);
    context.print_explanation(format!("find environment variables in applications that {}", query_processor.describe()));
    let start_instant = Instant::now();
    let applications = &context.dsh_api_client.as_ref().unwrap().get_applications().await?;
    context.print_execution_time(start_instant);

    let mut application_pairs = applications.iter().collect::<Vec<_>>();
    application_pairs.sort_by(|(application_id_a, _), (application_id_b, _)| application_id_a.cmp(application_id_b));

    let mut matching_applications: Vec<(String, HashMap<ApplicationEnvLabel, String>)> = vec![];
    for (application_id, application) in application_pairs {
      if (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped) {
        let mut envs: Vec<(String, String)> = application
          .env
          .iter()
          .filter_map(|(key, value)| {
            query_processor
              .matching_parts(value)
              .map(|ps| (key.to_string(), context.parts_to_string_stdout(ps.as_slice())))
          })
          .collect();
        envs.sort_by_key(|env| env.0.clone());
        let mut first = true;
        for (key, value) in envs {
          let mut env_map: HashMap<ApplicationEnvLabel, String> = HashMap::new();
          env_map.insert(ApplicationEnvLabel::Instances, application.instances.to_string());
          env_map.insert(ApplicationEnvLabel::EnvVar, key);
          env_map.insert(ApplicationEnvLabel::Value, value);
          if first {
            matching_applications.push((application_id.clone(), env_map));
          } else {
            matching_applications.push(("".to_string(), env_map));
          }
          first = false;
        }
      }
    }
    if matching_applications.is_empty() {
      context.print_outcome("no matches found in applications");
    } else {
      let mut formatter = ListFormatter::new(&APPLICATION_ENV_LABELS, None, context);
      formatter.push_target_id_value_pairs(&matching_applications);
      formatter.print()?;
    }
    Ok(())
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum ApplicationEnvLabel {
  Application,
  Instances,
  EnvVar,
  Value,
}

impl Label for ApplicationEnvLabel {
  fn as_str(&self) -> &str {
    match self {
      ApplicationEnvLabel::Application => "application id",
      ApplicationEnvLabel::Instances => "#",
      ApplicationEnvLabel::EnvVar => "environment variable",
      ApplicationEnvLabel::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Application)
  }
}

static APPLICATION_ENV_LABELS: [ApplicationEnvLabel; 4] =
  [ApplicationEnvLabel::Application, ApplicationEnvLabel::Instances, ApplicationEnvLabel::EnvVar, ApplicationEnvLabel::Value];
