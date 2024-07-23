use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::{ProcessorId, ProcessorType, ServiceId};

const SERVICE_ID: &str = "test-0-0-2";

#[tokio::main]
async fn main() -> Result<(), String> {
  let processor_registry = ProcessorRegistry::default();
  let dsh_service = processor_registry
    .processor(ProcessorType::DshService, &ProcessorId::try_from("consentfilter")?)
    .ok_or("")?;
  let status = dsh_service.status(&ServiceId::new(SERVICE_ID)).await?;
  println!("{}", status);
  Ok(())
}
