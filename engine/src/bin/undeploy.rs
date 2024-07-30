use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{ProcessorId, ProcessorType, ServiceName};
use trifonius_engine::target_client::DEFAULT_TARGET_CLIENT_FACTORY;

const PROCESSOR_ID: &str = "greenbox-consent-filter";
const SERVICE_NAME: &str = "consentfilter-test002";

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service_realization = processor_registry
    .processor_realization(ProcessorType::DshService, &ProcessorId::new(PROCESSOR_ID))
    .unwrap();
  let dsh_service = dsh_service_realization.processor(Some(&DEFAULT_TARGET_CLIENT_FACTORY))?;
  println!("{}", dsh_service.undeploy(&ServiceName::new(SERVICE_NAME)).await?);
  Ok(())
}
