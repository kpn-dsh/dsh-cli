#![allow(clippy::module_inception)]

use std::sync::Arc;

use crate::engine_target::{from_tenant_to_template_mapping, EngineTarget};
use crate::pipeline::PipelineId;
use crate::processor::dshservice::dshservice_config::read_dshservice_config;
use crate::processor::dshservice::dshservice_instance::DshServiceInstance;
use crate::processor::processor_config::ProcessorConfig;
use crate::processor::processor_context::ProcessorContext;
use crate::processor::processor_descriptor::{ProcessorDescriptor, ProfileDescriptor};
use crate::processor::processor_instance::ProcessorInstance;
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::{ProcessorId, ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};

pub struct DshServiceRealization {
  pub(crate) processor_identifier: ProcessorIdentifier,
  pub(crate) processor_config: ProcessorConfig,
}

impl DshServiceRealization {
  pub fn create(config_file_name: &str) -> Result<Self, String> {
    let processor_config = read_dshservice_config(config_file_name)?;
    Ok(DshServiceRealization {
      processor_identifier: ProcessorIdentifier {
        processor_technology: ProcessorTechnology::DshService,
        processor_realization_id: ProcessorRealizationId::try_from(processor_config.processor.processor_realization_id.as_str())?,
      },
      processor_config,
    })
  }
}

impl ProcessorRealization for DshServiceRealization {
  fn descriptor(&self, engine_target: &EngineTarget) -> ProcessorDescriptor {
    let profiles = self
      .processor_config
      .dshservice_specific_config
      .as_ref()
      .unwrap()
      .profiles
      .iter()
      .map(|p| p.convert_to_descriptor())
      .collect::<Vec<ProfileDescriptor>>();
    self
      .processor_config
      .convert_to_descriptor(profiles, &from_tenant_to_template_mapping(engine_target.tenant()))
  }

  fn processor_realization_id(&self) -> &ProcessorRealizationId {
    &self.processor_identifier.processor_realization_id
  }

  fn identifier(&self) -> &ProcessorIdentifier {
    &self.processor_identifier
  }

  fn label(&self) -> &str {
    &self.processor_config.processor.label
  }

  fn processor_instance<'a>(
    &'a self,
    pipeline_id: Option<PipelineId>,
    processor_id: ProcessorId,
    processor_context: Arc<ProcessorContext>,
  ) -> Result<Box<dyn ProcessorInstance + 'a>, String> {
    match DshServiceInstance::create(pipeline_id, processor_id, &self.processor_config, processor_context.clone()) {
      Ok(processor) => Ok(Box::new(processor)),
      Err(error) => Err(error),
    }
  }

  fn processor_technology(&self) -> ProcessorTechnology {
    ProcessorTechnology::DshService
  }
}
