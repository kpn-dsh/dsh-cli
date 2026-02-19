use crate::arguments::nodepool_id_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{NodeFeatures, NodepoolActual};
use dsh_api::DependantApplication;
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;

struct NodepoolSubject {}

const NODE_POOL_SUBJECT_TARGET: &str = "nodepool";

lazy_static! {
  pub(crate) static ref NODE_POOL_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(NodepoolSubject {});
}

#[async_trait]
impl Subject for NodepoolSubject {
  fn subject(&self) -> &'static str {
    NODE_POOL_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show node pool resources.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show node pools deployed on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      LIST_COMMAND => Some(NODE_POOL_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(NODE_POOL_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &NODE_POOL_CAPABILITIES
  }
}

lazy_static! {
  static ref NODE_POOL_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &NodepoolList {}, "List node pools")
      .set_long_about("Lists all available node pools.")
      .add_command_executors(vec![(FlagType::Ids, &NodepoolListIds {}, None), (FlagType::Usage, &NodepoolListUsage {}, None)])
  );
  static ref NODE_POOL_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &NodepoolShow {}, "Show node pool configuration")
      .add_command_executor(FlagType::Usage, &NodepoolShowUsage {}, None)
      .add_target_argument(nodepool_id_argument().required(true))
  );
  static ref NODE_POOL_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![NODE_POOL_LIST_CAPABILITY.as_ref(), NODE_POOL_SHOW_CAPABILITY.as_ref()];
}

struct NodepoolList {}

#[async_trait]
impl CommandExecutor for NodepoolList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all node pools with theit parameters");
    let start_instant = context.now();
    let nodepools: Vec<(String, NodepoolActual)> = client.nodepools().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&NODEPOOL_LABELS, context);
    formatter.push_target_id_value_pairs(&nodepools);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct NodepoolListIds {}

#[async_trait]
impl CommandExecutor for NodepoolListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list node pool ids");
    let start_instant = context.now();
    let nodepool_ids = client.get_nodepool_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("node pool id", context);
    formatter.push_target_ids(nodepool_ids.as_slice());
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct NodepoolListUsage {}

#[async_trait]
impl CommandExecutor for NodepoolListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all node pools that are used in services");
    let start_instant = context.now();
    let nodepools_with_dependant_applications: Vec<(String, NodepoolActual, Vec<DependantApplication<NodeFeatures>>)> = client.nodepools_with_dependant_applications().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&NODEPOOL_USAGE_LABELS, context);
    for (nodepool_id, nodepool_actual, dependant_applications) in &nodepools_with_dependant_applications {
      for DependantApplication { application_id, instances, injections } in dependant_applications {
        formatter.push_target_id_value_owned(nodepool_id.clone(), (nodepool_actual, application_id, instances, injections));
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no node pools found in apps or services");
    } else {
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct NodepoolShow {}

#[async_trait]
impl CommandExecutor for NodepoolShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let nodepool_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for node pool '{}'", nodepool_id));
    let start_instant = context.now();
    let (nodepool, allocation_status) = join!(client.get_nodepool_actual(&nodepool_id), client.get_nodepool_status(&nodepool_id));
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, NODE_POOL_SUBJECT_TARGET);
    UnitFormatter::new(nodepool_id, &NODEPOOL_LABELS, context).print(&nodepool?, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct NodepoolShowUsage {}

#[async_trait]
impl CommandExecutor for NodepoolShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let nodepool_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the services that use node pool '{}'", nodepool_id));
    let start_instant = context.now();
    let (_, dependant_services) = client.nodepool_with_dependant_applications(&nodepool_id).await?;
    context.print_execution_time(start_instant);
    if dependant_services.is_empty() {
      context.print_outcome("node pool not used in services")
    } else {
      context.print_outcome("node pool used in services");
      let mut formatter = ListFormatter::new(&NODEPOOL_USAGE_LABELS, context);
      let a = dependant_services
        .iter()
        .map(|dependant_service| (&dependant_service.application_id, dependant_service.instances))
        .collect_vec();

      formatter.push_values(&a);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum NodepoolLabel {
  GpuDriver,
  MaxInstances,
  NodeFeatures,
  ServiceId,
  ServiceInstances,
  Target,
}

impl Label for NodepoolLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::GpuDriver => "gpu driver",
      Self::MaxInstances => "maximum number of instances",
      Self::NodeFeatures => "node features",
      Self::ServiceId => "service id",
      Self::ServiceInstances => "service instances",
      Self::Target => "node pool id",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::GpuDriver => "driver",
      Self::MaxInstances => "max instances",
      Self::NodeFeatures => "nodes",
      Self::ServiceId => "service id",
      Self::ServiceInstances => "# services",
      Self::Target => "node pool id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<NodepoolLabel> for NodepoolActual {
  fn value(&self, label: &NodepoolLabel, target_id: &str) -> Value {
    match label {
      NodepoolLabel::GpuDriver => Value::plain(self.gpu_driver),
      NodepoolLabel::MaxInstances => Value::plain(self.max_instances),
      NodepoolLabel::Target => Value::target(target_id),
      _ => Value::not_applicable(),
    }
  }
}

impl SubjectFormatter<NodepoolLabel> for (&String, u64) {
  fn value(&self, label: &NodepoolLabel, _: &str) -> Value {
    let (service_id, service_instances) = self;
    match label {
      NodepoolLabel::ServiceId => Value::plain(service_id),
      NodepoolLabel::ServiceInstances => Value::plain(service_instances),
      _ => Value::not_applicable(),
    }
  }
}

impl SubjectFormatter<NodepoolLabel> for (&NodepoolActual, &String, &u64, &Vec<NodeFeatures>) {
  fn value(&self, label: &NodepoolLabel, target_id: &str) -> Value {
    let (nodepool_actual, service_id, service_instances, node_features) = self;
    match label {
      NodepoolLabel::NodeFeatures => Value::plain(node_features.iter().map(|node_feature| node_feature.to_string()).join(", ")),
      NodepoolLabel::ServiceId => Value::target(service_id),
      NodepoolLabel::ServiceInstances => Value::plain(service_instances),
      _ => nodepool_actual.value(label, target_id),
    }
  }
}

static NODEPOOL_USAGE_LABELS: [NodepoolLabel; 4] = [NodepoolLabel::Target, NodepoolLabel::ServiceId, NodepoolLabel::ServiceInstances, NodepoolLabel::NodeFeatures];
static NODEPOOL_LABELS: [NodepoolLabel; 3] = [NodepoolLabel::Target, NodepoolLabel::GpuDriver, NodepoolLabel::MaxInstances];
