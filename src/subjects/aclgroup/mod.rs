pub(crate) mod labels;
pub(crate) mod options;

use crate::arguments::acl_group_name_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS, GRANT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, REVOKE_COMMAND,
  SHOW_COMMAND, SHOW_COMMAND_ALIAS,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::error::DshCliError;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::OutputFormat;
use crate::subject::{Requirements, Subject};
use crate::subjects::aclgroup::labels::AclGroupLabel;
use crate::subjects::aclgroup::options::{
  read_internal_option, read_public_option, read_topic_option, write_internal_option, write_public_option, write_topic_option, READ_INTERNAL_OPTION, READ_PUBLIC_OPTION,
  READ_TOPIC_OPTION, WRITE_INTERNAL_OPTION, WRITE_PUBLIC_OPTION, WRITE_TOPIC_OPTION,
};
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::DshApiError;
use dsh_api::types::{KafkaAclGroup, KafkaAclGroupTopic, KafkaAclGroupTopicKind};
use futures::future::try_join_all;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::AsRef;
use std::sync::LazyLock;

struct AclGroupSubject {}

const ACL_GROUP_SUBJECT_TARGET: &str = "aclgroup";

lazy_static! {
  pub(crate) static ref ACL_GROUP_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(AclGroupSubject {});
}

#[async_trait]
impl Subject for AclGroupSubject {
  fn subject(&self) -> &'static str {
    ACL_GROUP_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH Kafka proxy ACL groups.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list Kafka proxy ACL groups. Note that the default limit on \
        the number of ACL groups for a tenant is zero, so you likely have to request to raise \
        this limit before being able to use ACL groups."
      .to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(ACL_GROUP_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(ACL_GROUP_DELETE_CAPABILITY.as_ref()),
      GRANT_COMMAND => Some(ACL_GROUP_GRANT_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(ACL_GROUP_LIST_CAPABILITY.as_ref()),
      REVOKE_COMMAND => Some(ACL_GROUP_REVOKE_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(ACL_GROUP_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &ACL_GROUP_CAPABILITIES
  }
}

static ACL_GROUP_CREATE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &AclGroupCreate {}, "Create proxy ACL group on dsh")
      .set_long_about(
        "Create a Kafka proxy ACL group on the DSH platform. Note that the default limit on \
        the number of ACL groups for a tenant is zero, so you likely have to request to raise \
        this limit before being able to use ACL groups.",
      )
      .add_target_argument(acl_group_name_argument().required(true))
      .add_extra_argument(read_internal_option())
      .add_extra_argument(read_public_option())
      .add_extra_argument(read_topic_option())
      .add_extra_argument(write_internal_option())
      .add_extra_argument(write_public_option())
      .add_extra_argument(write_topic_option()),
  )
});
static ACL_GROUP_DELETE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, Some(DELETE_COMMAND_ALIAS), &AclGroupDelete {}, "Delete Kafka ACL group").add_target_argument(acl_group_name_argument().required(true)),
  )
});
static ACL_GROUP_GRANT_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(GRANT_COMMAND, None, &AclGroupGrant {}, "Grant stream access to a proxy ACL group")
      .add_target_argument(acl_group_name_argument().required(true))
      .add_extra_argument(read_internal_option())
      .add_extra_argument(read_public_option())
      .add_extra_argument(read_topic_option())
      .add_extra_argument(write_internal_option())
      .add_extra_argument(write_public_option())
      .add_extra_argument(write_topic_option()),
  )
});
static ACL_GROUP_LIST_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &AclGroupList {}, "List DSH Kafka proxy ACL groups")
      .set_long_about("Lists all Kafka proxy ACL groups configured on the DSH.")
      .add_command_executor(FlagType::Ids, &AclGroupListIds {}, None),
  )
});
static ACL_GROUP_SHOW_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &AclGroupShow {}, "Show Kafka proxy ACL group configuration")
      .add_target_argument(acl_group_name_argument().required(true)),
  )
});
static ACL_GROUP_REVOKE_CAPABILITY: LazyLock<Box<dyn Capability + Send + Sync>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(REVOKE_COMMAND, None, &AclGroupRevoke {}, "Revoke stream access from a proxy ACL group")
      .add_target_argument(acl_group_name_argument().required(true))
      .add_extra_argument(read_internal_option())
      .add_extra_argument(read_public_option())
      .add_extra_argument(read_topic_option())
      .add_extra_argument(write_internal_option())
      .add_extra_argument(write_public_option())
      .add_extra_argument(write_topic_option()),
  )
});

