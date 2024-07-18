use trifonius_engine::processor::processor_registry::ProcessorRegistry;
use trifonius_engine::processor::ProcessorType;

const SERVICE_ID: &str = "test-0-0-2";

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();
  let processor_registry = ProcessorRegistry::default();
  let application = processor_registry.processor(ProcessorType::Application, "consentfilter").ok_or("")?;
  println!("{}", application.undeploy(SERVICE_ID).await?);
  Ok(())
}
