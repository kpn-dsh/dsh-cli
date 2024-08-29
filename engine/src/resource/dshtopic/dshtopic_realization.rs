use std::hash::{DefaultHasher, Hash, Hasher};

use dsh_sdk::dsh::datastream::Stream;
use lazy_static::lazy_static;
use regex::Regex;

use crate::engine_target::EngineTarget;
use crate::pipeline::PipelineId;
use crate::resource::dshtopic::dshtopic_descriptor::DshTopicDescriptor;
use crate::resource::dshtopic::dshtopic_instance::DshTopicInstance;
use crate::resource::dshtopic::DshTopicType;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_instance::ResourceInstance;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceRealizationId, ResourceType};

pub(crate) struct DshTopicRealization {
  pub(crate) resource_identifier: ResourceIdentifier,
  resource_descriptor: ResourceDescriptor,
}

impl DshTopicRealization {
  pub(crate) fn create(stream: &Stream, engine_target: &EngineTarget) -> Result<Self, String> {
    let resource_id = resource_id_from_stream_name(stream.name())?;
    let topic_name = topic_name_from_stream(stream);
    let topic_type = DshTopicType::try_from_topic_name(topic_name.as_str())?;
    let gateway_topic_name = match topic_type {
      DshTopicType::Internal => None,
      DshTopicType::Scratch => None,
      DshTopicType::Stream => Some(format!("{}.dsh", stream.name())),
    };
    let resource_descriptor = ResourceDescriptor {
      resource_type: ResourceType::DshTopic,
      id: resource_id.to_string(),
      label: stream.name().to_string(),
      description: "DSH Kafka topic".to_string(),
      version: None,
      icon: None,
      tags: vec![],
      writable: stream.write_access(),
      readable: stream.read_access(),
      metadata: Vec::default(),
      more_info_url: match topic_type {
        DshTopicType::Internal | DshTopicType::Stream => engine_target
          .platform()
          .console_url()
          .map(|url| format!("{}/#/profiles/{}/resources/streams", url, engine_target.tenant().name())),
        DshTopicType::Scratch => engine_target
          .platform()
          .console_url()
          .map(|url| format!("{}/#/profiles/{}/resources/topics", url, engine_target.tenant().name())),
      },
      metrics_url: None,
      viewer_url: engine_target
        .platform()
        .app_domain(engine_target.tenant().name())
        .map(|domain| format!("https://eavesdropper.{}?topics={}", domain, topic_name)),
      data_catalog_url: None,
      dshtopic_descriptor: Some(DshTopicDescriptor {
        name: stream.name().to_string(),
        topic: topic_name,
        gateway_topic: gateway_topic_name,
        topic_type: topic_type.clone(),
        partitions: u32::try_from(stream.partitions()).unwrap(),
        replication: u32::try_from(stream.replication()).unwrap(),
        // TODO Is dsh_envelope ok like this?
        dsh_envelope: topic_type == DshTopicType::Stream,
        read: stream.read().to_string(),
        write: stream.write().to_string(),
        read_pattern: stream.read_pattern().ok().map(|p| p.to_string()),
        write_pattern: stream.write_pattern().ok().map(|p| p.to_string()),
        partitioner: stream.partitioner().to_string(),
        partitioning_depth: u32::try_from(stream.partitioning_depth()).unwrap(),
        can_retain: stream.can_retain(),
        cluster: stream.cluster().to_string(),
      }),
    };
    let resource_identifier = ResourceIdentifier { resource_type: ResourceType::DshTopic, id: ResourceRealizationId::try_from(resource_descriptor.id.as_str())? };
    Ok(DshTopicRealization { resource_identifier, resource_descriptor })
  }
}

const MAX_RESOURCE_ID_LENGTH: usize = 50;
lazy_static! {
  static ref STREAM_NAME_REGEX: Regex = Regex::new("^[a-zA-Z0-9\\.\\-_]+$").unwrap();
}

fn resource_id_from_stream_name(stream_name: &str) -> Result<ResourceRealizationId, String> {
  if STREAM_NAME_REGEX.is_match(stream_name) {
    let parts: Vec<&str> = stream_name.split('.').collect();
    let id = if parts.len() == 2 {
      format!("{}-{}", parts.first().unwrap().replace('_', "-"), parts.get(1).unwrap().replace('_', "-"))
    } else if parts.len() == 3 {
      format!(
        "{}-{}-{}",
        parts.first().unwrap().replace('_', "-"),
        parts.get(2).unwrap().replace('_', "-"),
        parts.get(1).unwrap().replace('_', "-")
      )
    } else {
      return Err(format!("could not convert stream name {} to resource id", stream_name));
    };
    let id = if id.len() > MAX_RESOURCE_ID_LENGTH {
      let mut hasher = DefaultHasher::new();
      stream_name.hash(&mut hasher);
      let hash = hasher.finish();
      format!("{:.33}-{:016x}", id, hash)
    } else {
      id
    };
    ResourceRealizationId::try_from(id)
  } else {
    Err(format!("stream name {} contains invalid characters", stream_name))
  }
}

fn topic_name_from_stream(stream: &Stream) -> String {
  // TODO Check proper topic name
  match stream.write_pattern() {
    Ok(write_pattern) => write_pattern.to_string(),
    Err(_) => stream.name().to_string(),
  }
}

impl<'a> ResourceRealization<'a> for DshTopicRealization {
  fn descriptor(&self) -> ResourceDescriptor {
    self.resource_descriptor.clone()
  }

  fn identifier(&self) -> &ResourceIdentifier {
    &self.resource_identifier
  }

  fn id(&self) -> &ResourceRealizationId {
    &self.resource_identifier.id
  }

  fn label(&self) -> &str {
    &self.resource_descriptor.label
  }

  fn resource_instance(
    &'a self,
    pipeline_id: Option<&'a PipelineId>,
    resource_id: &'a ResourceId,
    engine_target: &'a EngineTarget,
  ) -> Result<Box<dyn ResourceInstance + 'a>, String> {
    match DshTopicInstance::create(pipeline_id, resource_id, self, engine_target) {
      Ok(resource) => Ok(Box::new(resource)),
      Err(error) => Err(error),
    }
  }

  fn resource_type(&self) -> ResourceType {
    ResourceType::DshTopic
  }
}

#[test]
fn test() {
  println!(
    "{}",
    resource_id_from_stream_name("internal.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz-100-2.tenant")
      .unwrap()
      .to_string()
  );
  assert_eq!(
    resource_id_from_stream_name("internal.abcdefghijklmnopqrstuvwxyzabcd-49.tenant")
      .unwrap()
      .to_string(),
    "internal-tenant-abcdefghijklmnopqrstuvwxyzabcd-49".to_string()
  );
  assert_eq!(
    resource_id_from_stream_name("internal.abcdefghijklmnopqrstuvwxyzabcde-50.tenant")
      .unwrap()
      .to_string(),
    "internal-tenant-abcdefghijklmnopqrstuvwxyzabcde-50".to_string()
  );
  assert!(resource_id_from_stream_name("internal.abcdefghijklmnopqrstuvwxyzabcdef-51.tenant")
    .unwrap()
    .to_string()
    .starts_with("internal-tenant-abcdefghijklmnopq-"));
  assert_ne!(
    resource_id_from_stream_name("internal.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz-100-1.tenant")
      .unwrap()
      .to_string(),
    resource_id_from_stream_name("internal.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz-100-2.tenant")
      .unwrap()
      .to_string()
  );
}
