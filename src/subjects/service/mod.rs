pub(crate) mod capabilities;
pub(crate) mod labels;

use crate::arguments::service_id_argument;
use crate::capability::{
  Capability, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, DUPLICATE_COMMAND, EDIT_COMMAND, EXPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS,
  OPEN_COMMAND, OPEN_COMMAND_ALIAS, RESTART_COMMAND, SHOW_COMMAND, SHOW_COMMAND_ALIAS, START_COMMAND, STOP_COMMAND, UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::filter_flags::FilterFlagType;
use crate::flags::FlagType;
use crate::subject::Subject;
use crate::subjects::service::capabilities::{
  cpus_option, instances_option, mem_option, ServiceCreate, ServiceDelete, ServiceDuplicate, ServiceEdit, ServiceExport, ServiceListAll, ServiceListAllocationStatus,
  ServiceListIds, ServiceOpen, ServiceRestart, ServiceShow, ServiceShowAllocationStatus, ServiceStart, ServiceStop, ServiceUpdate,
};
use crate::COMMAND_OPTIONS_HEADING;
use async_trait::async_trait;
use clap::{Arg, ArgAction};
use std::sync::LazyLock;

struct ServiceSubject {}

const SERVICE_SUBJECT_TARGET: &str = "service";

pub(crate) static SERVICE_SUBJECT: LazyLock<Box<dyn Subject + Send + Sync>> = LazyLock::new(|| Box::new(ServiceSubject {}));

static SERVICE_CREATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &ServiceCreate {}, "Create service")
      .set_long_about(
        "Create a new service. You will be prompted for the json configuration file. \
        You can also use piping to provide the json file to the command.",
      )
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_DELETE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &ServiceDelete {}, "Delete service")
      .set_long_about("Deletes a service from the DSH platform.")
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_DUPLICATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DUPLICATE_COMMAND, None, &ServiceDuplicate {}, "Duplicate service configuration")
      .set_long_about("Duplicate a service configuration and update it using your default editor.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(Arg::new("verbatim-flag").long("verbatim").action(ArgAction::SetTrue).help("Verbatim duplicate")),
  )
});
static SERVICE_EDIT_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(EDIT_COMMAND, None, &ServiceEdit {}, "Edit service configuration")
      .set_long_about("Edit the service configuration using your default editor.")
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_EXPORT_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(EXPORT_COMMAND, None, &ServiceExport {}, "Export service configuration")
      .set_long_about("Export the service configuration file.")
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_LIST_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ServiceListAll {}, "List services")
      .set_long_about(
        "Lists all DSH services. \
        This will also include services that are stopped \
        (deployed with 0 instances).",
      )
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &ServiceListAllocationStatus {}, None),
        (FlagType::Ids, &ServiceListIds {}, None),
      ])
      .add_filter_flags(vec![
        (FilterFlagType::Started, Some("List all started services.".to_string())),
        (FilterFlagType::Stopped, Some("List all stopped services.".to_string())),
      ]),
  )
});
static SERVICE_OPEN_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(OPEN_COMMAND, Some(OPEN_COMMAND_ALIAS), &ServiceOpen {}, "Open service vhost")
      .set_long_about("Open the vhost of a DSH service.")
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_RESTART_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(RESTART_COMMAND, None, &ServiceRestart {}, "Restart service")
      .set_long_about("Restarts an already running service.")
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_SHOW_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ServiceShow {}, "Show service configuration")
      .set_long_about("Show the configuration of a DSH service.")
      .add_command_executors(vec![(FlagType::AllocationStatus, &ServiceShowAllocationStatus {}, None)])
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_START_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(START_COMMAND, None, &ServiceStart {}, "Start service")
      .set_long_about("Start a DSH service.")
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(instances_option().help_heading(COMMAND_OPTIONS_HEADING)),
  )
});
static SERVICE_STOP_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(STOP_COMMAND, None, &ServiceStop {}, "Stop service")
      .set_long_about("Stop a running DSH service, by setting the number of instances to 0.")
      .add_target_argument(service_id_argument().required(true)),
  )
});
static SERVICE_UPDATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &ServiceUpdate {}, "Update service")
      .set_long_about(
        "Update a DSH service. When you provide the number of cpus, instances or the amount of \
        memory as command line arguments only those parameters will be updated. If you omit these \
        arguments you will be prompted for the json configuration file. You can also use piping \
        to provide the json file to the command.",
      )
      .add_target_argument(service_id_argument().required(true))
      .add_extra_argument(cpus_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(instances_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(mem_option().help_heading(COMMAND_OPTIONS_HEADING)),
  )
});

static SERVICE_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
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
    SERVICE_UPDATE_CAPABILITY.as_ref(),
  ]
});

#[async_trait]
impl Subject for ServiceSubject {
  fn subject(&self) -> &'static str {
    SERVICE_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list services deployed on the DSH.".to_string()
  }

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
