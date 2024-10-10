#![allow(clippy::module_inception)]

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use dsh_api::dsh_api_tenant::DshApiTenant;

use crate::pipeline::pipeline_config::{read_pipeline_config, PipelineConfig, PipelineConnectionConfig, ProcessorJunctionConfig};
use crate::pipeline::PipelineId;
use crate::processor::processor_descriptor::{JunctionDescriptor, ProcessorDescriptor};
use crate::processor::processor_realization::ProcessorRealization;
use crate::processor::processor_registry::ProcessorRegistry;
use crate::processor::{JunctionId, JunctionIdentifier, ParameterId, ProcessorId, ProcessorIdentifier, ProcessorRealizationId, ProcessorTechnology};
use crate::resource::resource_realization::ResourceRealization;
use crate::resource::resource_registry::ResourceRegistry;
use crate::resource::{ResourceId, ResourceIdentifier, ResourceRealizationId, ResourceTechnology};
use crate::ProfileId;

pub struct Pipeline<'a> {
  pub id: PipelineId,
  pub name: String,
  pub resources: HashMap<ResourceId, PipelineResource<'a>>,
  pub processors: HashMap<ProcessorId, PipelineProcessor<'a>>,
  pub connections: Vec<PipelineConnection>,
  pub dependencies: Vec<PipelineDependency>,
}

impl<'a> Pipeline<'a> {
  pub fn create(
    config_file_name: &str,
    dsh_api_tenant: &'a DshApiTenant,
    resource_registry: &'a ResourceRegistry,
    processor_registry: &'a ProcessorRegistry,
  ) -> Result<Pipeline<'a>, String> {
    let pipeline_config = read_pipeline_config(config_file_name)?;

    let mut resources = HashMap::<ResourceId, PipelineResource>::new();
    for resource_config in &pipeline_config.resources {
      let name = resource_config.name.clone();
      let realization = resource_registry
        .resource_realization(&resource_config.resource_realization_id)
        .ok_or(format!("processor realization '{}' does not exist", resource_config.resource_realization_id))?;
      resources.insert(
        resource_config.resource_id.clone(),
        PipelineResource {
          id: resource_config.resource_id.clone(),
          name,
          realization_id: resource_config.resource_realization_id.clone(),
          technology: realization.descriptor().technology,
          realization,
          parameters: Default::default(),
        },
      );
    }

    let mut processors = HashMap::<ProcessorId, PipelineProcessor>::new();
    for processor_config in &pipeline_config.processors {
      let name = processor_config.name.clone();
      let realization = processor_registry
        .processor_realization(&processor_config.processor_realization_id)
        .ok_or(format!("processor realization '{}' does not exist", processor_config.processor_realization_id))?;
      processors.insert(
        processor_config.processor_id.clone(),
        PipelineProcessor {
          id: processor_config.processor_id.clone(),
          name,
          realization_id: processor_config.processor_realization_id.clone(),
          technology: realization.descriptor(dsh_api_tenant).technology,
          realization,
          parameters: Default::default(),
          profile_id: None,
        },
      );
    }

