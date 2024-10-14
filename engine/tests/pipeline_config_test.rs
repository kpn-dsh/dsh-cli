use dsh_api::dsh_api_tenant::DEFAULT_DSH_API_TENANT;
use trifonius_engine::pipeline::pipeline::Pipeline;
use trifonius_engine::pipeline::pipeline_config::{PipelineConfig, PipelineProcessorConfig, PipelineResourceConfig};
use trifonius_engine::pipeline::PipelineId;
use trifonius_engine::processor::processor_registry::DEFAULT_PROCESSOR_REGISTRY;
use trifonius_engine::processor::{ProcessorId, ProcessorRealizationId};
use trifonius_engine::resource::resource_registry::DEFAULT_RESOURCE_REGISTRY;
use trifonius_engine::resource::{ResourceId, ResourceRealizationId};
use trifonius_engine::version::Version;

static _STR: &str = r#"pipeline-id = "pipeline1"
version = "1.2.3"
name = "Test pipeline"
description = "Test pipeline description"

resources = [
    { resource-id = "res1", name = "DSH Topic Resource 1", resource-realization = "internal-keyring-codomain-values-boss", parameters = { } },
    { resource-id = "res2", name = "DSH Topic Resource 2", resource-realization = "scratch-reference-implementation-compliant", parameters = { } }
]
processors = [
    { processor-id = "proc1", name = "DSH Service Processor 1", processor-realization = "service-topic-1", processor-realization-version = "0.1.2", parameters = { }, profile-id = "profile1" },
    { processor-id = "proc2", name = "DSH Service Processor 2", processor-realization = "service-topic-2", processor-realization-version = "0.1.2", parameters = { }, profile-id = "profile2" },
    { processor-id = "proc3", name = "DSH Service Processor gRPc", processor-realization = "service-grpc-1", processor-realization-version = "0.1.2", parameters = { }, profile-id = "profile1" }
]
connections = [
    { source-resources = ["res1", "res2"], target-processor = { processor-id = "proc1", junction = "inbound-topic" } },
    #    { source-resources = ["res1", "res2"], target-processor = { processor-id = "proc3", junction = "inbound-grpc" } },
    { source-processor = { processor-id = "proc2", junction = "outbound-topic" }, target-resources = ["res1", "res2"] },
    { source-processor = { processor-id = "proc1", junction = "outbound-topic" }, target-processor = { processor-id = "proc2", junction = "inbound-topic" } },
    { source-processor = { processor-id = "proc2", junction = "outbound-topic" }, target-processor = { processor-id = "proc1", junction = "inbound-topic" } }
]
dependencies = []
profiles = []
"#;

fn pipeline_resource_config(resource_id: &str, name: &str, realization_id: &str, version: &str) -> PipelineResourceConfig {
  PipelineResourceConfig {
    resource_id: ResourceId::new(resource_id),
    name: name.to_string(),
    resource_realization_id: ResourceRealizationId::new(realization_id),
    resource_realization_version: if version.is_empty() { None } else { Some(Version::try_from(version).unwrap()) },
    parameters: Default::default(),
  }
}

fn _pipeline_processor_config(processor_id: &str, name: &str, realization_id: &str, version: &str) -> PipelineProcessorConfig {
  PipelineProcessorConfig {
    processor_id: ProcessorId::new(processor_id),
    name: name.to_string(),
    processor_realization_id: ProcessorRealizationId::new(realization_id),
    processor_realization_version: Version::try_from(version).unwrap(),
    parameters: Default::default(),
    profile_id: None,
  }
}

fn pipeline_config(pipeline_id: &str, name: &str, version: &str) -> PipelineConfig {
  PipelineConfig {
    pipeline_id: PipelineId::new(pipeline_id),
    version: Version::try_from(version).unwrap(),
    name: name.to_string(),
    description: format!("description for {}", name),
    resources: vec![],
    processors: vec![],
    connections: vec![],
    dependencies: vec![],
    profiles: vec![],
  }
}

#[test]
fn test() {
  env_logger::builder().format_timestamp(None).format_target(false).format_level(false).init();
  let mut pipeline_config_under_test = pipeline_config("pl1", "PIPELINE1", "1.1.1");
  pipeline_config_under_test.add_resource_config(pipeline_resource_config("res1", "RESOURCE1", "replicator", "1.2.3"));
  let pipeline_under_test = Pipeline::create_from_config(
    &pipeline_config_under_test,
    &DEFAULT_DSH_API_TENANT,
    &DEFAULT_RESOURCE_REGISTRY,
    &DEFAULT_PROCESSOR_REGISTRY,
  )
  .unwrap();
  println!("{}", pipeline_under_test);
}
