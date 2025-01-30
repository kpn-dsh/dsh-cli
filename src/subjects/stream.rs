use crate::subject::Requirements;
use clap::ArgMatches;

// use std::collections::HashMap;
//
// use async_trait::async_trait;
// use clap::ArgMatches;
// use futures::{join, try_join};
// use lazy_static::lazy_static;
//
// use dsh_api::dsh_api_client::DshApiClient;
// use dsh_api::types::Application;
//
// use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
// use crate::flags::FlagType;
// use crate::formatters::formatter::{print_vec, TableBuilder};
// use crate::formatters::stream::{ManagedStream, INTERNAL_STREAM_LABELS, PUBLIC_STREAM_LABELS, STREAM_LABELS};
// use crate::formatters::usage::{Usage, UsageLabel, USAGE_LABELS_LIST, USAGE_LABELS_SHOW};
// use crate::subject::Subject;
// use crate::subjects::topic::applications_that_use_topic;
// use crate::{Context, DshCliResult};
//
// pub(crate) struct StreamSubject {}
//
// const STREAM_SUBJECT_TARGET: &str = "stream";
//
// lazy_static! {
//   pub static ref STREAM_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(StreamSubject {});
// }
//
// #[async_trait]
// impl Subject for StreamSubject {
//   fn subject(&self) -> &'static str {
//     STREAM_SUBJECT_TARGET
//   }
//
//   fn subject_first_upper(&self) -> &'static str {
//     "Stream"
//   }
//
//   fn subject_command_about(&self) -> String {
//     "Show, manage and list internal and public DSH streams.".to_string()
//   }
//
//   fn subject_command_long_about(&self) -> String {
//     "Show, manage and list internal and public topics deployed on the DSH.".to_string()
//   }
//
//   fn subject_command_name(&self) -> &str {
//     self.subject()
//   }
//
//   fn requirements(&self, sub_matches: &ArgMatches) -> Requirements {
//     Requirements::new(true, None)
//   }
//
//   fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
//     let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
//     // capabilities.insert(CapabilityType::Create, STREAM_CREATE_CAPABILITY.as_ref());
//     capabilities.insert(CapabilityType::List, STREAM_LIST_CAPABILITY.as_ref());
//     capabilities.insert(CapabilityType::Show, STREAM_SHOW_CAPABILITY.as_ref());
//     capabilities
//   }
// }
//
// lazy_static! {
//   // pub static ref STREAM_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
//   //   capability_type: CapabilityType::Create,
//   //   command_about: "Create stream".to_string(),
//   //   command_long_about: Some("Create a internal or public stream.".to_string()),
//   //   command_after_help: None,
//   //   command_after_long_help: None,
//   //   command_executors: vec![],
//   //   // default_command_executor: Some(&StreamCreate {}),
//   //   run_all_executors: false,
//   //   extra_arguments: vec![],
//   //   extra_flags: vec![],
//   // });
//   pub static ref STREAM_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
//     capability_type: CapabilityType::List,
//     command_about: "List streams".to_string(),
//     command_long_about: Some("Lists all available internal and public streams.".to_string()),
//     command_after_help: None,
//     command_after_long_help: None,
//     command_executors: vec![
//       (FlagType::All, &StreamListConfiguration {}, None),
//       (FlagType::Configuration, &StreamListConfiguration {}, None),
//       (FlagType::Ids, &StreamListIds {}, None),
//       (FlagType::Usage, &StreamListUsage {}, None),
//     ],
//     default_command_executor: Some(&StreamListIds {}),
//     run_all_executors: true,
//     extra_arguments: vec![],
//     filter_flags: vec![],
//   });
//   pub static ref STREAM_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
//     capability_type: CapabilityType::Show,
//     command_about: "Show stream configuration".to_string(),
//     command_long_about: None,
//     command_after_help: None,
//     command_after_long_help: None,
//     command_executors: vec![(FlagType::All, &StreamShowAll {}, None), (FlagType::Configuration, &StreamShowConfiguration {}, None), (FlagType::Usage, &StreamShowUsage {}, None),],
//     default_command_executor: Some(&StreamShowAll {}),
//     run_all_executors: false,
//     extra_arguments: vec![],
//     filter_flags: vec![],
//   });
// }
//
// // struct StreamCreate {}
//
// // #[async_trait]
// // impl CommandExecutor for StreamCreate {
// //   async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
// //     let stream_id = target.unwrap_or_else(|| unreachable!());
// //     context.print_capability_explanation(format!("create new internal stream '{}'", stream_id));
// //     if dsh_api_client.get_internal_stream(&stream_id).await.is_ok() {
// //       return Err(format!("internal stream '{}' already exists", stream_id));
// //     }
// //     // let mut line = String::new();
// //     // let stdin = stdin();
// //     // stdin.lock().read_line(&mut line).expect("could not read line");
// //     // println!("{}", line);
// //     let internal_stream = InternalManagedStream { kafka_properties: Default::default(), kind: InternalManagedStreamKind::Internal, partitions: 1, replication_factor: 1 };
// //     dsh_api_client.create_internal_stream(&stream_id, &internal_stream).await?;
// //     println!("ok");
// //     Ok(true)
// //   }
// // }
//
// struct StreamListConfiguration {}
//
// #[async_trait]
// impl CommandExecutor for StreamListConfiguration {
//   async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
//     context.print_explanation("list all internal and public streams");
//     let (stream_ids, internal_streams, public_streams) = try_join!(
//       dsh_api_client.get_stream_ids(),
//       dsh_api_client.get_internal_streams(),
//       dsh_api_client.get_public_streams()
//     )?;
//     let mut builder = TableBuilder::list(&STREAM_LABELS, context);
//     for stream_id in stream_ids {
//       if let Some(internal_stream) = internal_streams.get(stream_id.as_str()) {
//         builder.value(stream_id.to_string(), &ManagedStream::Internal(internal_stream));
//       }
//       if let Some(public_stream) = public_streams.get(stream_id.as_str()) {
//         builder.value(stream_id.to_string(), &ManagedStream::Public(public_stream));
//       }
//     }
//     builder.print();
//     Ok(false)
//   }
// }
//
// struct StreamListIds {}
//
// #[async_trait]
// impl CommandExecutor for StreamListIds {
//   async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
//     context.print_explanation("list all internal and public stream ids");
//     print_vec("stream ids".to_string(), dsh_api_client.get_stream_ids().await?, context);
//     Ok(false)
//   }
// }
//
// struct StreamListUsage {}
//
// #[async_trait]
// impl CommandExecutor for StreamListUsage {
//   async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
//     context.print_explanation("list all internal and public streams with the applications that use them");
//     let (stream_ids, applications) = try_join!(dsh_api_client.get_stream_ids(), dsh_api_client.get_application_configurations())?;
//     let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_LABELS_LIST, context);
//     for stream_id in &stream_ids {
//       let mut first = true;
//       let usages: Vec<(String, Vec<String>)> = applications_that_use_topic(stream_id, &applications);
//       for (application_id, envs) in usages {
//         if !envs.is_empty() {
//           if first {
//             builder.row(&Usage::application(stream_id.to_string(), application_id, envs));
//           } else {
//             builder.row(&Usage::application("".to_string(), application_id, envs));
//           }
//           first = false;
//         }
//       }
//     }
//     builder.print();
//     Ok(false)
//   }
// }
//
// struct StreamShowAll {}
//
// #[async_trait]
// impl CommandExecutor for StreamShowAll {
//   async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
//     let stream_id = target.unwrap_or_else(|| unreachable!());
//     let (internal_stream, public_stream) = join!(
//       dsh_api_client.get_internal_stream(stream_id.as_str()),
//       dsh_api_client.get_public_stream(stream_id.as_str())
//     );
//     if let Ok(internal_stream) = internal_stream {
//       let mut builder = TableBuilder::show(&INTERNAL_STREAM_LABELS, context);
//       builder.value(stream_id.clone(), &internal_stream);
//       builder.print();
//     }
//     if let Ok(public_stream) = public_stream {
//       let mut builder = TableBuilder::show(&PUBLIC_STREAM_LABELS, context);
//       builder.value(stream_id, &public_stream);
//       builder.print();
//     }
//     Ok(false)
//   }
// }
//
// struct StreamShowConfiguration {}
//
// #[async_trait]
// impl CommandExecutor for StreamShowConfiguration {
//   async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
//     let topic_id = target.unwrap_or_else(|| unreachable!());
//     context.print_explanation(format!("show the configuration for topic '{}'", topic_id));
//     let topic = dsh_api_client.get_topic_configuration(topic_id.as_str()).await?;
//     println!("{:#?}", topic);
//     Ok(false)
//   }
// }
//
// struct StreamShowUsage {}
//
// #[async_trait]
// impl CommandExecutor for StreamShowUsage {
//   async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context, dsh_api_client: &DshApiClient<'_>) -> DshCliResult {
//     let topic_id = target.unwrap_or_else(|| unreachable!());
//     context.print_explanation(format!("show the applications that use topic '{}'", topic_id));
//     let applications = dsh_api_client.get_application_configurations().await?;
//     let usages: Vec<(String, Vec<String>)> = applications_that_use_topic(topic_id.as_str(), &applications);
//     if !usages.is_empty() {
//       let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_LABELS_SHOW, context);
//       for (application_id, envs) in usages {
//         builder.row(&Usage::application(application_id.clone(), application_id.to_string(), envs));
//       }
//       builder.print();
//     } else {
//       println!("topic not used")
//     }
//     Ok(false)
//   }
// }
//
// fn _applications_that_use_stream(topic_id: &str, applications: &HashMap<String, Application>) -> Vec<(String, Vec<String>)> {
//   let mut application_ids: Vec<String> = applications.keys().map(|p| p.to_string()).collect();
//   application_ids.sort();
//   let mut pairs: Vec<(String, Vec<String>)> = vec![];
//   for application_id in application_ids {
//     let application = applications.get(&application_id).unwrap();
//     if !application.env.is_empty() {
//       let mut envs_that_contain_topic_id: Vec<String> = application.env.clone().into_iter().filter(|(_, v)| v.contains(topic_id)).map(|(k, _)| k).collect();
//       envs_that_contain_topic_id.sort();
//       pairs.push((application_id.clone(), envs_that_contain_topic_id));
//     }
//   }
//   pairs
// }

