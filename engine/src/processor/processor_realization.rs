//! # Defines the behavior of a Trifonius `ProcessorRealization`

#![allow(clippy::module_inception)]

use crate::processor::processor::Processor;
use crate::processor::processor_descriptor::ProcessorDescriptor;
use crate::processor::{ProcessorId, ProcessorIdentifier, ProcessorType};
use crate::target_client::TargetClientFactory;

/// Defines the behavior of a Trifonius `ProcessorRealization`
pub trait ProcessorRealization<'a> {
  /// # Get this `ProcessorRealization`s descriptor
  ///
  /// ## Returns
  /// * This `ProcessorRealization`s descriptor.
  fn descriptor(&self) -> ProcessorDescriptor;

  /// # Get this `ProcessorRealization`s id (name)
  ///
  /// ## Returns
  /// * This `ProcessorRealization`s id.
  fn id(&self) -> &ProcessorId;

  /// # Get this `ProcessorRealization`s `ProcessorIdentifier`
  ///
  /// ## Returns
  /// * This `ProcessorRealization`s identifier.
  fn identifier(&self) -> &ProcessorIdentifier;

  /// # Get this `ProcessorRealization`s label
  ///
  /// A `Processor`s label should be used to present it to a user.
  ///
  /// ## Returns
  /// * This `ProcessorRealization`s label.
  fn label(&self) -> &str;

  /// # Create a `Processor` from this `ProcessorRealization`
  ///
  /// ## Returns
  /// * The created `Processor`.
  fn processor(&'a self, target_client_factory: Option<&'a TargetClientFactory>) -> Result<Box<dyn Processor + 'a>, String>;

  /// # Get this `ProcessorRealization`s type
  ///
  /// ## Returns
  /// * This `ProcessorRealization`s type.
  fn processor_type(&self) -> ProcessorType;
}
