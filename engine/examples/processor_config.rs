use trifonius_engine::processor::processor_config::read_processor_config;
use trifonius_engine::processor::ProcessorTechnology;

#[path = "common.rs"]
mod common;

fn main() {
  let config = read_processor_config("config/processors/dshservice/greenbox-consent-filter.toml", ProcessorTechnology::DshService).unwrap();
  println!("{:#?}", config);
}
