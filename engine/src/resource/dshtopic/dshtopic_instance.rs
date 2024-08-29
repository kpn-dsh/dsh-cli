use async_trait::async_trait;

use crate::engine_target::EngineTarget;
use crate::pipeline::PipelineId;
use crate::resource::dshtopic::dshtopic_realization::DshTopicRealization;
use crate::resource::resource_instance::{ResourceInstance, ResourceStatus};
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::ResourceId;

pub(crate) struct DshTopicInstance<'a> {
  pipeline_id: Option<&'a PipelineId>,
  resource_id: &'a ResourceId,
  resource_realization: &'a DshTopicRealization,
  engine_target: &'a EngineTarget<'a>,
}

impl<'a> DshTopicInstance<'a> {
  pub(crate) fn create(
    pipeline_id: Option<&'a PipelineId>,
    resource_id: &'a ResourceId,
    resource_realization: &'a DshTopicRealization,
    engine_target: &'a EngineTarget,
  ) -> Result<Self, String> {
    Ok(Self { pipeline_id, resource_id, resource_realization, engine_target })
  }
}

#[async_trait]
impl ResourceInstance for DshTopicInstance<'_> {
  fn pipeline_id(&self) -> Option<&PipelineId> {
    self.pipeline_id
  }

  fn resource_id(&self) -> &ResourceId {
    self.resource_id
  }

  fn resource_realization(&self) -> &dyn ResourceRealization {
    self.resource_realization
  }

  async fn status(&self) -> Result<ResourceStatus, String> {
    match get_topic_status(
      self.engine_target,
      &self.resource_realization.descriptor().dshtopic_descriptor.as_ref().unwrap().topic,
    )
    .await?
    {
      Some(status) => Ok(status),
      None => Err(format!(
        "could not get status for non-existent topic '{}'",
        &self.resource_realization.resource_identifier.id
      )),
    }
  }
}

async fn get_topic_status(_engine_target: &EngineTarget<'_>, _topic_name: &str) -> Result<Option<ResourceStatus>, String> {
  // let target_client = engine_target.dsh_api_client_factory.client().await?;
  // match target_client
  //   .topic_get_by_tenant_topic_by_id_status(target_client.tenant(), topic_name, target_client.token())
  //   .await
  // {
  //   Ok(response) => {
  //     if response.status() == 404 {
  //       Ok(None)
  //     } else {
  //       let api_allocation_status = response.deref();
  //       Ok(Some(ResourceStatus { up: api_allocation_status.provisioned }))
  //     }
  //   }
  //   Err(e) => Err(format!("dsh api error ({})", e)),
  // }
  todo!()
}
