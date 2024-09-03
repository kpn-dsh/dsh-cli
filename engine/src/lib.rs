#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;

pub mod engine_target;
pub mod macros;
pub mod pipeline;
pub mod placeholder;
pub mod processor;
pub mod resource;

const TRIFONIUS_CONFIG_DIR: &str = "TRIFONIUS_CONFIG_DIR";
const DEFAULT_CONFIG_DIR: &str = "config";

pub(crate) fn config_dir_name() -> String {
  std::env::var(TRIFONIUS_CONFIG_DIR).unwrap_or(DEFAULT_CONFIG_DIR.to_string())
}

identifier!(
  "trifonius_engine",
  ProfileId,
  "profile id",
  "^[a-z][a-z0-9-]{0,49}$",
  "valid-profile-id",
  "invalid_profile_id"
);
