use std::ops::Deref;

use async_trait::async_trait;
use dsh_sdk::dsh::datastream::Stream;

use crate::processor::application::TargetClientFactory;
use crate::resource::dsh_topic::topic_descriptor::DshTopicDescriptor;
use crate::resource::resource::{Resource, ResourceIdentifier, ResourceStatus};
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::ResourceType;

pub struct TopicResourceImpl<'a> {
  pub resource_identifier: ResourceIdentifier,
  pub resource_descriptor: ResourceDescriptor,
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> TopicResourceImpl<'a> {
  pub fn create(stream: &Stream, target_client_factory: &'a TargetClientFactory) -> Result<Self, String> {
    let resource_descriptor = ResourceDescriptor {
      resource_type: ResourceType::DshTopic,
      id: stream.name().to_string(),
      label: stream.name().to_string(),
      description: "DSH Kafka topic".to_string(),
      version: None,
      writable: stream.write_access(),
      readable: stream.read_access(),
      metadata: Vec::default(),
      more_info_url: target_client_factory
        .dsh_platform
        .console_url()
        .map(|url| format!("{}/#/profiles/{}/resources/streams", url, target_client_factory.tenant)),
      metrics_url: None,
      viewer_url: target_client_factory
        .dsh_platform
        .app_domain(target_client_factory.tenant.as_str())
        .map(|domain| format!("https://eavesdropper.{}.{}?topics={}", target_client_factory.tenant, domain, stream.name())),
      dsh_topic_descriptor: Some(DshTopicDescriptor {
        // TODO Check proper topic name
        topic: match stream.write_pattern() {
          Ok(write_pattern) => write_pattern.to_string(),
          Err(_) => stream.name().to_string(),
        },
        partitions: u32::try_from(stream.partitions()).unwrap(),
        replication: u32::try_from(stream.replication()).unwrap(),
        // TODO Is dsh_envelope ok like this?
        dsh_envelope: stream.name().starts_with("stream."),
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
    Ok(TopicResourceImpl { resource_identifier: topic_resource_identifier(stream.name().to_string()), resource_descriptor, target_client_factory })
  }
}

#[async_trait]
impl Resource for TopicResourceImpl<'_> {
  fn descriptor(&self) -> &ResourceDescriptor {
    &self.resource_descriptor
  }

  fn identifier(&self) -> &ResourceIdentifier {
    &self.resource_identifier
  }

  fn id(&self) -> &str {
    &self.resource_identifier.id
  }

  fn label(&self) -> &str {
    &self.resource_descriptor.label
  }

  fn resource_type(&self) -> ResourceType {
    ResourceType::DshTopic
  }

  async fn status(&self) -> Result<ResourceStatus, String> {
    match get_topic_status(self.target_client_factory, &self.resource_identifier.id).await? {
      Some(status) => Ok(status),
      None => Err(format!("could not get status for non-existent topic {}", &self.resource_identifier.id)),
    }
  }
}

async fn get_topic_status(target_client_factory: &TargetClientFactory, topic_name: &str) -> Result<Option<ResourceStatus>, String> {
  let target_client = target_client_factory.get().await?;
  match target_client
    .client
    .topic_get_by_tenant_topic_by_id_status(target_client.tenant, topic_name, target_client.token.as_str())
    .await
  {
    Ok(response) => {
      if response.status() == 404 {
        Ok(None)
      } else {
        let api_allocation_status = response.deref();
        Ok(Some(ResourceStatus { up: api_allocation_status.provisioned }))
      }
    }
    Err(e) => Err(format!("dsh api error ({})", e)),
  }
}

pub fn topic_resource_identifier(id: String) -> ResourceIdentifier {
  ResourceIdentifier { resource_type: ResourceType::DshTopic, id }
}
