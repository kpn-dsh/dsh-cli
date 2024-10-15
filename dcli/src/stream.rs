use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::{join, try_join};
use lazy_static::lazy_static;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::Application;

use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::formatter::{print_ids, TableBuilder};
use crate::formatters::stream::{ManagedStream, INTERNAL_STREAM_LABELS, PUBLIC_STREAM_LABELS, STREAM_LABELS};
use crate::formatters::usage::{Usage, UsageLabel, USAGE_LABELS_LIST, USAGE_LABELS_SHOW};
use crate::subject::Subject;
use crate::topic::applications_that_use_topic;
use crate::{DcliContext, DcliResult};

pub(crate) struct StreamSubject {}

const STREAM_SUBJECT_TARGET: &str = "stream";

lazy_static! {
  pub static ref STREAM_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(StreamSubject {});
}

#[async_trait]
impl Subject for StreamSubject {
  fn subject(&self) -> &'static str {
    STREAM_SUBJECT_TARGET
  }

  fn subject_first_upper(&self) -> &'static str {
    "Stream"
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list internal and public DSH streams.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list internal and public topics deployed on the DSH.".to_string()
  }

  fn subject_command_name(&self) -> &str {
    self.subject()
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    // capabilities.insert(CapabilityType::Create, STREAM_CREATE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, STREAM_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, STREAM_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  // pub static ref STREAM_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
  //   capability_type: CapabilityType::Create,
  //   command_about: "Create stream".to_string(),
  //   command_long_about: Some("Create a internal or public stream.".to_string()),
  //   command_after_help: None,
  //   command_after_long_help: None,
  //   command_executors: vec![],
  //   // default_command_executor: Some(&StreamCreate {}),
  //   run_all_executors: false,
  //   extra_arguments: vec![],
  //   extra_flags: vec![],
  // });
  pub static ref STREAM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List streams".to_string(),
    command_long_about: Some("Lists all available internal and public streams.".to_string()),
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![
      (FlagType::All, &StreamListConfiguration {}, None),
      (FlagType::Configuration, &StreamListConfiguration {}, None),
      (FlagType::Ids, &StreamListIds {}, None),
      (FlagType::Usage, &StreamListUsage {}, None),
    ],
    default_command_executor: Some(&StreamListIds {}),
    run_all_executors: true,
    extra_arguments: vec![],
    filter_flags: vec![],
  });
  pub static ref STREAM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show stream configuration".to_string(),
    command_long_about: None,
    command_after_help: None,
    command_after_long_help: None,
    command_executors: vec![(FlagType::All, &StreamShowAll {}, None), (FlagType::Configuration, &StreamShowConfiguration {}, None), (FlagType::Usage, &StreamShowUsage {}, None),],
    default_command_executor: Some(&StreamShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
  });
}

// struct StreamCreate {}

// #[async_trait]
// impl CommandExecutor for StreamCreate {
//   async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
//     let stream_id = target.unwrap_or_else(|| unreachable!());
//     if context.show_capability_explanation() {
//       println!("create new internal stream '{}'", stream_id);
//     }
//     if dsh_api_client.get_internal_stream(&stream_id).await.is_ok() {
//       return Err(format!("internal stream '{}' already exists", stream_id));
//     }
//     // let mut line = String::new();
//     // let stdin = stdin();
//     // stdin.lock().read_line(&mut line).expect("could not read line");
//     // println!("{}", line);
//     let internal_stream = InternalManagedStream { kafka_properties: Default::default(), kind: InternalManagedStreamKind::Internal, partitions: 1, replication_factor: 1 };
//     dsh_api_client.create_internal_stream(&stream_id, &internal_stream).await?;
//     println!("ok");
//     Ok(true)
//   }
// }

struct StreamListConfiguration {}