    let mut connections = vec![];
    for connection_config in &pipeline_config.connections {
      match connection_config {
        PipelineConnectionConfig::ResourcesToProcessor { source_resource_ids, target_processor_junction } => {
          let source_resource_identifiers = Self::get_resource_identifiers(&resources, source_resource_ids, &pipeline_config.pipeline_id)?;
          let (target_processor_realization_id, target_processor_descriptor, inbound_junction_descriptor) =
            Self::get_processor_inbound_junction(dsh_api_tenant, &pipeline_config, &processors, target_processor_junction)?;
          if let Some((_, incompatible_resource)) = resources
            .iter()
            .find(|(_, resource)| !inbound_junction_descriptor.is_resource_technology_compatible(&resource.technology))
          {
            return Err(format!(
              "source resource '{}' has technology '{}' while inbound junction '{}.{}' expects technology '{}'",
              incompatible_resource.id,
              incompatible_resource.technology,
              target_processor_descriptor.id,
              inbound_junction_descriptor.id,
              inbound_junction_descriptor.junction_technology
            ));
          }
          let connection = PipelineConnection {
            connection: ConnectionType::ResourcesToProcessor {
              source_resources: source_resource_identifiers,
              target_junction: JunctionIdentifier::Processor(
                target_processor_descriptor.technology,
                target_processor_realization_id,
                inbound_junction_descriptor.id,
              ),
            },
            parameters: HashMap::<ParameterId, String>::new(),
          };
          connections.push(connection);
        }

        PipelineConnectionConfig::ProcessorToResources { source_processor_junction, target_resource_ids } => {
          let (source_processor_realization_id, source_processor_descriptor, outbound_junction_descriptor) =
            Self::get_processor_outbound_junction(dsh_api_tenant, &pipeline_config, &processors, source_processor_junction)?;
          let target_resource_identifiers = Self::get_resource_identifiers(&resources, target_resource_ids, &pipeline_config.pipeline_id)?;
          let connection = PipelineConnection {
            connection: ConnectionType::ProcessorToResources {
              source_junction: JunctionIdentifier::Processor(
                source_processor_descriptor.technology,
                source_processor_realization_id,
                outbound_junction_descriptor.id.clone(),
              ),
              target_resources: target_resource_identifiers,
            },
            parameters: HashMap::<ParameterId, String>::new(),
          };
          connections.push(connection);
        }

        PipelineConnectionConfig::ProcessorToProcessor { source_processor_junction, target_processor_junction } => {
          let (source_processor_realization_id, source_processor_descriptor, outbound_junction_descriptor) =
            Self::get_processor_outbound_junction(dsh_api_tenant, &pipeline_config, &processors, source_processor_junction)?;
          let (target_processor_realization_id, target_processor_descriptor, inbound_junction_descriptor) =
            Self::get_processor_inbound_junction(dsh_api_tenant, &pipeline_config, &processors, target_processor_junction)?;
          let connection = PipelineConnection {
            connection: ConnectionType::ProcessorToProcessor {
              source_junction: JunctionIdentifier::Processor(
                source_processor_descriptor.technology,
                source_processor_realization_id,
                outbound_junction_descriptor.id.clone(),
              ),
              target_junction: JunctionIdentifier::Processor(
                target_processor_descriptor.technology,
                target_processor_realization_id,
                inbound_junction_descriptor.id.clone(),
              ),
            },
            parameters: HashMap::<ParameterId, String>::new(),
          };
          connections.push(connection);
        }
      };
    }

    #[allow(unused_mut)]
    let mut dependencies = vec![];

