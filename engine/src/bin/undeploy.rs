use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{ProcessorId, ProcessorType, ServiceId};

const PROCESSOR_ID: &str = "consentfilter";
const SERVICE_ID: &str = "test002";

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service = processor_registry
    .processor(ProcessorType::DshService, &ProcessorId::try_from(PROCESSOR_ID)?)
    .ok_or("")?;
  println!("{}", dsh_service.undeploy(&ServiceId::new(SERVICE_ID)).await?);
  Ok(())
}
