use lazy_static::lazy_static;
use regex::Regex;

pub mod processor;
pub mod resource;
pub mod topology;

lazy_static! {
  static ref ID_REGEX: Regex = Regex::new("^[a-z][a-z0-9_-]{1,30}$").unwrap();
}

pub fn is_valid_id(id: &str) -> bool {
  ID_REGEX.is_match(id)
}