// use dsh_api::types::{InternalManagedStream, PublicManagedStream, PublicManagedStreamContractPartitioner};
//
// use crate::formatters::formatter::{Label, SubjectFormatter};
//
// #[derive(Eq, Hash, PartialEq)]
// pub enum ManagedStreamLabel {
//   CanBeRetained,
//   KafkaProperties,
//   Kind,
//   Partitioner,
//   Partitions,
//   ReplicationFactor,
//   Target,
// }
//
// impl Label for ManagedStreamLabel {
//   fn label_for_list(&self) -> &str {
//     match self {
//       Self::CanBeRetained => "ret",
//       Self::KafkaProperties => "props",
//       Self::Kind => "kind",
//       Self::Partitioner => "partnr",
//       Self::Partitions => "parts",
//       Self::ReplicationFactor => "repl",
//       Self::Target => "id",
//     }
//   }
//
//   fn label_for_show(&self) -> &str {
//     match self {
//       Self::CanBeRetained => "can be retained",
//       Self::KafkaProperties => "kafka properties",
//       Self::Kind => "kind",
//       Self::Partitioner => "partitioner",
//       Self::Partitions => "partitions",
//       Self::ReplicationFactor => "replication factor",
//       Self::Target => "stream id",
//     }
//   }
//
//   fn is_target_label(&self) -> bool {
//     *self == ManagedStreamLabel::Target
//   }
// }
//
// pub enum ManagedStream<'a> {
//   Internal(&'a InternalManagedStream),
//   Public(&'a PublicManagedStream),
// }
//
// impl SubjectFormatter<ManagedStreamLabel> for ManagedStream<'_> {
//   fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
//     match self {
//       ManagedStream::Internal(internal) => internal.value(label, target_id),
//       ManagedStream::Public(public) => public.value(label, target_id),
//     }
//   }
//
//   fn target_label(&self) -> Option<ManagedStreamLabel> {
//     match self {
//       ManagedStream::Internal(internal) => internal.target_label(),
//       ManagedStream::Public(public) => public.target_label(),
//     }
//   }
// }
//
// impl SubjectFormatter<ManagedStreamLabel> for InternalManagedStream {
//   fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
//     match label {
//       ManagedStreamLabel::CanBeRetained => "NA".to_string(),
//       ManagedStreamLabel::KafkaProperties => "PROPERTIES".to_string(),
//       ManagedStreamLabel::Kind => self.kind.to_string(),
//       ManagedStreamLabel::Partitioner => "NA".to_string(),
//       ManagedStreamLabel::Partitions => self.partitions.to_string(),
//       ManagedStreamLabel::ReplicationFactor => self.replication_factor.to_string(),
//       ManagedStreamLabel::Target => target_id.to_string(),
//     }
//   }
//
//   fn target_label(&self) -> Option<ManagedStreamLabel> {
//     Some(ManagedStreamLabel::Target)
//   }
// }
//
// impl SubjectFormatter<ManagedStreamLabel> for PublicManagedStream {
//   fn value(&self, label: &ManagedStreamLabel, target_id: &str) -> String {
//     match label {
//       ManagedStreamLabel::CanBeRetained => self.contract.can_be_retained.to_string(),
//       ManagedStreamLabel::KafkaProperties => "PROPERTIES".to_string(),
//       ManagedStreamLabel::Kind => self.kind.to_string(),
//       ManagedStreamLabel::Partitioner => match &self.contract.partitioner {
//         PublicManagedStreamContractPartitioner::TopicLevelPartitioner(partitioner) => format!("topic level partitioner ({})", partitioner.topic_level),
//         PublicManagedStreamContractPartitioner::KafkaDefaultPartitioner(_) => "kafka default partitioner".to_string(),
//       },
//       ManagedStreamLabel::Partitions => self.partitions.to_string(),
//       ManagedStreamLabel::ReplicationFactor => self.replication_factor.to_string(),
//       ManagedStreamLabel::Target => target_id.to_string(),
//     }
//   }
//
//   fn target_label(&self) -> Option<ManagedStreamLabel> {
//     Some(ManagedStreamLabel::Target)
//   }
// }
//
// pub static INTERNAL_STREAM_LABELS: [ManagedStreamLabel; 4] =
//   [ManagedStreamLabel::Target, ManagedStreamLabel::Partitions, ManagedStreamLabel::ReplicationFactor, ManagedStreamLabel::KafkaProperties];
//
// pub static PUBLIC_STREAM_LABELS: [ManagedStreamLabel; 7] = [
//   ManagedStreamLabel::Target,
//   ManagedStreamLabel::Partitions,
//   ManagedStreamLabel::ReplicationFactor,
//   ManagedStreamLabel::Kind,
//   ManagedStreamLabel::Partitioner,
//   ManagedStreamLabel::CanBeRetained,
//   ManagedStreamLabel::KafkaProperties,
// ];
//
// pub static STREAM_LABELS: [ManagedStreamLabel; 7] = [
//   ManagedStreamLabel::Target,
//   ManagedStreamLabel::Partitions,
//   ManagedStreamLabel::ReplicationFactor,
//   ManagedStreamLabel::Kind,
//   ManagedStreamLabel::Partitioner,
//   ManagedStreamLabel::CanBeRetained,
//   ManagedStreamLabel::KafkaProperties,
// ];
