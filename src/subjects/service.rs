use crate::argument_parsers::RangedValueParser;
use crate::arguments::service_id_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, DUPLICATE_COMMAND, EDIT_COMMAND, EXPORT_COMMAND, LIST_COMMAND,
  LIST_COMMAND_ALIAS, OPEN_COMMAND, OPEN_COMMAND_ALIAS, RESTART_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS, START_COMMAND, STOP_COMMAND, UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::error::DshCliError;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{hashmap_to_table, ColumnAlignment, Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::subject::{Requirements, Subject};
use crate::subjects::DEFAULT_ALLOCATION_STATUS_LABELS;
use crate::target_tenant::get_target_tenant;
use crate::{edit_configuration, err, get_target_platform, include_started_stopped, read_single_line, DshCliResult, COMMAND_OPTIONS_HEADING};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::parse::ImageString;
use dsh_api::platform::VhostZone;
use dsh_api::types::{Application, ApplicationSecret, TaskState};
use dsh_api::vhost::VhostString;
use futures::future::try_join_all;
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

struct ServiceSubject {}

const SERVICE_SUBJECT_TARGET: &str = "service";

lazy_static! {
  pub(crate) static ref SERVICE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ServiceSubject {});
}

lazy_static! {
  static ref SERVICE_CREATE_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &ServiceCreate {}, "Create service")
      .set_long_about("Create a new service.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_DELETE_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &ServiceDelete {}, "Delete service")
      .set_long_about("Deletes a service from the DSH platform.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_DUPLICATE_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(DUPLICATE_COMMAND, None, &ServiceDuplicate {}, "Duplicate service configuration")
      .set_long_about("Duplicate a service configuration and update it using your default editor.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(Arg::new("verbatim-flag").long("verbatim").action(ArgAction::SetTrue).help("Verbatim duplicate"))
  );
  static ref SERVICE_EDIT_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(EDIT_COMMAND, None, &ServiceEdit {}, "Edit service configuration")
      .set_long_about("Edit the service configuration using your default editor.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_EXPORT_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(EXPORT_COMMAND, None, &ServiceExport {}, "Export service configuration")
      .set_long_about("Export the service configuration file.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_LIST_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ServiceListAll {}, "List services")
      .set_long_about(
        "Lists all DSH services. \
        This will also include services that are stopped \
        (deployed with 0 instances)."
      )
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ServiceListAllocationStatus {}, None),
        (FlagType::Ids, &ServiceListIds {}, None),
      ])
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List all started services.".to_string())),
        (FilterFlagType::Stopped, Some("List all stopped services.".to_string()))
      ])
  );
  static ref SERVICE_OPEN_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(OPEN_COMMAND, Some(OPEN_COMMAND_ALIAS), &ServiceOpen {}, "Open service vhost")
      .set_long_about("Open the vhost of a DSH service.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_RESTART_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(RESTART_COMMAND, None, &ServiceRestart {}, "Restart service")
      .set_long_about("Restarts an already running service.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_SHOW_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ServiceShow {}, "Show service configuration")
      .set_long_about("Show the configuration of a DSH service.")
      .add_command_executors(vec![(FlagType::AllocationStatus, &ServiceShowAllocationStatus {}, None),])
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_START_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(START_COMMAND, None, &ServiceStart {}, "Start service")
      .set_long_about("Start a DSH service.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(instances_option().help_heading(COMMAND_OPTIONS_HEADING))
  );
  static ref SERVICE_STOP_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(STOP_COMMAND, None, &ServiceStop {}, "Stop service")
      .set_long_about("Stop a running DSH service, by setting the number of instances to 0.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_UPDATE_CAPABILITY: Box<dyn Capability + Send + Sync> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &ServiceUpdate {}, "Update service")
      .set_long_about("Update a DSH service.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(cpus_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(instances_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(mem_option().help_heading(COMMAND_OPTIONS_HEADING))
  );
  static ref SERVICE_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    SERVICE_CREATE_CAPABILITY.as_ref(),
    SERVICE_DELETE_CAPABILITY.as_ref(),
    SERVICE_DUPLICATE_CAPABILITY.as_ref(),
    SERVICE_EDIT_CAPABILITY.as_ref(),
    SERVICE_EXPORT_CAPABILITY.as_ref(),
    SERVICE_LIST_CAPABILITY.as_ref(),
    SERVICE_OPEN_CAPABILITY.as_ref(),
    SERVICE_RESTART_CAPABILITY.as_ref(),
    SERVICE_SHOW_CAPABILITY.as_ref(),
    SERVICE_START_CAPABILITY.as_ref(),
    SERVICE_STOP_CAPABILITY.as_ref(),
    SERVICE_UPDATE_CAPABILITY.as_ref()
  ];
}

#[async_trait]
impl Subject for ServiceSubject {
  fn subject(&self) -> &'static str {
    SERVICE_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list services deployed on the DSH.".to_string()
  }

  // Shortcut 's' is an alias for 'secret' since release 0.10.1
  // fn subject_command_alias(&self) -> Option<&str> { Some("s") }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(SERVICE_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(SERVICE_DELETE_CAPABILITY.as_ref()),
      EDIT_COMMAND => Some(SERVICE_EDIT_CAPABILITY.as_ref()),
      EXPORT_COMMAND => Some(SERVICE_EXPORT_CAPABILITY.as_ref()),
      DUPLICATE_COMMAND => Some(SERVICE_DUPLICATE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(SERVICE_LIST_CAPABILITY.as_ref()),
      OPEN_COMMAND => Some(SERVICE_OPEN_CAPABILITY.as_ref()),
      RESTART_COMMAND => Some(SERVICE_RESTART_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(SERVICE_SHOW_CAPABILITY.as_ref()),
      START_COMMAND => Some(SERVICE_START_CAPABILITY.as_ref()),
      STOP_COMMAND => Some(SERVICE_STOP_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(SERVICE_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &SERVICE_CAPABILITIES
  }
}

pub(crate) const CPUS_OPTION: &str = "cpus";

pub(crate) fn cpus_option() -> Arg {
  Arg::new(CPUS_OPTION)
    .long(CPUS_OPTION)
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<f64>::new(0.01, 16.0))
    .value_name("CPUS")
    .help("Number of cpus")
    .long_help(
      "Set the maximum number of cpus available for the service \
       (factions of a vCPU core, 1.0 equals 1 vCPU). \
       The value must be greater than or equal to 0.01 \
       and lower than or equal to 16.0.",
    )
}

pub(crate) const INSTANCES_OPTION: &str = "instances";

pub(crate) fn instances_option() -> Arg {
  Arg::new(INSTANCES_OPTION)
    .long("instances")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<u64>::with_lower(0))
    .value_name("INSTANCES")
    .help("Number of instances")
    .long_help(
      "Set the number of service instances that will be started. \
       Setting this value to 0 will deploy the service without starting it.",
    )
}

pub(crate) const MEM_OPTION: &str = "mem";

pub(crate) fn mem_option() -> Arg {
  Arg::new(MEM_OPTION)
    .long(MEM_OPTION)
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<u64>::new(1, 131072))
    .value_name("MEM")
    .help("Amount of memory")
    .long_help(
      "Set the amount of memory available for the service (MiB). \
       The value must be greater than or equal to 1 and lower than or equal to 131072.",
    )
}

