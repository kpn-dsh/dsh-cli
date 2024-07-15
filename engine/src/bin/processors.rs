use trifonius_engine::processor::application::DEFAULT_TARGET_CLIENT_FACTOR;
use trifonius_engine::processor::processor_registry::ProcessorRegistry;

#[tokio::main]
async fn main() {
  let processor_registry = ProcessorRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();

  let identifiers = processor_registry.processor_identifiers();
  println!("processor identifiers");
  for identifier in identifiers {
    println!("  {}", identifier);
  }

  let descriptors = processor_registry.processor_descriptors();
  println!("{}", serde_json::to_string_pretty(&descriptors).unwrap());
  // for descriptor in descriptors {
  //   println!("{}", serde_json::to_string_pretty(&descriptor).unwrap());
  //   // println!("{}", &descriptor);
  // }
}
