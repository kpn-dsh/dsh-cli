use std::ops::Deref;

use async_trait::async_trait;
use dsh_sdk::dsh::datastream::Stream;

use crate::resource::dsh_topic::topic_registry::resource_identifier;
use crate::resource::resource::{Resource, ResourceIdentifier, ResourceStatus};
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::ResourceType;
use crate::TargetClientFactory;

pub struct TopicResourceImpl<'a> {
  pub resource_identifier: ResourceIdentifier,
  pub resource_descriptor: ResourceDescriptor,
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> TopicResourceImpl<'a> {
  pub fn create(stream: &Stream, target_client_factory: &'a TargetClientFactory) -> Result<Self, String> {
    let resource_descriptor = ResourceDescriptor {
      resource_type: ResourceType::DshTopic,
      name: stream.name().to_string(),
      description: "".to_string(),
      version: None,
      writable: !stream.write().is_empty(),
      readable: !stream.read().is_empty(),
      metadata: vec![
        ("partitions".to_string(), stream.partitions().to_string()),
        ("replication".to_string(), stream.replication().to_string()),
        ("partitioner".to_string(), stream.partitioner().to_string()),
        ("partitioning-depth".to_string(), stream.partitioning_depth().to_string()),
        ("can-retain".to_string(), stream.can_retain().to_string()),
        ("cluster".to_string(), stream.cluster().to_string()),
      ],
      more_info_url: Some(format!(
        "https://console.dsh-dev.dsh.np.aws.kpn.com/#/profiles/{}/resources/streams",
        target_client_factory.tenant
      )),
      metrics_url: None,
      viewer_url: Some(format!(
        "https://eavesdropper.{}.dsh-dev.dsh.np.aws.kpn.com?topics={}",
        target_client_factory.tenant,
        stream.name()
      )),
    };
    Ok(TopicResourceImpl { resource_identifier: resource_identifier(stream.name().to_string()), resource_descriptor, target_client_factory })
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

  fn name(&self) -> &str {
    &self.resource_identifier.name
  }

  fn resource_type(&self) -> ResourceType {
    ResourceType::DshTopic
  }

  async fn status(&self) -> Result<ResourceStatus, String> {
    match get_topic_status(self.target_client_factory, &self.resource_identifier.name).await? {
      Some(status) => Ok(status),
      None => Err(format!("could not get status for non-existent topic {}", &self.resource_identifier.name)),
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
