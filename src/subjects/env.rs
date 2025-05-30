use crate::arguments::query_argument;
use crate::capability::{Capability, CommandExecutor, FIND_COMMAND, FIND_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::formatters::formatter::Label;
use crate::formatters::list_formatter::ListFormatter;
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::query_processor::{ExactMatchQueryProcessor, QueryProcessor, RegexQueryProcessor};
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;

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
    "Find values used in environment variables used to configure services and apps deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("e")
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
    CapabilityBuilder::new(FIND_COMMAND, Some(FIND_COMMAND_ALIAS), &EnvFind {}, "Find environment variable values")
      .set_long_about("Find values in environment variables in the configurations of services and apps deployed on the DSH.")
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("Search in all started services.".to_string())),
        (FilterFlagType::Stopped, Some("Search in all stopped services.".to_string()))
      ])
      .add_target_argument(query_argument(None).required(true))
      .add_modifier_flag(ModifierFlagType::Regex, None)
  );
  static ref ENV_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![ENV_FIND_CAPABILITY.as_ref()];
}

struct EnvFind {}

#[async_trait]
impl CommandExecutor for EnvFind {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let query = target.unwrap_or_else(|| unreachable!());
    let query_processor: &dyn QueryProcessor =
      if matches.get_flag(ModifierFlagType::Regex.id()) { &RegexQueryProcessor::create(&query)? } else { &ExactMatchQueryProcessor::create(&query)? };
    let (include_started, include_stopped) = include_started_stopped(matches);
    context.print_explanation(format!("find environment variables in services that {}", query_processor.describe()));
    let start_instant = context.now();
    let services = &client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);

    let mut service_pairs = services.iter().collect::<Vec<_>>();
    service_pairs.sort_by(|(service_id_a, _), (service_id_b, _)| service_id_a.cmp(service_id_b));

    let mut matching_services: Vec<(String, HashMap<ServiceEnvLabel, String>)> = vec![];
    for (service_id, service) in service_pairs {
      if (service.instances > 0 && include_started) || (service.instances == 0 && include_stopped) {
        let mut envs: Vec<(String, String)> = service
          .env
          .iter()
          .filter_map(|(key, value)| {
            query_processor
              .matching_parts(value)
              .map(|ps| (key.to_string(), context.parts_to_string_for_stdout(ps.as_slice(), None)))
          })
          .collect();
        envs.sort_by_key(|env| env.0.clone());
        for (key, value) in envs {
          let mut env_map: HashMap<ServiceEnvLabel, String> = HashMap::new();
          env_map.insert(ServiceEnvLabel::Instances, service.instances.to_string());
          env_map.insert(ServiceEnvLabel::EnvVar, key);
          env_map.insert(ServiceEnvLabel::Value, value);
          matching_services.push((service_id.clone(), env_map));
        }
      }
    }
    if matching_services.is_empty() {
      context.print_outcome("no matches found in services");
    } else {
      let mut formatter = ListFormatter::new(&SERVICE_ENV_LABELS, None, context);
      formatter.push_target_id_value_pairs(&matching_services);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum ServiceEnvLabel {
  EnvVar,
  Instances,
  Service,
  Value,
}

impl Label for ServiceEnvLabel {
  fn as_str(&self) -> &str {
    match self {
      ServiceEnvLabel::Service => "service id",
      ServiceEnvLabel::Instances => "#",
      ServiceEnvLabel::EnvVar => "environment variable",
      ServiceEnvLabel::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Service)
  }
}

const SERVICE_ENV_LABELS: [ServiceEnvLabel; 4] = [ServiceEnvLabel::Service, ServiceEnvLabel::Instances, ServiceEnvLabel::EnvVar, ServiceEnvLabel::Value];
