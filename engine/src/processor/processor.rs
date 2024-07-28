//! # Defines the behavior of a Trifonius `Processor`
//!

#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::pipeline::PipelineId;
use async_trait::async_trait;

use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::{JunctionId, ParameterId, ProcessorId, ProcessorIdentifier, ProcessorType, ProfileId, ServiceId};
use crate::resource::ResourceIdentifier;

/// Defines the behavior of a Trifonius `Processor`
#[async_trait]
pub trait Processor {
  /// # Deploy this `Processor`
  ///
  /// ## Parameters
  /// * `service_id`         - Service id (name) of the deployed processor.
  /// * `inbound_junctions`  - Map containing the inbound resources.
  /// * `outbound_junctions` - Map containing the outbound resources.
  /// * `parameters`         - Map containing the deployment parameters.
  /// * `profile_id`         - Profile id.
  ///
  /// ## Returns
  /// * `Ok<()>`   - when the deployment request was successfully sent.
  /// * `Err(msg)` - when the deployment request could not be sent.
  async fn deploy(
    &self,
    pipeline_id: &PipelineId,
    service_id: &ServiceId,
    inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    parameters: &HashMap<ParameterId, String>,
    profile_id: Option<&ProfileId>,
  ) -> Result<(), String>;

  /// # Get the resources compatible with this `Processor`
  ///
  /// ## Parameters
  /// * `junction_id` - identifies the junction for which the compatible resources need to be
  ///                   retrieved.
  ///
  /// ## Returns
  /// * `Ok<Vec<ResourceIdentifier>` - list of identifiers of compatible resources.
  /// * `Err(msg)`                   - when the list could not be composed.
  async fn compatible_resources(&self, junction_id: &JunctionId) -> Result<Vec<ResourceIdentifier>, String>;

  /// # Get this `Processor`s descriptor
  ///
  /// ## Returns
  /// * This `Processor`s descriptor.
  fn descriptor(&self) -> ProcessorDescriptor;

  /// # Get this `Processor`s id (name)
  ///
  /// ## Returns
  /// * This `Processor`s id.
  fn id(&self) -> &ProcessorId;

  /// # Get this `Processor`s `ProcessorIdentifier`
  ///
  /// ## Returns
  /// * This `Processor`s identifier.
  fn identifier(&self) -> &ProcessorIdentifier;

  /// # Get this `Processor`s label
  ///
  /// A `Processor`s label should be used to present it to a user.
  ///
  /// ## Returns
  /// * This `Processor`s label.
  fn label(&self) -> &str;

  /// # Get this `Processor`s type
  ///
  /// ## Returns
  /// * This `Processor`s type.
  fn processor_type(&self) -> ProcessorType;

  /// # Start this `Processor`
  ///
  /// ## Parameters
  /// * `service_id` - Service id (name) of the processor that should be undeployed.
  ///
  /// ## Returns
  /// * `Ok<true>`  - when the start request was successfully sent.
  /// * `Ok<false>` - when no processor with `service_id` exists.
  /// * `Err(msg)`  - when the start request could not be sent.
  async fn start(&self, pipeline_id: &PipelineId, service_id: &ServiceId) -> Result<bool, String>;

  /// # Get this `Processor`s status
  ///
  /// ## Parameters
  /// * `service_id` - Service id (name) of the processor that should be stopped.
  ///
  /// ## Returns
  /// * `Ok<ProcessorStatus>` - signals whether the processor with the given `service_id` is active
  ///                           or not.
  /// * `Err(msg)`            - when the status request could not be sent.
  async fn status(&self, pipeline_id: &PipelineId, service_id: &ServiceId) -> Result<ProcessorStatus, String>;

  /// # Stop this `Processor`
  ///
  /// ## Parameters
  /// * `service_id` - Service id (name) of the processor that should be stopped.
  ///
  /// ## Returns
  /// * `Ok<true>`  - when the stop request was successfully sent.
  /// * `Ok<false>` - when no processor with `service_id` exists.
  /// * `Err(msg)`  - when the stop request could not be sent.
  async fn stop(&self, pipeline_id: &PipelineId, service_id: &ServiceId) -> Result<bool, String>;

  /// # Undeploy this `Processor`
  ///
  /// ## Parameters
  /// * `service_id` - Service id (name) of the processor that should be undeployed.
  ///
  /// ## Returns
  /// * `Ok<true>`  - when the undeployment request was successfully sent.
  /// * `Ok<false>` - when no processor with `service_id` exists.
  /// * `Err(msg)`  - when the undeployment request could not be sent.
  async fn undeploy(&self, pipeline_id: &PipelineId, service_id: &ServiceId) -> Result<bool, String>;
}

#[derive(Debug)]
pub struct ProcessorStatus {
  pub up: bool,
}

impl Display for ProcessorStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.up {
      write!(f, "up")
    } else {
      write!(f, "down")
    }
  }
}

pub fn service_name(pipeline_id: &PipelineId, processor_id: &ProcessorId, service_id: &ServiceId) -> String {
  format!("{}-{}-{}", pipeline_id, processor_id, service_id)
}
