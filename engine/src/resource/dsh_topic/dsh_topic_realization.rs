use dsh_sdk::dsh::datastream::Stream;

use crate::engine_target::EngineTarget;
use crate::pipeline::PipelineName;
use crate::resource::dsh_topic::dsh_topic_descriptor::DshTopicDescriptor;
use crate::resource::dsh_topic::dsh_topic_instance::DshTopicInstance;
use crate::resource::dsh_topic::DshTopicType;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_instance::ResourceInstance;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceName, ResourceType};

pub(crate) struct DshTopicRealization {
  pub(crate) resource_identifier: ResourceIdentifier,
  resource_descriptor: ResourceDescriptor,
}

impl DshTopicRealization {
  pub(crate) fn create(stream: &Stream, engine_target: &EngineTarget) -> Result<Self, String> {
    // TODO Check proper topic name
    let topic_name = match stream.write_pattern() {
      Ok(write_pattern) => write_pattern.to_string(),
      Err(_) => stream.name().to_string(),
    };
    let topic_type = DshTopicType::try_from_topic_name(topic_name.as_str())?;
    let gateway_topic_name = match topic_type {
      DshTopicType::Internal => None,
      DshTopicType::Scratch => None,
      DshTopicType::Stream => Some(format!("{}.dsh", stream.name())),
    };
    let resource_descriptor = ResourceDescriptor {
      resource_type: ResourceType::DshTopic,
      id: stream.name().replace('.', "-"),
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
      dsh_topic_descriptor: Some(DshTopicDescriptor {
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
    let resource_identifier = ResourceIdentifier { resource_type: ResourceType::DshTopic, id: ResourceId::try_from(resource_descriptor.id.as_str())? };
    Ok(DshTopicRealization { resource_identifier, resource_descriptor })
  }
}

impl<'a> ResourceRealization<'a> for DshTopicRealization {
  fn descriptor(&self) -> ResourceDescriptor {
    self.resource_descriptor.clone()
  }

  fn identifier(&self) -> &ResourceIdentifier {
    &self.resource_identifier
  }

  fn id(&self) -> &ResourceId {
    &self.resource_identifier.id
  }

  fn label(&self) -> &str {
    &self.resource_descriptor.label
  }

  fn resource_instance(
    &'a self,
    pipeline_name: Option<&'a PipelineName>,
    resource_name: &'a ResourceName,
    engine_target: &'a EngineTarget,
  ) -> Result<Box<dyn ResourceInstance + 'a>, String> {
    match DshTopicInstance::create(pipeline_name, resource_name, self, engine_target) {
      Ok(resource) => Ok(Box::new(resource)),
      Err(error) => Err(error),
    }
  }

  fn resource_type(&self) -> ResourceType {
    ResourceType::DshTopic
  }
}
