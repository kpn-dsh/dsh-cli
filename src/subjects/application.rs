use crate::arguments::application_id_argument;
use crate::capability::{
  Capability, CommandExecutor, DELETE_COMMAND, DEPLOY_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS, START_COMMAND, STOP_COMMAND, UPDATE_COMMAND,
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
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use chrono::DateTime;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::application::parse_image_string;
use dsh_api::types::Application;
use dsh_api::types::{Task, TaskStatus};
use dsh_api::DshApiError;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

pub(crate) struct ApplicationSubject {}

const APPLICATION_SUBJECT_TARGET: &str = "application";

lazy_static! {
  pub static ref APPLICATION_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ApplicationSubject {});
}

#[async_trait]
impl Subject for ApplicationSubject {
  fn subject(&self) -> &'static str {
    APPLICATION_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list applications deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("a")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(APPLICATION_DELETE_CAPABILITY.as_ref()),
      DEPLOY_COMMAND => Some(APPLICATION_DEPLOY_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(APPLICATION_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(APPLICATION_SHOW_CAPABILITY.as_ref()),
      START_COMMAND => Some(APPLICATION_START_CAPABILITY.as_ref()),
      STOP_COMMAND => Some(APPLICATION_STOP_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(APPLICATION_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &APPLICATION_CAPABILITIES
  }
}

lazy_static! {
  static ref APPLICATION_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, "Delete application")
      .set_long_about("Deletes an application from the DSH platform.")
      .set_default_command_executor(&ApplicationDelete {})
      .add_target_argument(application_id_argument().required(true))
  );
  static ref APPLICATION_DEPLOY_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DEPLOY_COMMAND, None, "Deploy application")
      .set_long_about("Deploy a new application.")
      .set_default_command_executor(&ApplicationDeploy {})
      .add_target_argument(application_id_argument().required(true))
      .add_extra_argument(instances_flag())
  );
  static ref APPLICATION_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List applications")
      .set_long_about(
        "Lists all deployed DSH applications. \
        This will also include applications that are stopped \
        (deployed with 0 instances)."
      )
      .set_default_command_executor(&ApplicationListAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ApplicationListAllocationStatus {}, None),
        (FlagType::Ids, &ApplicationListIds {}, None),
        (FlagType::Tasks, &ApplicationListTasks {}, None),
      ])
      .set_run_all_executors(true)
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List all started applications.".to_string())),
        (FilterFlagType::Stopped, Some("List all stopped applications.".to_string()))
      ])
  );
  static ref APPLICATION_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), "Show application configuration")
      .set_long_about("Show the configuration of an application deployed on the DSH.")
      .set_default_command_executor(&ApplicationShowAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ApplicationShowAllocationStatus {}, None),
        (FlagType::Tasks, &ApplicationShowTasks {}, None),
      ])
      .add_target_argument(application_id_argument().required(true))
  );
  static ref APPLICATION_START_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(START_COMMAND, None, "Start application")
      .set_long_about("Starts an already deployed application.")
      .set_default_command_executor(&ApplicationStart {})
      .add_target_argument(application_id_argument().required(true))
      .add_extra_argument(instances_flag())
  );
  static ref APPLICATION_STOP_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(STOP_COMMAND, None, "Stop application")
      .set_long_about("Stops a running application, by setting the number of instances to 0.")
      .set_default_command_executor(&ApplicationStop {})
      .add_target_argument(application_id_argument().required(true))
  );
  static ref APPLICATION_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, "Update application")
      .set_long_about("Update an already deployed application.")
      .set_default_command_executor(&ApplicationUpdate {})
      .add_target_argument(application_id_argument().required(true))
      .add_extra_argument(instances_flag())
  );
  static ref APPLICATION_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    APPLICATION_DELETE_CAPABILITY.as_ref(),
    APPLICATION_DEPLOY_CAPABILITY.as_ref(),
    APPLICATION_LIST_CAPABILITY.as_ref(),
    APPLICATION_SHOW_CAPABILITY.as_ref(),
    APPLICATION_START_CAPABILITY.as_ref(),
    APPLICATION_STOP_CAPABILITY.as_ref(),
    APPLICATION_UPDATE_CAPABILITY.as_ref()
  ];
}

