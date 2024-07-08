use std::ops::Deref;
use std::str::FromStr;

use num_traits::ToPrimitive;

use crate::resource::resource::{Resource, ResourceIdentifier};
use crate::resource::resource_descriptor::{ResourceDescriptor, ResourceType};
use crate::resource::topic::topic_descriptor::{CleanupPolicy, MessageTimestampType, TopicDescriptor, TopicStatus};
use crate::resource::topic::topic_resource::TopicResourceImpl;
use crate::TargetClientFactory;

pub struct TopicRegistry<'a> {
  target_client_factory: &'a TargetClientFactory,
}

impl<'a> TopicRegistry<'a> {
  pub fn create(target_client_factory: &'a TargetClientFactory) -> Result<TopicRegistry, String> {
    Ok(TopicRegistry { target_client_factory })
  }

  pub async fn resource_by_name(&self, resource_name: &str) -> Result<Box<dyn Resource + 'a>, String> {
    match get_topic_descriptor(self.target_client_factory, resource_name).await? {
      Some(topic_descriptor) => match TopicResourceImpl::create(topic_descriptor, self.target_client_factory) {
        Ok(topic_resource) => Ok(topic_resource),
        Err(e) => Err(format!("failed to create topic resource for topic {} ({})", resource_name, e)),
      },
      None => Err(format!("could not create topic resource for non-existent topic {}", resource_name)),
    }
  }

  pub async fn resource_identifiers(&self) -> Result<Vec<ResourceIdentifier>, String> {
    Ok(
      get_topic_descriptors(self.target_client_factory)
        .await?
        .iter()
        .map(|td| ResourceIdentifier { resource_type: ResourceType::Topic, name: td.topic_name.clone() })
        .collect(),
    )
  }

  pub async fn resource_descriptor_by_name(&self, resource_name: &str) -> Result<Option<ResourceDescriptor>, String> {
    Ok(get_topic_descriptor(self.target_client_factory, resource_name).await?.map(ResourceDescriptor::from))
  }

  pub async fn resource_descriptors(&self) -> Result<Vec<ResourceDescriptor>, String> {
    Ok(
      get_topic_descriptors(self.target_client_factory)
        .await?
        .iter()
        .map(|td| ResourceDescriptor::from(td.clone()))
        .collect(),
    )
  }

  pub async fn resource_descriptors_with_status(&self) -> Result<Vec<(ResourceDescriptor, bool)>, String> {
    Ok(
      get_topic_descriptors_with_status(self.target_client_factory)
        .await?
        .into_iter()
        .map(|(td, s)| (ResourceDescriptor::from(td.clone()), s))
        .collect::<Vec<(ResourceDescriptor, bool)>>(),
    )
  }
}

async fn get_topic_names(target_client_factory: &TargetClientFactory) -> Result<Vec<String>, String> {
  let target_client = target_client_factory.get().await?;
  match target_client
    .client
    .topic_get_by_tenant_topic(target_client.tenant, target_client.token.as_str())
    .await
  {
    Ok(response) => {
      let mut topic_names = response.deref().0.clone();
      topic_names.sort();
      Ok(topic_names)
    }
    Err(e) => Err(format!("dsh api error ({})", e)),
  }
}

async fn get_topic_descriptor(target_client_factory: &TargetClientFactory, topic_name: &str) -> Result<Option<TopicDescriptor>, String> {
  let target_client = target_client_factory.get().await?;
  match target_client
    .client
    .topic_get_by_tenant_topic_by_id_configuration(target_client.tenant, topic_name, target_client.token.as_str())
    .await
  {
    Ok(response) => {
      if response.status() == 404 {
        Ok(None)
      } else {
        let api_topic = response.deref();
        let descriptor = TopicDescriptor {
          topic_name: topic_name.to_string(),
          message_timestamp_type: match api_topic.kafka_properties.get("message.timestamp.type") {
            Some(mtt) => Some(MessageTimestampType::try_from(mtt.as_str())?),
            None => None,
          },
          cleanup_policy: match api_topic.kafka_properties.get("cleanup.policy") {
            Some(cp) => Some(CleanupPolicy::try_from(cp.as_str())?),
            None => None,
          },
          max_message_bytes: match api_topic.kafka_properties.get("max.message.bytes") {
            Some(mmbs) => match u64::from_str(mmbs) {
              Ok(mmbs) => Some(mmbs),
              Err(_) => return Err("could not convert max.message.bytes value".to_string()),
            },
            None => None,
          },
          segment_bytes: match api_topic.kafka_properties.get("segment.bytes") {
            Some(sbs) => match u64::from_str(sbs) {
              Ok(sbs) => Some(sbs),
              Err(_) => return Err("could not convert segment.bytes value".to_string()),
            },
            None => None,
          },
          number_of_partitions: api_topic.partitions.to_u64().ok_or("could not convert number of partitions value".to_string())?,
          replication_factor: api_topic
            .replication_factor
            .to_u64()
            .ok_or("could not convert replication factor value".to_string())?,
        };
        Ok(Some(descriptor))
      }
    }
    Err(e) => Err(format!("dsh api error ({})", e)),
  }
}

pub(crate) async fn get_topic_status(target_client_factory: &TargetClientFactory, topic_name: &str) -> Result<Option<TopicStatus>, String> {
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
        Ok(Some(TopicStatus {
          provisioned: api_allocation_status.provisioned,
          derived_from: None,
          notifications: None,
        }))
      }
    }
    Err(e) => Err(format!("dsh api error ({})", e)),
  }
}

async fn get_topic_descriptors(target_client_factory: &TargetClientFactory) -> Result<Vec<TopicDescriptor>, String> {
  let mut topic_descriptors = Vec::new();
  let topic_names = get_topic_names(target_client_factory).await?;
  for topic_name in topic_names {
    let topic_descriptor = get_topic_descriptor(target_client_factory, topic_name.as_str()).await?.unwrap();
    topic_descriptors.push(topic_descriptor);
  }
  Ok(topic_descriptors)
}

async fn get_topic_descriptors_with_status(target_client_factory: &TargetClientFactory) -> Result<Vec<(TopicDescriptor, bool)>, String> {
  let mut topic_descriptor_status_pairs = Vec::new();
  let topic_names = get_topic_names(target_client_factory).await?;
  for topic_name in topic_names {
    match (
      get_topic_descriptor(target_client_factory, topic_name.as_str()).await?,
      get_topic_status(target_client_factory, topic_name.as_str()).await?,
    ) {
      (Some(topic_descriptor), Some(topic_status)) => topic_descriptor_status_pairs.push((topic_descriptor, topic_status.provisioned)),
      _ => return Err("could not retrieve topic information".to_string()),
    }
  }
  Ok(topic_descriptor_status_pairs)
}