    Ok(Pipeline { id: pipeline_config.pipeline_id, name: pipeline_config.name, resources, processors, connections, dependencies })
  }

  fn get_processor_inbound_junction(
    dsh_api_tenant: &DshApiTenant,
    pipeline_config: &PipelineConfig,
    processors: &HashMap<ProcessorId, PipelineProcessor>,
    processor_junction: &ProcessorJunctionConfig,
  ) -> Result<(ProcessorRealizationId, ProcessorDescriptor, JunctionDescriptor), String> {
    let (processor_realization_id, processor_descriptor) = match processors.get(&processor_junction.processor_id) {
      Some(processor) => (processor.realization_id.clone(), processor.realization.descriptor(dsh_api_tenant)),
      None => {
        return Err(format!(
          "pipeline '{}' contains no definition for target processor '{}'",
          pipeline_config.pipeline_id, processor_junction.processor_id
        ))
      }
    };
    let inbound_junction_descriptor = match processor_descriptor
      .inbound_junctions
      .iter()
      .find(|inbound_junction| inbound_junction.id == processor_junction.junction)
    {
      Some(inbound_junction_descriptor) => inbound_junction_descriptor.to_owned(),
      None => {
        return Err(format!(
          "target processor '{}' has no inbound junction '{}'",
          processor_descriptor.id, processor_junction.junction
        ))
      }
    };
    Ok((processor_realization_id, processor_descriptor, inbound_junction_descriptor))
  }

  fn get_processor_outbound_junction(
    dsh_api_tenant: &DshApiTenant,
    pipeline_config: &PipelineConfig,
    processors: &HashMap<ProcessorId, PipelineProcessor>,
    processor_junction: &ProcessorJunctionConfig,
  ) -> Result<(ProcessorRealizationId, ProcessorDescriptor, JunctionDescriptor), String> {
    let (processor_realization_id, processor_descriptor) = match processors.get(&processor_junction.processor_id) {
      Some(processor) => (processor.realization_id.clone(), processor.realization.descriptor(dsh_api_tenant)),
      None => {
        return Err(format!(
          "pipeline '{}' contains no definition for target processor '{}'",
          pipeline_config.pipeline_id, processor_junction.processor_id
        ))
      }
    };
    let outbound_junction_descriptor = match processor_descriptor
      .outbound_junctions
      .iter()
      .find(|outbound_junction| outbound_junction.id == processor_junction.junction)
    {
      Some(outbound_junction_descriptor) => outbound_junction_descriptor.to_owned(),
      None => {
        return Err(format!(
          "target processor '{}' has no outbound junction '{}'",
          processor_descriptor.id, processor_junction.junction
        ))
      }
    };
    Ok((processor_realization_id, processor_descriptor, outbound_junction_descriptor))
  }

  fn get_resource_identifiers(resources: &HashMap<ResourceId, PipelineResource>, resource_ids: &[ResourceId], pipeline_id: &PipelineId) -> Result<Vec<ResourceIdentifier>, String> {
    let resource_identifiers = resource_ids
      .iter()
      .filter_map(|id| resources.get(id))
      .collect::<Vec<_>>()
      .iter()
      .map(|resource| ResourceIdentifier { resource_type: resource.technology.clone(), id: resource.realization_id.clone() })
      .collect::<Vec<_>>();
    if resource_identifiers.is_empty() {
      Err(format!("connection with empty resources list in pipeline '{}'", pipeline_id))
    } else {
      Ok(resource_identifiers)
    }
  }

  /// # Deploy this `PipelineInstance`
  ///
  /// ## Parameters
  /// * `inbound_junctions`  - Map containing the inbound resources.
  /// * `outbound_junctions` - Map containing the outbound resources.
  /// * `deploy_parameters`  - Map containing the deployment parameters.
  /// * `profile_id`         - Profile id.
  ///
  /// ## Returns
  /// * `Ok<()>`   - when the deployment request was successfully sent.
  /// * `Err(msg)` - when the deployment request could not be sent.
  async fn deploy(
    &self,
    _inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    _outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    _deploy_parameters: &HashMap<ParameterId, String>,
    _profile_id: Option<&ProfileId>, // TODO Move this to start() method
  ) -> Result<(), String> {
    todo!()
  }

  /// # Dry-run for deployment of this `PipelineInstance`
  ///
  /// This method does everything that the regular `deploy()` method does,
  /// except for the actual deployment to the target platform.
  /// Instead, it returns the configuration that would be used if the deployment would be real.
  ///
  /// ## Parameters
  /// * `inbound_junctions`  - Map containing the inbound resources.
  /// * `outbound_junctions` - Map containing the outbound resources.
  /// * `deploy_parameters`  - Map containing the deployment parameters.
  /// * `profile_id`         - Profile id.
  ///
  /// ## Returns
  /// * `Ok<String>` - when the deployment request was successfully sent.
  /// * `Err(msg)`   - when the deployment request could not be sent.
  async fn deploy_dry_run(
    &self,
    _inbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    _outbound_junctions: &HashMap<JunctionId, Vec<ResourceIdentifier>>,
    _deploy_parameters: &HashMap<ParameterId, String>,
    _profile_id: Option<&ProfileId>, // TODO Move this to start() method
  ) -> Result<String, String> {
    todo!()
  }

  /// # Get the resources compatible with this `PipelineInstance`
  ///
  /// ## Parameters
  /// * `junction_id` - identifies the junction for which the compatible resources need to be
  ///                   retrieved.
  ///
  /// ## Returns
  /// * `Ok<Vec<ResourceIdentifier>` - list of identifiers of compatible resources.
  /// * `Err(msg)`                   - when the list could not be composed.
  async fn compatible_resources(&self, _junction_id: &JunctionId) -> Result<Vec<ResourceIdentifier>, String> {
    todo!()
  }

  /// # Returns the pipeline id of this `PipelineInstance`
  ///
  /// ## Returns
  /// * The `PipelineId` of this `PipelineInstance`.
  fn pipeline_id(&self) -> &PipelineId {
    todo!()
  }

  /// # Start this `PipelineInstance`
  ///
  /// ## Returns
  /// * `Ok<true>`  - when the start request was successfully sent.
  /// * `Ok<false>` - when no pipeline instance with `pipeline_id` exists.
  /// * `Err(msg)`  - when the start request could not be sent.
  async fn start(&self) -> Result<bool, String> {
    todo!()
  }

  /// # Get this `PipelineInstance`s status
  ///
  /// ## Returns
  /// * `Ok<PipelineStatus>` - signals whether the pipeline instance  with the given
  ///                           `pipeline` is active or not.
  /// * `Err(msg)`            - when the status request could not be sent.
  async fn status(&self) -> Result<PipelineStatus, String> {
    todo!()
  }

  /// # Stop this `PipelineInstance`
  ///
  /// ## Returns
  /// * `Ok<true>`  - when the stop request was successfully sent.
  /// * `Ok<false>` - when no pipeline instance with `pipeline_id` exists.
  /// * `Err(msg)`  - when the stop request could not be sent.
  async fn stop(&self) -> Result<bool, String> {
    todo!()
  }

  /// # Undeploy this `PipelineInstance`
  ///
  /// ## Returns
  /// * `Ok<true>`  - when the undeployment request was successfully sent.
  /// * `Ok<false>` - when no pipeline instance with `pipeline_id` exists.
  /// * `Err(msg)`  - when the undeployment request could not be sent.
  async fn undeploy(&self) -> Result<bool, String> {
    todo!()
  }
}

