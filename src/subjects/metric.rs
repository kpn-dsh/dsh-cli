use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::list_formatter::ListFormatter;
use crate::subject::{Requirements, Subject};
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::Application;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

pub(crate) struct MetricSubject {}

const METRIC_SUBJECT_TARGET: &str = "metric";

lazy_static! {
  pub static ref METRIC_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(MetricSubject {});
}

#[async_trait]
impl Subject for MetricSubject {
  fn subject(&self) -> &'static str {
    METRIC_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show metric exports.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show which DSH components export metric.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("m")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(METRIC_LIST_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &METRIC_CAPABILITIES
  }
}

lazy_static! {
  static ref METRIC_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List exported metrics")
      .set_long_about("List all services/apps that have metrics export configured.")
      .set_default_command_executor(&MetricList {})
      .add_filter_flags(vec![(FilterFlagType::Started, None), (FilterFlagType::Stopped, None)])
  );
  static ref METRIC_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![METRIC_LIST_CAPABILITY.as_ref()];
}

struct MetricList {}

#[async_trait]
impl CommandExecutor for MetricList {
  async fn execute(&self, _argument: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (include_started, include_stopped) = include_started_stopped(matches);
    context.print_explanation("find exported metrics in services");
    let start_instant = Instant::now();
    let services = context.client_unchecked().get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let metrics_usage = metrics_usage_from_services(&services, include_started, include_stopped);
    if metrics_usage.is_empty() {
      context.print_outcome("no metrics exported in services");
    } else {
      let mut formatter = ListFormatter::new(&METRIC_USAGE_LABELS, None, context);
      formatter.push_values(&metrics_usage);
      formatter.print()?;
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

fn metrics_usage_from_services(services: &HashMap<String, Application>, include_started: bool, include_stopped: bool) -> Vec<MetricUsage> {
  let mut services = services.iter().collect::<Vec<_>>();
  services.sort_by(|(service_id_a, _), (service_id_b, _)| service_id_a.cmp(service_id_b));
  let mut metric_uage: Vec<MetricUsage> = vec![];
  for (service_id, service) in services {
    if (service.instances > 0 && include_started) || (service.instances == 0 && include_stopped) {
      if let Some(ref metric) = service.metrics {
        metric_uage.push(MetricUsage::new(service_id.clone(), service.instances, metric.path.clone(), metric.port));
      }
    }
  }
  metric_uage
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
enum MetricUsageLabel {
  Instances,
  Path,
  Port,
  Service,
}

impl Label for MetricUsageLabel {
  fn as_str(&self) -> &str {
    match self {
      MetricUsageLabel::Instances => "#",
      MetricUsageLabel::Path => "path",
      MetricUsageLabel::Port => "port",
      MetricUsageLabel::Service => "service id",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
struct MetricUsage {
  service_id: String,
  instances: u64,
  path: String,
  port: u64,
}

impl MetricUsage {
  fn new(service_id: String, instances: u64, path: String, port: u64) -> Self {
    Self { service_id, instances, path, port }
  }
}

impl SubjectFormatter<MetricUsageLabel> for MetricUsage {
  fn value(&self, label: &MetricUsageLabel, _target_id: &str) -> String {
    match label {
      MetricUsageLabel::Service => self.service_id.clone(),
      MetricUsageLabel::Path => self.path.clone(),
      MetricUsageLabel::Instances => self.instances.to_string(),
      MetricUsageLabel::Port => self.port.to_string(),
    }
  }
}

const METRIC_USAGE_LABELS: [MetricUsageLabel; 4] = [MetricUsageLabel::Service, MetricUsageLabel::Instances, MetricUsageLabel::Path, MetricUsageLabel::Port];
