use dsh_api::dsh_api_tenant::DshApiTenant;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorTechnology;

#[path = "common.rs"]
mod common;

fn main() {
  let dsh_api_tenant = DshApiTenant::default();
  let dshservice_descriptors = ProcessorRegistry::default().processor_descriptors_by_type(ProcessorTechnology::DshService, &dsh_api_tenant);
  println!("{}", serde_json::to_string_pretty(&dshservice_descriptors).unwrap());
}
