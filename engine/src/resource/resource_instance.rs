#![allow(clippy::module_inception)]

use std::fmt::{Display, Formatter};

use async_trait::async_trait;

use crate::pipeline::PipelineId;
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::ResourceId;

#[async_trait]
pub trait ResourceInstance: Send + Sync {
  /// # Returns the pipeline id of this `ResourceInstance`
  ///
  /// ## Returns
  /// * The optional `PipelineId` of this `ResourceInstance`.
  fn pipeline_id(&self) -> Option<&PipelineId>;

  /// # Returns the resource id of this `ResourceInstance`
  ///
  /// ## Returns
  /// * The `ResourceId` of this `ResourceInstance`.
  fn resource_id(&self) -> &ResourceId;

  /// # Returns the `ResourceRealizaton` from this `ResourceInstance`
  ///
  /// ## Returns
  /// * The `ResourceRealization`.
  fn resource_realization(&self) -> &dyn ResourceRealization;

  /// # Get this `ResourceInstance`s status
  ///
  /// ## Parameters
  /// * `resource_name` - Resource name of the deployed resource instance.
  ///
  /// ## Returns
  /// * `Ok<ResourceStatus>` - signals whether the resource instance with the given
  ///                          `resource_name` is active or not.
  /// * `Err(msg)`           - when the status request could not be sent.
  async fn status(&self) -> Result<ResourceStatus, String>;
}

#[derive(Debug)]
pub struct ResourceStatus {
  pub up: bool,
}

impl Display for ResourceStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.up {
      write!(f, "up")
    } else {
      write!(f, "down")
    }
  }
}
