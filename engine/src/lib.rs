use lazy_static::lazy_static;
use regex::Regex;

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

lazy_static! {
  static ref ID_REGEX: Regex = Regex::new("^[a-z][a-z0-9_-]{1,30}$").unwrap();
}

pub fn is_valid_id(id: &str) -> bool {
  ID_REGEX.is_match(id)
}
