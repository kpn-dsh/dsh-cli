use crate::arguments::service_id_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DUPLICATE_COMMAND, EDIT_COMMAND, EXPORT_COMMAND, EXPORT_COMMAND_ALIAS, LIST_COMMAND,
  LIST_COMMAND_ALIAS, RESTART_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS, START_COMMAND, STOP_COMMAND, UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::formatters::formatter::{hashmap_to_table, Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::subject::{Requirements, Subject};
use crate::subjects::DEFAULT_ALLOCATION_STATUS_LABELS;
use crate::{edit_configuration, include_started_stopped, read_single_line, DshCliResult};
use async_trait::async_trait;
use chrono::DateTime;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::application::parse_image_string;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{Application, TaskState};
use dsh_api::types::{Task, TaskStatus};
use dsh_api::DshApiError;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::thread::sleep;
use std::time::Duration;

pub(crate) struct ServiceSubject {}

const SERVICE_SUBJECT_TARGET: &str = "service";

lazy_static! {
  pub static ref SERVICE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ServiceSubject {});
}

lazy_static! {
  static ref SERVICE_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &ServiceCreate {}, "Create service")
      .set_long_about("Create a new service.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(instances_flag().help_heading(HELP_HEADING))
  );
  static ref SERVICE_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &ServiceDelete {}, "Delete service")
      .set_long_about("Deletes a service from the DSH platform.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_DUPLICATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DUPLICATE_COMMAND, None, &ServiceDuplicate {}, "Duplicate service configuration")
      .set_long_about("Duplicate a service configuration and update it using your default editor.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(Arg::new("verbatim-flag").long("verbatim").action(ArgAction::SetTrue).help("Verbatim duplicate"))
  );
  static ref SERVICE_EDIT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(EDIT_COMMAND, None, &ServiceEdit {}, "Edit service configuration")
      .set_long_about("Edit the service configuration using your default editor.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_EXPORT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(EXPORT_COMMAND, Some(EXPORT_COMMAND_ALIAS), &ServiceExport {}, "Export service configuration")
      .set_long_about("Export the service configuration file.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ServiceListAll {}, "List services")
      .set_long_about(
        "Lists all DSH services. \
        This will also include services that are stopped \
        (deployed with 0 instances)."
      )
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ServiceListAllocationStatus {}, None),
        (FlagType::Ids, &ServiceListIds {}, None),
        (FlagType::Tasks, &ServiceListTasks {}, None),
      ])
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List all started services.".to_string())),
        (FilterFlagType::Stopped, Some("List all stopped services.".to_string()))
      ])
  );
  static ref SERVICE_RESTART_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(RESTART_COMMAND, None, &ServiceRestart {}, "Restart service")
      .set_long_about("Restarts an already running service.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ServiceShowAll {}, "Show service configuration")
      .set_long_about("Show the configuration of a DSH service.")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ServiceShowAllocationStatus {}, None),
        (FlagType::Tasks, &ServiceShowTasks {}, None),
      ])
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_START_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(START_COMMAND, None, &ServiceStart {}, "Start service")
      .set_long_about("Start a DSH service.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(instances_flag().help_heading(HELP_HEADING))
  );
  static ref SERVICE_STOP_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(STOP_COMMAND, None, &ServiceStop {}, "Stop service")
      .set_long_about("Stop a running DSH service, by setting the number of instances to 0.")
      .add_target_argument(service_id_argument().required(true))
  );
  static ref SERVICE_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &ServiceUpdate {}, "Update service")
      .set_long_about("Update a DSH service.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(cpus_flag().help_heading(HELP_HEADING))
      .add_extra_argument(instances_flag().help_heading(HELP_HEADING))
      .add_extra_argument(mem_flag().help_heading(HELP_HEADING))
  );
  static ref SERVICE_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    SERVICE_CREATE_CAPABILITY.as_ref(),
    SERVICE_DELETE_CAPABILITY.as_ref(),
    SERVICE_DUPLICATE_CAPABILITY.as_ref(),
    SERVICE_EDIT_CAPABILITY.as_ref(),
    SERVICE_EXPORT_CAPABILITY.as_ref(),
    SERVICE_LIST_CAPABILITY.as_ref(),
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

  fn subject_command_alias(&self) -> Option<&str> {
    Some("s")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(SERVICE_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(SERVICE_DELETE_CAPABILITY.as_ref()),
      EDIT_COMMAND => Some(SERVICE_EDIT_CAPABILITY.as_ref()),
      EXPORT_COMMAND => Some(SERVICE_EXPORT_CAPABILITY.as_ref()),
      DUPLICATE_COMMAND => Some(SERVICE_DUPLICATE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(SERVICE_LIST_CAPABILITY.as_ref()),
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

const HELP_HEADING: &str = "Service options";

const CPUS_FLAG: &str = "cpus";

fn cpus_flag() -> Arg {
  Arg::new(CPUS_FLAG)
    .long(CPUS_FLAG)
    .action(ArgAction::Set)
    .value_parser(clap::value_parser!(f64))
    .value_name("CPUS")
    .help("Number of cpus")
    .long_help("Set number of cpus for the service.")
}

const INSTANCES_FLAG: &str = "instances";

fn instances_flag() -> Arg {
  Arg::new(INSTANCES_FLAG)
    .long("instances")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..))
    .value_name("INSTANCES")
    .help("Number of instances")
    .long_help("Number of service instances that will be started.")
    .help_heading(HELP_HEADING)
}

const MEM_FLAG: &str = "mem";

fn mem_flag() -> Arg {
  Arg::new(MEM_FLAG)
    .long(MEM_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=131072))
    .value_name("MEM")
    .help("Amount of memory")
    .long_help("Set amount of memory available for the service (MiB).")
}