pub struct PipelineResource<'a> {
  id: ResourceId,
  name: String,
  realization_id: ResourceRealizationId,
  technology: ResourceTechnology,
  realization: &'a dyn ResourceRealization,
  parameters: HashMap<ParameterId, String>,
}

pub struct PipelineProcessor<'a> {
  id: ProcessorId,
  name: String,
  realization_id: ProcessorRealizationId,
  technology: ProcessorTechnology,
  realization: &'a dyn ProcessorRealization,
  parameters: HashMap<ParameterId, String>,
  profile_id: Option<ProfileId>,
}

pub struct PipelineConnection {
  connection: ConnectionType,
  parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConnectionType {
  ResourcesToProcessor { source_resources: Vec<ResourceIdentifier>, target_junction: JunctionIdentifier },
  ProcessorToResources { source_junction: JunctionIdentifier, target_resources: Vec<ResourceIdentifier> },
  ProcessorToProcessor { source_junction: JunctionIdentifier, target_junction: JunctionIdentifier },
}

pub struct PipelineDependency {
  parameters: HashMap<ParameterId, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DependencyType {
  ProcessorOnProcessor { depended: ProcessorIdentifier, depends_on: ProcessorIdentifier },
  ProcessorOnResource { depended: JunctionIdentifier, depends_on: Vec<ResourceIdentifier> },
  ResourceOnProcessor { depended: ProcessorIdentifier, depends_on: ResourceIdentifier },
}

#[derive(Debug)]
pub struct PipelineStatus {
  pub deployed: bool,
  pub up: Option<bool>,
}

impl Display for Pipeline<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}", self.id)?;
    if !self.resources.is_empty() {
      writeln!(f, "  resources")?;
      let mut resources = self.resources.values().collect::<Vec<_>>();
      resources.sort_by(|pr1, pr2| pr1.id.cmp(&pr2.id));
      for resource in resources {
        writeln!(f, "    {}", resource)?;
      }
    }
    if !self.processors.is_empty() {
      writeln!(f, "  processors")?;
      let mut processors = self.processors.values().collect::<Vec<_>>();
      processors.sort_by(|pp1, pp2| pp1.id.cmp(&pp2.id));
      for processor in processors {
        writeln!(f, "    {}:{}:{}", processor.id, processor.technology, processor.realization_id)?;
        if let Some(inbound_junctions) = processor.realization.inbound_junction_descriptors() {
          writeln!(
            f,
            "      inbound junctions: {}",
            inbound_junctions
              .iter()
              .map(|junction_descriptor| format!("{}:{}", junction_descriptor.id, junction_descriptor.junction_technology))
              .collect::<Vec<_>>()
              .join(", ")
          )?;
        }
        if let Some(outbound_junctions) = processor.realization.outbound_junction_descriptors() {
          writeln!(
            f,
            "      outbound junctions: {}",
            outbound_junctions
              .iter()
              .map(|junction_descriptor| format!("{}:{}", junction_descriptor.id, junction_descriptor.junction_technology))
              .collect::<Vec<_>>()
              .join(", ")
          )?;
        }
      }
    }
    if !self.connections.is_empty() {
      writeln!(f, "  connections")?;
      for connection in &self.connections {
        writeln!(f, "    {}", connection)?;
      }
    }
    Ok(())
  }
}