struct ServiceCreate {}

#[async_trait]
impl CommandExecutor for ServiceCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    if client.get_application_configuration(&service_id).await.is_ok() {
      return err!("service '{}' already exists", service_id);
    }
    context.print_explanation(format!("create new service '{}'", service_id));
    let configuration = context.read_multi_line("enter json configuration (terminate input with ctrl-d after last line)")?;
    match serde_json::from_str::<Application>(&configuration) {
      Ok(service) => {
        if context.dry_run() {
          context.print_warning("dry-run mode, service not created");
        } else {
          client.put_application_configuration(&service_id, &service).await?;
          context.print_outcome(format!("service '{}' created", service_id));
        }
        Ok(())
      }
      Err(error) => err!("invalid json configuration ({})", error),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceDelete {}

#[async_trait]
impl CommandExecutor for ServiceDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    if client.get_application_configuration(&service_id).await.is_err() {
      return err!("service '{}' does not exist", service_id);
    }
    if context.confirmed(format!("delete service '{}'?", service_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, service not deleted");
      } else {
        client.delete_application_configuration(&service_id).await?;
        context.print_outcome(format!("service '{}' deleted", service_id));
      }
    } else {
      context.print_outcome(format!("cancelled, service '{}' not deleted", service_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceDuplicate {}

#[async_trait]
impl CommandExecutor for ServiceDuplicate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("create new service from service '{}'", service_id));
    let verbatim = matches.get_flag("verbatim-flag");
    let duplicate_service_id = read_single_line("service name for new service: ")?;
    if client.get_application_configuration(&duplicate_service_id).await.is_ok() {
      context.print_error(format!("service '{}' already exists", duplicate_service_id));
      return Ok(());
    }
    match client.get_application_configuration(&service_id).await {
      Ok(mut application) => {
        if !verbatim {
          match edit_configuration(
            &application,
            &format!("{}.{}.{}.configuration.json", &client.platform().name(), client.tenant().name(), &service_id,),
            matches,
          )
          .await?
          {
            Some(updated_application) => application = updated_application,
            None => context.print_warning("configuration file hasn't changed, verbatim duplicate created"),
          }
        }
        if context.confirmed(format!("create duplicate service '{}'?", duplicate_service_id))? {
          if context.dry_run() {
            context.print_warning("dry-run mode, duplicate service not created");
          } else {
            client.put_application_configuration(&duplicate_service_id, &application).await?;
            context.print_outcome(format!("new service '{}' created from service '{}'", duplicate_service_id, service_id));
          }
        }
        Ok(())
      }
      Err(error) => DshCliError::accept_not_found(error, || context.print_error(format!("service '{}' does not exist", service_id))),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceEdit {}

#[async_trait]
impl CommandExecutor for ServiceEdit {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("edit service '{}' configuration", service_id));
    match client.get_application_configuration(&service_id).await {
      Ok(application) => {
        match edit_configuration(
          &application,
          &format!("{}.{}.{}.configuration.json", &client.platform().name(), client.tenant().name(), &service_id,),
          matches,
        )
        .await?
        {
          Some(updated_application) => {
            if context.confirmed(format!("update service '{}'?", service_id))? {
              if context.dry_run() {
                context.print_warning("dry-run mode, service configuration not updated");
              } else {
                client.put_application_configuration(&service_id, &updated_application).await?;
                context.print_outcome(format!("service '{}' configuration updated", service_id));
              }
            }
          }
          None => context.print_warning("configuration file hasn't changed, service configuration not updated"),
        }
        Ok(())
      }
      Err(error) => DshCliError::accept_not_found(error, || context.print_error(format!("service '{}' does not exist", service_id))),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceExport {}

#[async_trait]
impl CommandExecutor for ServiceExport {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("export configuration file for service '{}'", service_id));
    let start_instant = context.now();
    let service = client.get_application_configuration(&service_id).await?;
    context.print_execution_time(start_instant);
    match context.output_format(None) {
      OutputFormat::Json => context.print_serializable(service, Some(OutputFormat::Json)),
      OutputFormat::JsonCompact => context.print_serializable(&service, Some(OutputFormat::JsonCompact)),
      _ => context.print_serializable(service, Some(OutputFormat::Json)),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static SERVICE_LABELS_LIST: [ServiceLabel; 8] = [
  ServiceLabel::Target,
  ServiceLabel::NeedsToken,
  ServiceLabel::Instances,
  ServiceLabel::Cpus,
  ServiceLabel::Mem,
  ServiceLabel::ExposedPorts,
  ServiceLabel::Metrics,
  ServiceLabel::Image,
];

struct ServiceListAll {}

#[async_trait]
impl CommandExecutor for ServiceListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all services with their parameters");
    let start_instant = context.now();
    let services = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let mut service_ids = services.keys().map(|k| k.to_string()).collect_vec();
    service_ids.sort();
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut formatter = ListFormatter::new(&SERVICE_LABELS_LIST, context);
    for service_id in service_ids {
      if let Some(service) = services.get(&service_id) {
        if (service.instances > 0 && include_started) || (service.instances == 0 && include_stopped) {
          formatter.push_target_id_value(service_id.clone(), service);
        }
      };
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceListAllocationStatus {}

#[async_trait]
impl CommandExecutor for ServiceListAllocationStatus {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all services with their allocation status");
    let start_instant = context.now();
    let service_ids = client.application_ids().await?;
    let allocation_statuses = try_join_all(service_ids.iter().map(|service_id| client.get_application_status(service_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
    formatter.push_target_ids_and_values(service_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceListIds {}

#[async_trait]
impl CommandExecutor for ServiceListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all service ids");
    let start_instant = context.now();
    let ids = client.application_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("service id", context);
    formatter.push_target_ids(ids.as_slice());
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceOpen {}

#[async_trait]
impl CommandExecutor for ServiceOpen {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let service_id = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let service = client.get_application_configuration(&service_id).await?;
    context.print_execution_time(start_instant);
    if service.exposed_ports.len() > 1 {
      err!("service has more than one exposed port")
    } else {
      match service.exposed_ports.iter().next() {
        Some((_, port_mapping)) => match &port_mapping.vhost {
          Some(vhost) => {
            let vhost_string = VhostString::from_str(vhost)?;
            match vhost_string.zone {
              Some(VhostZone::Private) => {
                context.open_url(
                  format!(
                    "https://{}",
                    client.platform().tenant_private_vhost_domain(client.tenant().name(), vhost_string.vhost_name)?
                  ),
                  format!("private vhost for tenant '{}@{}' and service '{}'", tenant_name, platform, service_id),
                );
                Ok(())
              }
              Some(VhostZone::Public) => {
                context.open_url(
                  format!("https://{}", client.platform().public_vhost_domain(vhost_string.vhost_name)),
                  format!("public vhost for tenant '{}@{}' and service '{}'", tenant_name, platform, service_id),
                );
                Ok(())
              }
              None => err!("exposed port has no zone"),
            }
          }
          None => err!("port mapping has no vhost"),
        },
        None => err!("service has no exposed ports"),
      }
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceRestart {}

#[async_trait]
impl CommandExecutor for ServiceRestart {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("restart service '{}'", service_id));
    match client.get_application_configuration(&service_id).await {
      Ok(mut configuration) => {
        let instances = configuration.instances;
        if instances == 0 {
          context.print_warning(format!("service '{}' not started", service_id));
        } else if !configuration.volumes.is_empty() && !context.confirmed("volume content will not be preserved, do you want to continue?")? {
          context.print_warning("cancelled, service not restarted");
        } else if context.dry_run() {
          context.print_warning("dry-run mode, service not restarted");
        } else {
          let task_ids = client.get_task_appid_ids(&service_id).await?;
          let task_statuses = try_join_all(task_ids.iter().map(|task_id| client.get_task(&service_id, task_id))).await?;
          let running_task_ids = task_ids
            .into_iter()
            .zip(task_statuses)
            .filter(|(_, task_status)| task_status.actual.clone().is_some_and(|t| t.state == TaskState::Running))
            .map(|(task_id, _)| task_id)
            .collect_vec();
          configuration.instances = 0;
          if running_task_ids.len() == 1 {
            context.print_outcome(format!("stop service '{}'", service_id));
          } else {
            context.print_outcome(format!("stop service '{}' ({} instances)", service_id, running_task_ids.len()));
          }
          client.put_application_configuration(&service_id, &configuration).await?;
          loop {
            context.print_progress_step();
            sleep(Duration::from_millis(1000)).await;
            let poll_tasks = try_join_all(running_task_ids.iter().map(|running_task_id| client.get_task(&service_id, running_task_id))).await?;
            if poll_tasks
              .iter()
              .all(|task_status| task_status.actual.clone().is_some_and(|task| task.state == TaskState::Killed))
            {
              break;
            }
          }
          if running_task_ids.len() == 1 {
            context.print_outcome(format!("\nservice '{}' stopped", service_id));
          } else {
            context.print_outcome(format!("\nservice '{}' stopped ({} instances)", service_id, running_task_ids.len()));
          }
          configuration.instances = instances;
          client.put_application_configuration(&service_id, &configuration).await?;
          if instances == 1 {
            context.print_outcome(format!("service '{}' started", service_id));
          } else {
            context.print_outcome(format!("service '{}' started ({} instances)", service_id, instances));
          }
        }
        Ok(())
      }
      Err(error) => DshCliError::accept_not_found(error, || context.print_error(format!("service '{}' does not exist", service_id))),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) static SERVICE_LABELS_SHOW: [ServiceLabel; 19] = [
  ServiceLabel::Target,
  ServiceLabel::NeedsToken,
  ServiceLabel::Instances,
  ServiceLabel::Cpus,
  ServiceLabel::Mem,
  ServiceLabel::ExposedPorts,
  ServiceLabel::Volumes,
  ServiceLabel::Metrics,
  ServiceLabel::NodepoolFeatures,
  ServiceLabel::Image,
  ServiceLabel::HealthCheck,
  ServiceLabel::ReadableStreams,
  ServiceLabel::WritableStreams,
  ServiceLabel::Secrets,
  ServiceLabel::SingleInstance,
  ServiceLabel::SpreadGroup,
  ServiceLabel::Topics,
  ServiceLabel::User,
  ServiceLabel::Env,
];

struct ServiceShow {}

#[async_trait]
impl CommandExecutor for ServiceShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for service '{}'", service_id));
    let start_instant = context.now();
    let (service, allocation_status) = join!(client.get_application_configuration(&service_id), client.get_application_status(&service_id));
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, SERVICE_SUBJECT_TARGET);
    UnitFormatter::new(service_id, &SERVICE_LABELS_SHOW, context).print(&service?, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ServiceShowAllocationStatus {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for service '{}'", service_id));
    let start_instant = context.now();
    let allocation_status = client.get_application_status(&service_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(service_id, &DEFAULT_ALLOCATION_STATUS_LABELS, context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceStart {}

#[async_trait]
impl CommandExecutor for ServiceStart {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    let instances: u64 = matches.get_one::<u64>(INSTANCES_OPTION).cloned().unwrap_or(1);
    if instances == 1 {
      context.print_explanation(format!("start service '{}'", service_id));
    } else {
      context.print_explanation(format!("start {} instances of service '{}'", instances, service_id));
    }
    match client.get_application_configuration(&service_id).await {
      Ok(mut configuration) => {
        if configuration.instances > 0 {
          context.print_warning(format!("service '{}' already started", service_id));
        } else if context.dry_run() {
          context.print_warning("dry-run mode, service not started");
        } else {
          configuration.instances = instances;
          client.put_application_configuration(&service_id, &configuration).await?;
          if instances == 1 {
            context.print_outcome(format!("service '{}' started", service_id));
          } else {
            context.print_outcome(format!("service '{}' started ({} instances)", service_id, instances));
          }
        }
        Ok(())
      }
      Err(error) => DshCliError::accept_not_found(error, || context.print_error(format!("service '{}' does not exist", service_id))),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceStop {}

#[async_trait]
impl CommandExecutor for ServiceStop {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("stop service '{}'", service_id));
    match client.get_application_configuration(&service_id).await {
      Ok(mut configuration) => {
        let running_instances = configuration.instances;
        if running_instances == 0 {
          context.print_warning(format!("service '{}' already stopped", service_id));
        } else if context.dry_run() {
          context.print_warning("dry-run mode, service not stopped");
        } else {
          configuration.instances = 0;
          client.put_application_configuration(&service_id, &configuration).await?;
          if running_instances == 1 {
            context.print_outcome(format!("service '{}' stopped", service_id));
          } else {
            context.print_outcome(format!("service '{}' stopped ({} instances)", service_id, running_instances));
          }
        }
        Ok(())
      }
      Err(error) => DshCliError::accept_not_found(error, || context.print_error(format!("service '{}' does not exist", service_id))),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceUpdate {}

#[async_trait]
impl CommandExecutor for ServiceUpdate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let service_id = target.unwrap_or_else(|| unreachable!());
    let cpus: Option<f64> = match matches.get_one::<f64>(CPUS_OPTION).cloned() {
      Some(cpus) => {
        if cpus >= 0.1 {
          Some(cpus)
        } else {
          return err!("cpus should be greater than or equal to 0.1");
        }
      }
      None => None,
    };
    let instances = matches.get_one::<u64>(INSTANCES_OPTION).cloned();
    let mem = matches.get_one::<u64>(MEM_OPTION).cloned();
    match client.get_application_configuration(&service_id).await {
      Ok(mut configuration) => {
        if cpus.is_some() || instances.is_some() || mem.is_some() {
          context.print_explanation(format!("update service '{}' from arguments", service_id));
          if cpus.iter().any(|cpus| *cpus != configuration.cpus)
            | instances.iter().any(|instances| *instances != configuration.instances)
            | mem.iter().any(|mem| *mem != configuration.mem)
          {
            if context.dry_run() {
              context.print_warning("dry-run mode, service not updated");
            } else {
              if let Some(cpus) = cpus {
                configuration.cpus = cpus
              }
              if let Some(instances) = instances {
                configuration.instances = instances
              }
              if let Some(mem) = mem {
                configuration.mem = mem
              }
              client.put_application_configuration(&service_id, &configuration).await?;
              context.print_outcome(format!("service '{}' updated", service_id));
            }
            Ok(())
          } else {
            context.print_outcome("provided arguments are equal to the current configuration, service not updated");
            Ok(())
          }
        } else {
          context.print_explanation(format!("update service '{}' from json configuration", service_id));
          let update_configuration_json = context.read_multi_line("enter json configuration (terminate input with ctrl-d after last line)")?;
          match serde_json::from_str::<Application>(&update_configuration_json) {
            Ok(update_configuration) => {
              if context.dry_run() {
                context.print_warning("dry-run mode, service not updated");
              } else {
                client.put_application_configuration(&service_id, &update_configuration).await?;
                context.print_outcome(format!("service '{}' updated", service_id));
              }
              Ok(())
            }
            Err(error) => err!("invalid json configuration ({})", error),
          }
        }
      }
      Err(error) => DshCliError::accept_not_found(error, || context.print_error(format!("service '{}' does not exist", service_id))),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ServiceLabel {
  Cpus,
  Env,
  ExposedPorts,
  HealthCheck,
  Image,
  Instances,
  Mem,
  Metrics,
  NeedsToken,
  NodepoolFeatures,
  ReadableStreams,
  Secrets,
  SingleInstance,
  SpreadGroup,
  Target,
  Topics,
  User,
  Volumes,
  WritableStreams,
}

impl Label for ServiceLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Target => "service id",
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "exposed ports",
      Self::HealthCheck => "health check",
      Self::Image => "image",
      Self::Instances => "instances",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "needs token",
      Self::NodepoolFeatures => "nodepool features",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single instance",
      Self::SpreadGroup => "spread group",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "ports",
      Self::HealthCheck => "health",
      Self::Image => "image",
      Self::Instances => "#",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "token",
      Self::NodepoolFeatures => "nodepool",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single",
      Self::SpreadGroup => "spread group",
      Self::Target => "service id",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }

  fn column_alignment(&self) -> ColumnAlignment {
    match self {
      Self::Mem => ColumnAlignment::Right,
      _ => ColumnAlignment::default(),
    }
  }
}

impl SubjectFormatter<ServiceLabel> for Application {
  fn value(&self, label: &ServiceLabel, target_id: &str) -> Value {
    match label {
      ServiceLabel::Cpus => Value::plain(self.cpus),
      ServiceLabel::Env => Value::plain(hashmap_to_table(&self.env)),
      ServiceLabel::ExposedPorts => {
        if self.exposed_ports.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            self
              .exposed_ports
              .iter()
              .map(|(port, port_mapping)| {
                format!(
                  "{} : {}",
                  port,
                  VhostString::try_from(port_mapping).map(|vhost_string| vhost_string.to_string()).unwrap_or_default()
                )
              })
              .collect_vec()
              .join("\n"),
          )
        }
      }
      ServiceLabel::HealthCheck => Value::some_or_hide(self.health_check.clone()),
      ServiceLabel::Image => Value::plain(ImageString::from(self.image.as_str())),
      ServiceLabel::Instances => Value::plain(self.instances),
      ServiceLabel::Mem => Value::plain(self.mem),
      ServiceLabel::Metrics => Value::some_or_hide(self.metrics.clone().map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))),
      ServiceLabel::NeedsToken => Value::plain(self.needs_token),
      ServiceLabel::NodepoolFeatures => Value::some_or_hide(self.node_features.clone()),
      ServiceLabel::ReadableStreams => {
        if self.readable_streams.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            self
              .readable_streams
              .clone()
              .into_iter()
              .map(|readable_stream| readable_stream.to_string())
              .collect_vec()
              .join("\n"),
          )
        }
      }
      ServiceLabel::Secrets => {
        if self.secrets.is_empty() {
          Value::hide()
        } else {
          Value::plain(secrets_to_table(&self.secrets))
        }
      }
      ServiceLabel::SingleInstance => Value::plain(self.single_instance),
      ServiceLabel::SpreadGroup => Value::some_or_hide(self.spread_group.clone()),
      ServiceLabel::Target => Value::target(target_id),
      ServiceLabel::Topics => {
        if self.topics.is_empty() {
          Value::hide()
        } else {
          Value::plain(self.topics.clone().into_iter().map(|topic| topic.to_string()).collect_vec().join("\n"))
        }
      }
      ServiceLabel::User => Value::plain(&self.user),
      ServiceLabel::Volumes => {
        if self.volumes.is_empty() {
          Value::hide()
        } else {
          Value::plain(self.volumes.keys().map(|key| key.to_string()).collect_vec().join("\n"))
        }
      }
      ServiceLabel::WritableStreams => {
        if self.writable_streams.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            self
              .writable_streams
              .clone()
              .into_iter()
              .map(|writable_stream| writable_stream.to_string())
              .collect_vec()
              .join("\n"),
          )
        }
      }
    }
  }
}

fn secrets_to_table(secrets: &[ApplicationSecret]) -> String {
  let m: HashMap<String, String> = secrets
    .iter()
    .map(|application_secret| {
      (
        application_secret.name.clone(),
        application_secret
          .injections
          .iter()
          .map(|injection| injection.get("env").map(|s| s.to_string()).unwrap_or("".to_string()))
          .join(", "),
      )
    })
    .collect::<HashMap<_, _>>();
  hashmap_to_table(&m)
}
