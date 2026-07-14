use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::subject::Requirements;
use crate::subjects::vhost::{VhostListValue, VHOST_LIST_LABELS};
use crate::subjects::DEPENDANT_LABELS_LIST;
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::vhost::{VhostInjection, VhostString};
use dsh_api::Dependant;
use itertools::Itertools;
use std::str::FromStr;

pub(crate) struct VhostList {}

#[async_trait]
impl CommandExecutor for VhostList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list configured vhosts");
    let start_instant = context.now();
    let applications = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut vhost_list_values = applications
      .iter()
      .filter(|(_, application)| (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped))
      .flat_map(|(application_id, application)| {
        application
          .exposed_ports
          .iter()
          .filter_map(|(port, port_mapping)| match port_mapping.vhost {
            Some(ref vhost_string) => match VhostString::from_str(vhost_string) {
              Ok(vhost) => Some(VhostListValue {
                vhost: vhost.vhost_name,
                zone: vhost.zone,
                tenant: vhost.tenant_name,
                kafka_flag: vhost.kafka,
                service_id: application_id.to_string(),
                instances: application.instances,
                port: port.to_string(),
                port_mapping: port_mapping.clone(),
              }),
              Err(_) => None,
            },
            None => None,
          })
          .collect_vec()
      })
      .collect_vec();
    if vhost_list_values.is_empty() {
      context.print_outcome("no vhosts configured");
      Ok(())
    } else {
      vhost_list_values.sort_by(|a, b| (&a.vhost, &a.service_id).cmp(&(&b.vhost, &b.service_id)));
      let mut formatter = ListFormatter::new(&VHOST_LIST_LABELS, context);
      formatter.push_values(&vhost_list_values);
      formatter.print(None)
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct VhostListUsage {}

#[async_trait]
impl CommandExecutor for VhostListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list vhosts with services and apps that use them");
    let start_instant = context.now();
    let vhosts_with_usage: Vec<(String, Vec<Dependant<VhostInjection>>)> = client.vhosts_with_dependants().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, context);
    for (vhost, dependants) in &vhosts_with_usage {
      for dependant in dependants {
        formatter.push_target_id_value(vhost.clone(), dependant);
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}