impl Display for PipelineResource<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}({},{})", self.id, self.technology, self.realization_id, self.name)?;
    if !self.parameters.is_empty() {
      write!(
        f,
        ", {}",
        self
          .parameters
          .iter()
          .map(|(key, value)| format!("{}:{}", key, value))
          .collect::<Vec<_>>()
          .join(", ")
      )?
    }
    Ok(())
  }
}

impl Display for PipelineProcessor<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}({},{})", self.id, self.technology, self.realization_id, self.name)?;
    if let Some(profile_id) = &self.profile_id {
      write!(f, " (profile id: {})", profile_id)?;
    }
    if !self.parameters.is_empty() {
      write!(
        f,
        ", {}",
        self
          .parameters
          .iter()
          .map(|(key, value)| format!("{}:{}", key, value))
          .collect::<Vec<_>>()
          .join(", ")
      )?
    }
    Ok(())
  }
}

impl Display for PipelineConnection {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self.connection {
      ConnectionType::ResourcesToProcessor { source_resources, target_junction } => {
        write!(
          f,
          "[{}] -> {}",
          source_resources.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", "),
          target_junction
        )
      }
      ConnectionType::ProcessorToResources { source_junction, target_resources } => {
        write!(
          f,
          "{} -> [{}]",
          source_junction,
          target_resources.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ")
        )
      }
      ConnectionType::ProcessorToProcessor { source_junction, target_junction } => {
        write!(f, "{} -> {}", source_junction, target_junction)
      }
    }
  }
}

impl Display for ConnectionType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ConnectionType::ResourcesToProcessor { source_resources, target_junction } => {
        write!(
          f,
          "r2p: {} -> {}",
          source_resources.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(", "),
          target_junction
        )
      }
      ConnectionType::ProcessorToResources { source_junction, target_resources } => {
        write!(
          f,
          "p2r: {} -> {}",
          source_junction,
          target_resources.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(", ")
        )
      }
      ConnectionType::ProcessorToProcessor { source_junction, target_junction } => {
        write!(f, "p2p: {} -> {}", source_junction, target_junction)
      }
    }
  }
}

impl Display for PipelineDependency {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      self
        .parameters
        .iter()
        .map(|(key, value)| format!("{}: {}", key, value))
        .collect::<Vec<String>>()
        .join(", ")
    )
  }
}

impl Display for PipelineStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.deployed {
      match self.up {
        Some(up) => {
          if up {
            write!(f, "deployed:up")
          } else {
            write!(f, "deployed:down")
          }
        }
        None => write!(f, "deployed:unknown"),
      }
    } else {
      write!(f, "not-deployed")
    }
  }
}
