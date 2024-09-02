#![allow(clippy::module_inception)]

use crate::pipeline::PipelineId;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_instance::ResourceInstance;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceRealizationId, ResourceType};

pub trait ResourceRealization {
  /// # Get this `ResourceRealization`s descriptor
  ///
  /// ## Returns
  /// * This `ResourceRealization`s descriptor.
  fn descriptor(&self) -> ResourceDescriptor;

  /// # Get this `ResourceRealization`s id (name)
  ///
  /// ## Returns
  /// * This `ResourceRealization`s id.
  fn id(&self) -> &ResourceRealizationId;

  /// # Get this `ResourceRealization`s `ResourceIdentifier`
  ///
  /// ## Returns
  /// * This `ResourceRealization`s identifier.
  fn identifier(&self) -> &ResourceIdentifier;

  /// # Get this `ResourceRealization`s label
  ///
  /// A `ResourceRealization`s label should be used to present it to a user.
  ///
  /// ## Returns
  /// * This `ResourceRealization`s label.
  fn label(&self) -> &str;

  /// # Create a `ResourceInstance` from this `ResourceRealization`
  ///
  /// ## Parameters
  /// * `pipeline_id` - Pipeline id wrapped in a `Some` when the created
  ///                   `ResourceInstance` is part of a _Pipeline_,
  ///                   `None` when it is not.
  /// * `resource_id` - Resource name.
  ///
  /// ## Returns
  /// * The created `ResourceInstance`.
  fn resource_instance<'a>(&'a self, pipeline_id: Option<PipelineId>, resource_id: ResourceId) -> Result<Box<dyn ResourceInstance + 'a>, String>;

  /// # Get this `ResourceRealization`s type
  ///
  /// ## Returns
  /// * This `ResourceRealization`s type.
  fn resource_type(&self) -> ResourceType;
}
