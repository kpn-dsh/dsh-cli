use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::AppCatalogApp;

use crate::{DcliContext, DcliResult, include_app_application, include_started_stopped};
use crate::app::get_application_from_app;
use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::filter_flags::FilterFlagType;
use crate::formatters::formatter::StringTableBuilder;
use crate::subject::Subject;

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

  fn subject_first_upper(&self) -> &'static str {
    "Metric"
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

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, METRIC_LIST_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref METRIC_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List exported metrics".to_string(),
    command_long_about: None,
    command_executors: vec![],
    default_command_executor: Some(&MetricList {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![(FilterFlagType::App, None), (FilterFlagType::Application, None), (FilterFlagType::Started, None), (FilterFlagType::Stopped, None)],
    modifier_flags: vec![],
  });
}

struct MetricList {}

#[async_trait]
impl CommandExecutor for MetricList {
  async fn execute(&self, _argument: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let (include_app, include_application) = include_app_application(matches);
    let (include_started, include_stopped) = include_started_stopped(matches);
    if include_app {
      if context.show_capability_explanation() {
        println!("find exported metrics in apps");
      }
      let apps: &HashMap<String, AppCatalogApp> = &dsh_api_client.get_app_configurations().await?;
      let mut app_ids = apps.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      app_ids.sort();
      let mut builder = StringTableBuilder::new(&["app", "application resource", "#", "path", "port"], context);
      let mut found = false;
      for app_id in app_ids {
        let app = apps.get(&app_id).unwrap();
        if let Some((resource_id, application)) = get_application_from_app(app) {
          if (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped) {
            if let Some(ref metric) = application.metrics {
              builder.vec(&vec![
                app_id,
                resource_id.to_string(),
                application.instances.to_string(),
                metric.path.clone(),
                metric.port.to_string(),
              ]);
              found = true;
            }
          }
        }
      }
      if found {
        builder.print_list();
      } else {
        println!("no metrics exported in apps");
      }
    }
    if include_application {
      if context.show_capability_explanation() {
        println!("find exported metrics in applications");
      }
      let applications = &dsh_api_client.get_application_actual_configurations().await?;
      let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<String>>();
      application_ids.sort();
      let mut builder = StringTableBuilder::new(&["application", "#", "path", "port"], context);
      let mut found = false;
      for application_id in application_ids {
        let application = applications.get(&application_id).unwrap();
        if (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped) {
          if let Some(ref metric) = application.metrics {
            builder.vec(&vec![
              application_id,
              application.instances.to_string(),
              metric.path.clone(),
              metric.port.to_string(),
            ]);
            found = true;
          }
        }
      }
      if found {
        builder.print_list();
      } else {
        println!("no metrics exported in applications");
      }
    }
    Ok(false)
  }
}