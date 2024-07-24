pub mod macros;
pub mod placeholder;
pub mod platform;
pub mod processor;
pub mod resource;
pub mod target_client;
pub mod topology;

const TRIFONIUS_CONFIG_DIR: &str = "TRIFONIUS_CONFIG_DIR";
const DEFAULT_CONFIG_DIR: &str = "config";

pub(crate) fn config_dir_name() -> String {
  std::env::var(TRIFONIUS_CONFIG_DIR).unwrap_or(DEFAULT_CONFIG_DIR.to_string())
}