static ACL_GROUP_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
    ACL_GROUP_CREATE_CAPABILITY.as_ref(),
    ACL_GROUP_DELETE_CAPABILITY.as_ref(),
    ACL_GROUP_GRANT_CAPABILITY.as_ref(),
    ACL_GROUP_LIST_CAPABILITY.as_ref(),
    ACL_GROUP_SHOW_CAPABILITY.as_ref(),
    ACL_GROUP_REVOKE_CAPABILITY.as_ref(),
  ]
});

static ACL_GROUP_LABELS: [AclGroupLabel; 4] = [AclGroupLabel::StreamName, AclGroupLabel::Kind, AclGroupLabel::Readable, AclGroupLabel::Writable];

struct AclGroupCreate {}

#[async_trait]
impl CommandExecutor for AclGroupCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let acl_group_name = target.unwrap_or_else(|| unreachable!());
    if client.get_aclgroup_configuration(&acl_group_name).await.is_ok() {
      return err!("acl group '{}' already exists", acl_group_name);
    }
    let (readable_streams, writable_streams) = get_kafka_acl_group_topic_options(matches);
    if readable_streams.is_empty() && writable_streams.is_empty() {
      context.print_warning("no readable or writable streams are provided");
      if !context.confirmed("create empty acl group?")? {
        return err!("cancelled");
      }
    } else {
      let mut formatter = ListFormatter::new(&ACL_GROUP_LABELS, context);
      let streams = get_streams(&readable_streams, &writable_streams).into_iter().map(Some).collect_vec();
      formatter.push_values(&streams);
      formatter.print(None)?;
      if !context.confirmed("create acl group?")? {
        return err!("cancelled");
      }
    }
    if context.dry_run() {
      context.print_warning("dry-run mode, acl group not created");
    } else {
      let kafka_acl_group = KafkaAclGroup { readable_streams, writable_streams };
      client.put_aclgroup_configuration(&acl_group_name, &kafka_acl_group).await?;
      context.print_outcome(format!("acl group '{}' created", acl_group_name));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AclGroupDelete {}

#[async_trait]
impl CommandExecutor for AclGroupDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let acl_group_name = target.unwrap_or_else(|| unreachable!());
    match client.get_aclgroup_configuration(&acl_group_name).await {
      Ok(acl_group) => {
        list_streams(&acl_group_name, &acl_group, context)?;
        if context.confirmed(format!("delete acl group '{}'?", acl_group_name))? {
          if context.dry_run() {
            context.print_warning("dry-run mode, acl group not deleted");
            Ok(())
          } else {
            client.delete_aclgroup_configuration(&acl_group_name).await?;
            context.print_outcome(format!("acl group '{}' deleted", acl_group_name));
            Ok(())
          }
        } else {
          context.print_outcome(format!("cancelled, acl group '{}' not deleted", acl_group_name));
          Ok(())
        }
      }
      Err(error) => match error {
        DshApiError::NotFound { .. } => return err!("acl group '{}' does not exist", acl_group_name),
        _ => Err(DshCliError::from(error)),
      },
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AclGroupGrant {}

#[async_trait]
impl CommandExecutor for AclGroupGrant {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let acl_group_name = target.unwrap_or_else(|| unreachable!());
    let (readable_options, writable_options) = get_kafka_acl_group_topic_options(matches);
    if readable_options.is_empty() && writable_options.is_empty() {
      return err!("no readable or writable stream options are provided");
    }
    let mut acl_group = client.get_aclgroup_configuration(&acl_group_name).await?;
    context.print_outcome("current acl group authorizations");
    list_streams(&acl_group_name, &acl_group, context)?;

    let mut update = false;

    for readable_option in readable_options {
      match acl_group.readable_streams.iter_mut().find(|acl_group_topic| **acl_group_topic == readable_option) {
        Some(already_granted) => context.print_warning(format!("{} stream '{}' is already readable", already_granted.kind, already_granted.name)),
        None => {
          acl_group.readable_streams.push(readable_option);
          update = true;
        }
      }
    }
    for writable_option in writable_options {
      match acl_group.writable_streams.iter_mut().find(|acl_group_topic| **acl_group_topic == writable_option) {
        Some(already_granted) => context.print_warning(format!("{} stream '{}' is already writable", already_granted.kind, already_granted.name)),
        None => {
          acl_group.writable_streams.push(writable_option);
          update = true;
        }
      }
    }
    if update {
      context.print_outcome("new acl group authorizations");
      list_streams(&acl_group_name, &acl_group, context)?;
      if context.confirmed(format!("update acl group '{}'?", acl_group_name))? {
        if context.dry_run() {
          context.print_warning("dry-run mode, acl group not updated");
        } else {
          client.put_aclgroup_configuration(&acl_group_name, &acl_group).await?;
          context.print_outcome(format!("acl group '{}' updated", acl_group_name));
        }
      } else {
        context.print_outcome(format!("cancelled, acl group '{}' not updated", acl_group_name));
      }
    } else {
      context.print_outcome(format!("no changes with provided options, acl group '{}' not updated", acl_group_name));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static ACL_GROUP_LABELS_LIST: [AclGroupLabel; 5] = [AclGroupLabel::AclGroupName, AclGroupLabel::StreamName, AclGroupLabel::Kind, AclGroupLabel::Readable, AclGroupLabel::Writable];

struct AclGroupList {}

#[async_trait]
impl CommandExecutor for AclGroupList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all proxy acl groups");
    let start_instant = context.now();
    let acl_group_ids = client.get_aclgroup_ids().await?;
    let acl_groups = try_join_all(acl_group_ids.iter().map(|acl_group_name| client.get_aclgroup_configuration(acl_group_name))).await?;
    let streams = &acl_groups
      .iter()
      .map(|acl_group| {
        get_streams(&acl_group.readable_streams, &acl_group.writable_streams)
          .into_iter()
          .map(Some)
          .collect_vec()
      })
      .collect_vec();

    context.print_execution_time(start_instant);

    let mut formatter = ListFormatter::new(&ACL_GROUP_LABELS_LIST, context);

    for (acl_group_name, streams) in acl_group_ids.iter().zip(streams) {
      if streams.is_empty() {
        formatter.push_target_id_value(acl_group_name.to_string(), &None);
      } else {
        streams.iter().for_each(|stream| {
          formatter.push_target_id_value(acl_group_name.to_string(), stream);
        });
      }
    }
    formatter.print(None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AclGroupListIds {}

#[async_trait]
impl CommandExecutor for AclGroupListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all proxy acl group ids");
    let start_instant = context.now();
    let proxy_ids = client.get_aclgroup_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("proxy id", context);
    formatter.push_target_ids(&proxy_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AclGroupRevoke {}

#[async_trait]
impl CommandExecutor for AclGroupRevoke {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let acl_group_name = target.unwrap_or_else(|| unreachable!());
    let (readable_options, writable_options) = get_kafka_acl_group_topic_options(matches);
    if readable_options.is_empty() && writable_options.is_empty() {
      return err!("no readable or writable stream options are provided");
    }
    let mut acl_group = client.get_aclgroup_configuration(&acl_group_name).await?;
    list_streams(&acl_group_name, &acl_group, context)?;

    let mut update = false;

    for readable_option in readable_options {
      if acl_group.readable_streams.contains(&readable_option) {
        context.print(format!("revoke read access from {} stream '{}'", readable_option.kind, readable_option.name));
        acl_group.readable_streams.retain(|acl_group_topic| *acl_group_topic != readable_option);
        update = true;
      } else {
        context.print_warning(format!("read access is not granted to {} stream '{}'", readable_option.kind, readable_option.name));
      }
    }

    for writable_option in writable_options {
      if acl_group.writable_streams.contains(&writable_option) {
        context.print_outcome(format!("revoke write access from {} stream '{}'", writable_option.kind, writable_option.name));
        acl_group.writable_streams.retain(|acl_group_topic| *acl_group_topic != writable_option);
        update = true;
      } else {
        context.print_warning(format!("write access is not granted to {} stream '{}'", writable_option.kind, writable_option.name));
      }
    }

    if update {
      if context.confirmed(format!("update acl group '{}'?", acl_group_name))? {
        if context.dry_run() {
          context.print_warning("dry-run mode, acl group not updated");
        } else {
          client.put_aclgroup_configuration(&acl_group_name, &acl_group).await?;
          context.print_outcome(format!("acl group '{}' updated", acl_group_name));
          if acl_group.readable_streams.is_empty() && acl_group.writable_streams.is_empty() {
            context.print_warning(format!("acl group '{}' is not authorized for any readable or writable streams", acl_group_name));
            if context.confirmed(format!("delete acl group '{}'?", acl_group_name))? {
              client.delete_aclgroup_configuration(&acl_group_name).await?;
              context.print_outcome(format!("acl group '{}' deleted", acl_group_name));
            }
          }
        }
      } else {
        context.print_outcome(format!("cancelled, acl group '{}' not updated", acl_group_name));
      }
    } else {
      context.print_outcome(format!("no changes with provided options, acl group '{}' not updated", acl_group_name));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct AclGroupShow {}

#[async_trait]
impl CommandExecutor for AclGroupShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let acl_group_name = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show configuration of proxy acl group '{}'", acl_group_name));
    let acl_group = client.get_aclgroup_configuration(&acl_group_name).await?;
    list_streams(&acl_group_name, &acl_group, context)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

fn list_streams(acl_group_name: &String, acl_group: &KafkaAclGroup, context: &Context) -> Result<(), DshCliError> {
  let streams = get_streams(&acl_group.readable_streams, &acl_group.writable_streams)
    .into_iter()
    .map(Some)
    .collect_vec();
  if streams.is_empty() {
    context.print_warning(format!(
      "acl group '{}' exists but is not authorized for any readable or writable streams",
      acl_group_name
    ));
  } else {
    let mut formatter = ListFormatter::new(&ACL_GROUP_LABELS, context);
    formatter.push_values(&streams);
    formatter.print(None)?;
  }
  Ok(())
}

fn get_streams<'a>(readable_streams: &'a Vec<KafkaAclGroupTopic>, writable_streams: &'a Vec<KafkaAclGroupTopic>) -> Vec<(&'a str, &'a KafkaAclGroupTopicKind, bool, bool)> {
  let mut map = HashMap::<(&str, &KafkaAclGroupTopicKind), (bool, bool)>::new();
  for readable_stream in readable_streams {
    map.entry((&readable_stream.name, &readable_stream.kind)).or_default().0 = true;
  }
  for writable_stream in writable_streams {
    map.entry((&writable_stream.name, &writable_stream.kind)).or_default().1 = true;
  }
  let mut vec = map
    .iter()
    .map(|((name, kind), (readable, writable))| (*name, *kind, *readable, *writable))
    .collect_vec();
  vec.sort_by_key(|(name, _, _, _)| *name);
  vec
}

fn get_kafka_acl_group_topic_options(matches: &ArgMatches) -> (Vec<KafkaAclGroupTopic>, Vec<KafkaAclGroupTopic>) {
  let mut readable_streams: Vec<KafkaAclGroupTopic> = vec![];
  if let Some(topics) = matches.get_many::<String>(READ_INTERNAL_OPTION) {
    for topic in topics {
      readable_streams.push(KafkaAclGroupTopic::new(KafkaAclGroupTopicKind::Internal, topic))
    }
  }
  if let Some(topics) = matches.get_many::<String>(READ_PUBLIC_OPTION) {
    for topic in topics {
      readable_streams.push(KafkaAclGroupTopic::new(KafkaAclGroupTopicKind::Public, topic))
    }
  }
  if let Some(topics) = matches.get_many::<String>(READ_TOPIC_OPTION) {
    for topic in topics {
      readable_streams.push(KafkaAclGroupTopic::new(KafkaAclGroupTopicKind::Topic, topic))
    }
  }
  let mut writable_streams: Vec<KafkaAclGroupTopic> = vec![];
  if let Some(topics) = matches.get_many::<String>(WRITE_INTERNAL_OPTION) {
    for topic in topics {
      writable_streams.push(KafkaAclGroupTopic::new(KafkaAclGroupTopicKind::Internal, topic))
    }
  }
  if let Some(topics) = matches.get_many::<String>(WRITE_PUBLIC_OPTION) {
    for topic in topics {
      writable_streams.push(KafkaAclGroupTopic::new(KafkaAclGroupTopicKind::Public, topic))
    }
  }
  if let Some(topics) = matches.get_many::<String>(WRITE_TOPIC_OPTION) {
    for topic in topics {
      writable_streams.push(KafkaAclGroupTopic::new(KafkaAclGroupTopicKind::Topic, topic))
    }
  }
  (readable_streams, writable_streams)
}