struct ServiceCreate {}

#[async_trait]
impl CommandExecutor for ServiceCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    if client.get_application_configuration(&service_id).await.is_ok() {
      return Err(format!("service '{}' already exists", service_id));
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
      Err(error) => Err(format!("invalid json configuration ({})", error)),
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceDelete {}

#[async_trait]
impl CommandExecutor for ServiceDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete service '{}'", service_id));
    if client.get_application_configuration(&service_id).await.is_err() {
      return Err(format!("service '{}' does not exist", service_id));
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
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
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
      Err(error) => match error {
        DshApiError::NotFound(None) => {
          context.print_error(format!("service '{}' does not exist", service_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceEdit {}

#[async_trait]
impl CommandExecutor for ServiceEdit {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("edit service '{}' configuration", service_id));
    match client.get_application_configuration(&service_id).await {
      Ok(application) => {
        match edit_configuration(
          &application,
          &format!("{}.{}.{}.configuration.json", &client.platform().name(), client.tenant().name(), &service_id,),
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
      Err(error) => match error {
        DshApiError::NotFound(None) => {
          context.print_error(format!("service '{}' does not exist", service_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceExport {}

#[async_trait]
impl CommandExecutor for ServiceExport {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("export configuration file for service '{}'", service_id));
    let start_instant = context.now();
    let service = client.get_application_configuration(&service_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(service_id, &SERVICE_LABELS_SHOW, Some("service id"), context).print(&service, Some(OutputFormat::Json))
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceListAll {}

#[async_trait]
impl CommandExecutor for ServiceListAll {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all services with their parameters");
    let start_instant = context.now();
    let services = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let mut service_ids = services.keys().map(|k| k.to_string()).collect::<Vec<_>>();
    service_ids.sort();
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut formatter = ListFormatter::new(&SERVICE_LABELS_LIST, None, context);
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all services with their allocation status");
    let start_instant = context.now();
    let service_ids = client.list_application_ids().await?;
    let allocation_statuses = try_join_all(service_ids.iter().map(|service_id| client.get_application_status(service_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("service id"), context);
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
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    context.print_explanation("list all service ids");
    let start_instant = context.now();
    let ids = client.list_application_ids().await?;
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

struct ServiceListTasks {}

#[async_trait]
impl CommandExecutor for ServiceListTasks {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    fn tasks_to_string(tasks: Vec<String>) -> String {
      if tasks.len() <= 4 {
        tasks.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
      } else {
        format!(
          "{}, plus {} more",
          tasks.iter().take(4).map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
          tasks.len() - 4,
        )
      }
    }
    context.print_explanation("list all services with their tasks");
    let start_instant = context.now();
    let services_with_tasks = client.get_task_ids().await?;
    let tasks: Vec<Vec<String>> = try_join_all(services_with_tasks.iter().map(|service_id| client.get_task_appid_ids(service_id))).await?;
    context.print_execution_time(start_instant);
    let service_id_tasks_pairs: Vec<(String, String)> = services_with_tasks
      .iter()
      .zip(tasks)
      .map(|(id, tasks)| (id.to_string(), tasks_to_string(tasks)))
      .collect::<Vec<_>>();
    let mut formatter: ListFormatter<ServiceLabel, (String, String)> = ListFormatter::new(&[ServiceLabel::Target, ServiceLabel::Tasks], None, context);
    formatter.push_values(&service_id_tasks_pairs);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceRestart {}

#[async_trait]
impl CommandExecutor for ServiceRestart {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("restart service '{}'", service_id));
    match client.get_application_configuration(&service_id).await {
      Ok(mut configuration) => {
        let instances = configuration.instances;
        if instances == 0 {
          context.print_warning(format!("service '{}' not started", service_id));
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
            .collect::<Vec<_>>();
          configuration.instances = 0;
          if running_task_ids.len() == 1 {
            context.print_outcome(format!("stop service '{}'", service_id));
          } else {
            context.print_outcome(format!("stop service '{}' ({} instances)", service_id, running_task_ids.len()));
          }
          client.put_application_configuration(&service_id, &configuration).await?;
          loop {
            context.print_progress_step();
            sleep(Duration::from_millis(1000));
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
      Err(error) => match error {
        DshApiError::NotFound(None) => {
          context.print_error(format!("service '{}' does not exist", service_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceShowAll {}

#[async_trait]
impl CommandExecutor for ServiceShowAll {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for service '{}'", service_id));
    let start_instant = context.now();
    let service = client.get_application_configuration(&service_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(service_id, &SERVICE_LABELS_SHOW, Some("service id"), context).print(&service, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ServiceShowAllocationStatus {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for service '{}'", service_id));
    let start_instant = context.now();
    let allocation_status = client.get_application_status(&service_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(service_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("service id"), context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceShowTasks {}

#[async_trait]
impl CommandExecutor for ServiceShowTasks {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all tasks for service '{}'", service_id));
    let start_instant = context.now();
    let task_ids = client.get_task_appid_ids(&service_id).await?;
    let task_statuses = try_join_all(task_ids.iter().map(|task_id| client.get_task(&service_id, task_id))).await?;
    context.print_execution_time(start_instant);
    let mut tasks: Vec<(String, TaskStatus)> = task_ids.into_iter().zip(task_statuses).collect();
    tasks.sort_by(|first, second| second.1.actual.clone().unwrap().staged_at.cmp(&first.1.actual.clone().unwrap().staged_at));
    let mut formatter = ListFormatter::new(&TASK_LABELS_LIST, None, context);
    formatter.push_target_id_value_pairs(tasks.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceStart {}

#[async_trait]
impl CommandExecutor for ServiceStart {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    let instances: u64 = matches.get_one::<u64>(INSTANCES_FLAG).cloned().unwrap_or(1);
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
      Err(error) => match error {
        DshApiError::NotFound(None) => {
          context.print_error(format!("service '{}' does not exist", service_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceStop {}

#[async_trait]
impl CommandExecutor for ServiceStop {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
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
      Err(error) => match error {
        DshApiError::NotFound(None) => {
          context.print_error(format!("service '{}' does not exist", service_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ServiceUpdate {}

#[async_trait]
impl CommandExecutor for ServiceUpdate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult {
    let service_id = target.unwrap_or_else(|| unreachable!());
    let cpus: Option<f64> = match matches.get_one::<f64>(CPUS_FLAG).cloned() {
      Some(cpus) => {
        if cpus >= 0.1 {
          Some(cpus)
        } else {
          return Err("cpus should be greater than or equal to 0.1".to_string());
        }
      }
      None => None,
    };
    let instances = matches.get_one::<u64>(INSTANCES_FLAG).cloned();
    let mem = matches.get_one::<u64>(MEM_FLAG).cloned();
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
            Err(error) => Err(format!("invalid json configuration ({})", error)),
          }
        }
      }
      Err(error) => match error {
        DshApiError::NotFound(None) => {
          context.print_error(format!("service '{}' does not exist", service_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
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
  ReadableStreams,
  Secrets,
  SingleInstance,
  SpreadGroup,
  Target,
  Tasks,
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
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single instance",
      Self::SpreadGroup => "spread group",
      Self::Tasks => "tasks",
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
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single",
      Self::SpreadGroup => "spread group",
      Self::Target => "service id",
      Self::Tasks => "tasks",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<ServiceLabel> for Application {
  fn value(&self, label: &ServiceLabel, service_id: &str) -> String {
    match label {
      ServiceLabel::Cpus => self.cpus.to_string(),
      ServiceLabel::Env => hashmap_to_table(&self.env),
      ServiceLabel::ExposedPorts => self.exposed_ports.keys().map(|port| port.to_string()).collect::<Vec<_>>().join(","),
      ServiceLabel::HealthCheck => match self.health_check {
        Some(ref health_check) => match health_check.protocol {
          Some(protocol) => format!("{}:{}/{}", protocol.to_string(), health_check.port, health_check.path),
          None => format!("{}/{}", health_check.port, health_check.path),
        },
        None => "".to_string(),
      },
      ServiceLabel::Image => match parse_image_string(&self.image) {
        Ok((kind, image)) => format!("{}:{}", kind, image),
        Err(_) => self.image.clone(),
      },
      ServiceLabel::Instances => self.instances.to_string(),
      ServiceLabel::Mem => self.mem.to_string(),
      ServiceLabel::Metrics => self
        .metrics
        .clone()
        .map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))
        .unwrap_or_default(),
      ServiceLabel::NeedsToken => self.needs_token.to_string(),
      ServiceLabel::ReadableStreams => self
        .readable_streams
        .clone()
        .into_iter()
        .map(|readable_stream| readable_stream.to_string())
        .collect::<Vec<_>>()
        .join(", "),
      ServiceLabel::Secrets => self.secrets.clone().into_iter().map(|secret| secret.name).collect::<Vec<_>>().join(", "),
      ServiceLabel::SingleInstance => self.single_instance.to_string(),
      ServiceLabel::SpreadGroup => self.spread_group.clone().unwrap_or_default(),
      ServiceLabel::Target => service_id.to_string(),
      ServiceLabel::Tasks => "".to_string(),
      ServiceLabel::Topics => self.topics.clone().into_iter().map(|topic| topic.to_string()).collect::<Vec<_>>().join(", "),
      ServiceLabel::User => self.user.clone(),
      ServiceLabel::Volumes => self.volumes.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(","),
      ServiceLabel::WritableStreams => self
        .writable_streams
        .clone()
        .into_iter()
        .map(|writable_stream| writable_stream.to_string())
        .collect::<Vec<_>>()
        .join(", "),
    }
  }
}

pub static SERVICE_LABELS_LIST: [ServiceLabel; 8] = [
  ServiceLabel::Target,
  ServiceLabel::NeedsToken,
  ServiceLabel::Instances,
  ServiceLabel::Cpus,
  ServiceLabel::Mem,
  ServiceLabel::ExposedPorts,
  ServiceLabel::Metrics,
  ServiceLabel::Image,
];

pub static SERVICE_LABELS_SHOW: [ServiceLabel; 18] = [
  ServiceLabel::Target,
  ServiceLabel::NeedsToken,
  ServiceLabel::Instances,
  ServiceLabel::Cpus,
  ServiceLabel::Mem,
  ServiceLabel::ExposedPorts,
  ServiceLabel::Volumes,
  ServiceLabel::Metrics,
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

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum TaskLabel {
  Healthy,
  HostIpAddress,
  _LastestLog,
  LastUpdateAt,
  StagedAt,
  StartedAt,
  State,
  StoppedAt,
  Target,
}

impl Label for TaskLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Healthy => "healthy",
      Self::HostIpAddress => "host ip address",
      Self::_LastestLog => "latest log",
      Self::LastUpdateAt => "last update",
      Self::StagedAt => "staged",
      Self::StartedAt => "started",
      Self::State => "state",
      Self::StoppedAt => "stopped",
      Self::Target => "task id",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::Healthy => "healthy",
      Self::HostIpAddress => "host",
      Self::_LastestLog => "log",
      Self::LastUpdateAt => "update",
      Self::StagedAt => "staged",
      Self::StartedAt => "started",
      Self::State => "state",
      Self::StoppedAt => "stopped",
      Self::Target => "task id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<TaskLabel> for TaskStatus {
  fn value(&self, label: &TaskLabel, task_id: &str) -> String {
    let task: Option<Task> = match self.actual.clone() {
      Some(actual) => Some(actual),
      None => self.configuration.clone(),
    };
    match task {
      Some(task) => match label {
        TaskLabel::Healthy => task.healthy.map(|healthy| healthy.to_string()).unwrap_or_default(),
        TaskLabel::HostIpAddress => task.host.to_string(),
        TaskLabel::_LastestLog => task.logs.map(|log| log.to_string()).unwrap_or_default(),
        TaskLabel::LastUpdateAt => task
          .last_update
          .and_then(|update| DateTime::from_timestamp_millis(update).map(|ts| ts.to_string()))
          .unwrap_or_default(),
        TaskLabel::StagedAt => task.staged_at.to_string(),
        TaskLabel::StartedAt => task.started_at.to_string(),
        TaskLabel::State => task.state.to_string(),
        TaskLabel::StoppedAt => task.stopped_at.map(|update| update.to_string()).unwrap_or_default(),
        TaskLabel::Target => task_id.to_string(),
      },
      None => "".to_string(),
    }
  }
}

pub static TASK_LABELS_LIST: [TaskLabel; 8] =
  [TaskLabel::StartedAt, TaskLabel::State, TaskLabel::Healthy, TaskLabel::Target, TaskLabel::HostIpAddress, TaskLabel::LastUpdateAt, TaskLabel::StagedAt, TaskLabel::StoppedAt];
