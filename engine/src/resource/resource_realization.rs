#![allow(clippy::module_inception)]

use crate::pipeline::PipelineName;
use crate::resource::resource_descriptor::ResourceDescriptor;
use crate::resource::resource_instance::ResourceInstance;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceName, ResourceType};
use crate::target_client::TargetClientFactory;

pub trait ResourceRealization<'a> {
  /// # Get this `ResourceRealization`s descriptor
  ///
  /// ## Returns
  /// * This `ResourceRealization`s descriptor.
  fn descriptor(&self) -> ResourceDescriptor;

  /// # Get this `ResourceRealization`s id (name)
  ///
  /// ## Returns
  /// * This `ResourceRealization`s id.
  fn id(&self) -> &ResourceId;

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
  /// * `pipeline_name`         - Pipeline name wrapped in a `Some` when the created
  ///                             `ResourceInstance` is part of a _Pipeline_,
  ///                             `None` when it is not.
  /// * `resource_name`         - Resource name.
  /// * `target_client_factory` - Target client factory.
  ///
  /// ## Returns
  /// * The created `ResourceInstance`.
  fn resource_instance(
    &'a self,
    pipeline_name: Option<&'a PipelineName>,
    resource_name: &'a ResourceName,
    target_client_factory: &'a TargetClientFactory,
  ) -> Result<Box<dyn ResourceInstance + 'a>, String>;

  /// # Get this `ResourceRealization`s type
  ///
  /// ## Returns
  /// * This `ResourceRealization`s type.
  fn resource_type(&self) -> ResourceType;
}
