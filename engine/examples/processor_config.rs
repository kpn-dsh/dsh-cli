use trifonius_engine::processor::processor_config::read_processor_config;
use trifonius_engine::processor::ProcessorType;

#[path = "common.rs"]
mod common;

fn main() {
  let config = read_processor_config("config/processors/dsh-services/greenbox-consent-filter.toml", ProcessorType::DshService).unwrap();
  println!("{:#?}", config);
}
