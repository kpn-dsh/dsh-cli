use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{ProcessorId, ProcessorType, ServiceName};

const PROCESSOR_ID: &str = "greenbox-consent-filter";
const SERVICE_NAME: &str = "consentfilter-test002";

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service = processor_registry
    .processor(ProcessorType::DshService, &ProcessorId::try_from(PROCESSOR_ID)?)
    .ok_or("")?;
  let status = dsh_service.status(&ServiceName::new(SERVICE_NAME)).await?;
  println!("{}", status);
  Ok(())
}