#[async_trait]
impl CommandExecutor for StreamListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all internal and public streams");
    }
    let (stream_ids, internal_streams, public_streams) = try_join!(
      dsh_api_client.get_stream_ids(),
      dsh_api_client.get_internal_streams(),
      dsh_api_client.get_public_streams()
    )?;
    let mut builder = TableBuilder::list(&STREAM_LABELS, context);
    for stream_id in stream_ids {
      if let Some(internal_stream) = internal_streams.get(stream_id.as_str()) {
        builder.value(stream_id.to_string(), &ManagedStream::Internal(internal_stream));
      }
      if let Some(public_stream) = public_streams.get(stream_id.as_str()) {
        builder.value(stream_id.to_string(), &ManagedStream::Public(public_stream));
      }
    }
    builder.print();
    Ok(false)
  }
}

struct StreamListIds {}

#[async_trait]
impl CommandExecutor for StreamListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all internal and public stream ids");
    }
    print_ids("stream ids".to_string(), dsh_api_client.get_stream_ids().await?, context);
    Ok(false)
  }
}

struct StreamListUsage {}

#[async_trait]
impl CommandExecutor for StreamListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all internal and public streams with the applications that use them");
    }
    let (stream_ids, applications) = try_join!(dsh_api_client.get_stream_ids(), dsh_api_client.get_application_configurations())?;
    let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_LABELS_LIST, context);
    for stream_id in &stream_ids {
      let mut first = true;
      let usages: Vec<(String, Vec<String>)> = applications_that_use_topic(stream_id, &applications);
      for (application_id, envs) in usages {
        if !envs.is_empty() {
          if first {
            builder.row(&Usage::application(stream_id.to_string(), application_id, envs));
          } else {
            builder.row(&Usage::application("".to_string(), application_id, envs));
          }
          first = false;
        }
      }
    }
    builder.print();
    Ok(false)
  }
}

struct StreamShowAll {}

#[async_trait]
impl CommandExecutor for StreamShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let stream_id = target.unwrap_or_else(|| unreachable!());
    let (internal_stream, public_stream) = join!(
      dsh_api_client.get_internal_stream(stream_id.as_str()),
      dsh_api_client.get_public_stream(stream_id.as_str())
    );
    if let Ok(internal_stream) = internal_stream {
      let mut builder = TableBuilder::show(&INTERNAL_STREAM_LABELS, context);
      builder.value(stream_id.clone(), &internal_stream);
      builder.print();
    }
    if let Ok(public_stream) = public_stream {
      let mut builder = TableBuilder::show(&PUBLIC_STREAM_LABELS, context);
      builder.value(stream_id, &public_stream);
      builder.print();
    }
    Ok(false)
  }
}

struct StreamShowConfiguration {}

#[async_trait]
impl CommandExecutor for StreamShowConfiguration {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the configuration for topic '{}'", topic_id);
    }
    let topic = dsh_api_client.get_topic_configuration(topic_id.as_str()).await?;
    println!("{:#?}", topic);
    Ok(false)
  }
}

struct StreamShowUsage {}

#[async_trait]
impl CommandExecutor for StreamShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext, dsh_api_client: &DshApiClient<'_>) -> DcliResult {
    let topic_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the applications that use topic '{}'", topic_id);
    }
    let applications = dsh_api_client.get_application_configurations().await?;
    let usages: Vec<(String, Vec<String>)> = applications_that_use_topic(topic_id.as_str(), &applications);
    if !usages.is_empty() {
      let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_LABELS_SHOW, context);
      for (application_id, envs) in usages {
        builder.row(&Usage::application(application_id.clone(), application_id.to_string(), envs));
      }
      builder.print();
    } else {
      println!("topic not used")
    }
    Ok(false)
  }
}

fn _applications_that_use_stream(topic_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, Vec<String>)> {
  let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
  application_ids.sort();
  let mut pairs: Vec<(String, Vec<String>)> = vec![];
  for application_id in application_ids {
    let application = applications.get(&application_id).unwrap();
    if !application.env.is_empty() {
      let mut envs_that_contain_topic_id: Vec<String> = application.env.clone().into_iter().filter(|(_, v)| v.contains(topic_id)).map(|(k, _)| k).collect();
      envs_that_contain_topic_id.sort();
      pairs.push((application_id.clone(), envs_that_contain_topic_id));
    }
  }
  pairs
}
