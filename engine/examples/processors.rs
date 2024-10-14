use dsh_api::dsh_api_tenant::DEFAULT_DSH_API_TENANT;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[path = "common.rs"]
mod common;

fn main() {
  let dsh_api_tenant = &DEFAULT_DSH_API_TENANT;
  let processor_descriptors = ProcessorRegistry::default().processor_descriptors(&dsh_api_tenant);
  println!("{}", serde_json::to_string_pretty(&processor_descriptors).unwrap());
}