fn instances_flag() -> Arg {
  Arg::new("instances")
    .long("instances")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..))
    .value_name("INSTANCES")
    .help("Number of instances.")
    .long_help("Number of application instances that will be started.")
}

struct ApplicationDelete {}

#[async_trait]
impl CommandExecutor for ApplicationDelete {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("delete application '{}'", application_id));
    if context.client_unchecked().get_application_configuration(&application_id).await.is_err() {
      return Err(format!("application '{}' does not exist", application_id));
    }
    if context.confirmed(format!("type 'yes' to delete application '{}': ", application_id).as_str())? {
      if context.dry_run {
        context.print_warning("dry-run mode, application not deleted");
      } else {
        context.client_unchecked().delete_application_configuration(application_id.as_str()).await?;
        context.print_outcome(format!("application '{}' deleted", application_id));
      }
    } else {
      context.print_outcome(format!("cancelled, application '{}' not deleted", application_id));
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationDeploy {}

#[async_trait]
impl CommandExecutor for ApplicationDeploy {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    if context.client_unchecked().get_application_configuration(&application_id).await.is_ok() {
      return Err(format!("application '{}' already exists", application_id));
    }
    context.print_explanation(format!("deploy application '{}'", application_id));
    let configuration = context.read_multi_line("enter json configuration text (terminate input with ctrl-d after last line)")?;
    match serde_json::from_str::<Application>(&configuration) {
      Ok(application) => {
        if context.dry_run {
          context.print_warning("dry-run mode, application not deployed");
        } else {
          context.client_unchecked().put_application_configuration(&application_id, &application).await?;
          context.print_outcome(format!("application '{}' deployed", application_id));
        }
        Ok(())
      }
      Err(error) => Err(format!("invalid json configuration ({})", error)),
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationListAll {}

#[async_trait]
impl CommandExecutor for ApplicationListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all deployed services with their parameters");
    let start_instant = Instant::now();
    let applications = context.client_unchecked().get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let mut application_ids = applications.keys().map(|k| k.to_string()).collect::<Vec<_>>();
    application_ids.sort();
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut formatter = ListFormatter::new(&APPLICATION_LABELS_LIST, None, context);
    for application_id in application_ids {
      if let Some(application) = applications.get(&application_id) {
        if (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped) {
          formatter.push_target_id_value(application_id.clone(), application);
        }
      };
    }
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationListAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all applications with their allocation status");
    let start_instant = Instant::now();
    let application_ids = context.client_unchecked().list_application_ids().await?;
    let allocation_statuses = try_join_all(
      application_ids
        .iter()
        .map(|application_id| context.client_unchecked().get_application_status(application_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("application id"), context);
    formatter.push_target_ids_and_values(application_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationListIds {}

#[async_trait]
impl CommandExecutor for ApplicationListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all application ids");
    let start_instant = Instant::now();
    let ids = context.client_unchecked().list_application_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("application id", context);
    formatter.push_target_ids(ids.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(Some(OutputFormat::Plain))
  }
}

struct ApplicationListTasks {}

#[async_trait]
impl CommandExecutor for ApplicationListTasks {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
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
    context.print_explanation("list all applications with their tasks");
    let start_instant = Instant::now();
    let application_ids = context.client_unchecked().get_task_ids().await?;
    let tasks: Vec<Vec<String>> = try_join_all(
      application_ids
        .iter()
        .map(|application_id| context.client_unchecked().get_task_appid_ids(application_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let application_id_tasks_pairs: Vec<(String, String)> = application_ids
      .iter()
      .zip(tasks)
      .map(|(id, tasks)| (id.to_string(), tasks_to_string(tasks)))
      .collect::<Vec<_>>();
    let mut formatter: ListFormatter<ApplicationLabel, (String, String)> = ListFormatter::new(&[ApplicationLabel::Target, ApplicationLabel::Tasks], None, context);
    formatter.push_values(&application_id_tasks_pairs);
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationShowAll {}

#[async_trait]
impl CommandExecutor for ApplicationShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for application '{}'", application_id));
    let start_instant = Instant::now();
    let application = context.client_unchecked().get_application_configuration(application_id.as_str()).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(application_id, &APPLICATION_LABELS_SHOW, Some("application id"), context).print(&application)
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for ApplicationShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for application '{}'", application_id));
    let start_instant = Instant::now();
    let allocation_status = context.client_unchecked().get_application_status(application_id.as_str()).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(application_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("application id"), context).print(&allocation_status)
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationShowTasks {}

#[async_trait]
impl CommandExecutor for ApplicationShowTasks {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all tasks for application '{}'", application_id));
    let start_instant = Instant::now();
    let task_ids = context.client_unchecked().get_task_appid_ids(application_id.as_str()).await?;
    let task_statuses = try_join_all(
      task_ids
        .iter()
        .map(|task_id| context.client_unchecked().get_task(application_id.as_str(), task_id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut tasks: Vec<(String, TaskStatus)> = task_ids.into_iter().zip(task_statuses).collect();
    tasks.sort_by(|first, second| second.1.actual.clone().unwrap().staged_at.cmp(&first.1.actual.clone().unwrap().staged_at));
    let mut formatter = ListFormatter::new(&TASK_LABELS_LIST, Some("application id"), context);
    formatter.push_target_id_value_pairs(tasks.as_slice());
    formatter.print()?;
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationStart {}

#[async_trait]
impl CommandExecutor for ApplicationStart {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    let instances: u64 = matches.get_one("instances").cloned().unwrap_or(1);
    if instances == 1 {
      context.print_explanation(format!("start service '{}'", application_id));
    } else {
      context.print_explanation(format!("start {} instances of service '{}'", instances, application_id));
    }
    match context.client_unchecked().get_application_configuration(application_id.as_str()).await {
      Ok(mut configuration) => {
        if configuration.instances > 0 {
          context.print_warning(format!("service '{}' already started", application_id));
        } else {
          configuration.instances = instances;
          context
            .client_unchecked()
            .put_application_configuration(application_id.as_str(), &configuration)
            .await?;
          if instances == 1 {
            context.print_outcome(format!("service '{}' started", application_id));
          } else {
            context.print_outcome(format!("service '{}' started ({} instances)", application_id, instances));
          }
        }
        Ok(())
      }
      Err(error) => match error {
        DshApiError::NotFound => {
          context.print_error(format!("service '{}' is not deployed", application_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationStop {}

#[async_trait]
impl CommandExecutor for ApplicationStop {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("stop service '{}'", application_id));
    match context.client_unchecked().get_application_configuration(application_id.as_str()).await {
      Ok(mut configuration) => {
        let running_instances = configuration.instances;
        if running_instances == 0 {
          context.print_warning(format!("service '{}' not running", application_id));
        } else {
          configuration.instances = 0;
          context
            .client_unchecked()
            .put_application_configuration(application_id.as_str(), &configuration)
            .await?;
          if running_instances == 1 {
            context.print_outcome(format!("service '{}' stopped", application_id));
          } else {
            context.print_outcome(format!("service '{}' stopped ({} instances)", application_id, running_instances));
          }
        }
        Ok(())
      }
      Err(error) => match error {
        DshApiError::NotFound => {
          context.print_error(format!("service '{}' is not deployed", application_id));
          Ok(())
        }
        error => Err(String::from(error)),
      },
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApplicationUpdate {}

#[async_trait]
impl CommandExecutor for ApplicationUpdate {
  async fn execute(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let application_id = target.unwrap_or_else(|| unreachable!());
    let _instances: u64 = matches.get_one("instances").cloned().unwrap_or(1);
    context.print_explanation(format!("update service '{}'", application_id));
    Ok(())
    // match context.client_unchecked().get_application_configuration(application_id.as_str()).await
    // {
    //   Ok(mut configuration) => {
    //     if configuration.instances > 0 {
    //       context.print_warning(format!("service '{}' already started", application_id));
    //     } else {
    //       configuration.instances = instances;
    //       context.put_application_configuration(application_id.as_str(), &configuration).await?;
    //       if instances == 1 {
    //         context.print_outcome(format!("service '{}' started", application_id));
    //       } else {
    //         context.print_outcome(format!("service '{}' started ({} instances)", application_id, instances));
    //       }
    //     }
    //     Ok(())
    //   }
    //   Err(error) => match error {
    //     DshApiError::NotFound => {
    //       context.print_error(format!("service '{}' is not deployed", application_id));
    //       Ok(())
    //     }
    //     error => Err(String::from(error)),
    //   },
    // }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ApplicationLabel {
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

impl Label for ApplicationLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Target => "application id",
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
      Self::Target => "application id",
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

impl SubjectFormatter<ApplicationLabel> for Application {
  fn value(&self, label: &ApplicationLabel, application_id: &str) -> String {
    match label {
      ApplicationLabel::Cpus => self.cpus.to_string(),
      ApplicationLabel::Env => hashmap_to_table(&self.env),
      ApplicationLabel::ExposedPorts => self.exposed_ports.keys().map(|port| port.to_string()).collect::<Vec<_>>().join(","),
      ApplicationLabel::HealthCheck => match self.health_check {
        Some(ref health_check) => match health_check.protocol {
          Some(protocol) => format!("{}:{}/{}", protocol.to_string(), health_check.port, health_check.path),
          None => format!("{}/{}", health_check.port, health_check.path),
        },
        None => "".to_string(),
      },
      ApplicationLabel::Image => match parse_image_string(self.image.as_str()) {
        Ok((kind, image)) => format!("{}:{}", kind, image),
        Err(_) => self.image.clone(),
      },
      ApplicationLabel::Instances => self.instances.to_string(),
      ApplicationLabel::Mem => self.mem.to_string(),
      ApplicationLabel::Metrics => self
        .metrics
        .clone()
        .map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))
        .unwrap_or_default(),
      ApplicationLabel::NeedsToken => self.needs_token.to_string(),
      ApplicationLabel::ReadableStreams => self
        .readable_streams
        .clone()
        .into_iter()
        .map(|readable_stream| readable_stream.to_string())
        .collect::<Vec<_>>()
        .join(", "),
      ApplicationLabel::Secrets => self.secrets.clone().into_iter().map(|secret| secret.name).collect::<Vec<_>>().join(", "),
      ApplicationLabel::SingleInstance => self.single_instance.to_string(),
      ApplicationLabel::SpreadGroup => self.spread_group.clone().unwrap_or_default(),
      ApplicationLabel::Target => application_id.to_string(),
      ApplicationLabel::Tasks => "".to_string(),
      ApplicationLabel::Topics => self.topics.clone().into_iter().map(|topic| topic.to_string()).collect::<Vec<_>>().join(", "),
      ApplicationLabel::User => self.user.clone(),
      ApplicationLabel::Volumes => self.volumes.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(","),
      ApplicationLabel::WritableStreams => self
        .writable_streams
        .clone()
        .into_iter()
        .map(|writable_stream| writable_stream.to_string())
        .collect::<Vec<_>>()
        .join(", "),
    }
  }
}

pub static APPLICATION_LABELS_LIST: [ApplicationLabel; 8] = [
  ApplicationLabel::Target,
  ApplicationLabel::NeedsToken,
  ApplicationLabel::Instances,
  ApplicationLabel::Cpus,
  ApplicationLabel::Mem,
  ApplicationLabel::ExposedPorts,
  ApplicationLabel::Metrics,
  ApplicationLabel::Image,
];

pub static APPLICATION_LABELS_SHOW: [ApplicationLabel; 18] = [
  ApplicationLabel::Target,
  ApplicationLabel::NeedsToken,
  ApplicationLabel::Instances,
  ApplicationLabel::Cpus,
  ApplicationLabel::Mem,
  ApplicationLabel::ExposedPorts,
  ApplicationLabel::Volumes,
  ApplicationLabel::Metrics,
  ApplicationLabel::Image,
  ApplicationLabel::HealthCheck,
  ApplicationLabel::ReadableStreams,
  ApplicationLabel::WritableStreams,
  ApplicationLabel::Secrets,
  ApplicationLabel::SingleInstance,
  ApplicationLabel::SpreadGroup,
  ApplicationLabel::Topics,
  ApplicationLabel::User,
  ApplicationLabel::Env,
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
