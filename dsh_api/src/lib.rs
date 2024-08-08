#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::env;
use std::fmt::{Display, Formatter};

use dsh_sdk::RestTokenFetcher;
use lazy_static::lazy_static;

pub use crate::generated::types;
use crate::generated::Client as GeneratedClient;
use crate::platform::DshPlatform;

pub mod app_catalog;
pub mod app_catalog_app_configuration;
pub mod app_catalog_manifest;
pub mod application;
pub mod dsh_api_client;
pub mod platform;
pub mod secret;
pub mod topic;

// Private module `generated` will contain the generated Client code.
mod generated {
  include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}
// tenant is implicit in the client

// get_[SUBJECT]s              get all SUBJECTs
// get_xxx                     get one SUBJECT by implicit subject key
// get_xxx_by_yyy              get one SUBJECT by explicit subject key

#[derive(Clone, Debug)]
pub struct DshApiTenant {
  name: String,
  user: String,
  platform: DshPlatform,
}

#[derive(Debug)]
pub struct DshApiClient<'a> {
  token: String,
  generated_client: &'a GeneratedClient,
  tenant: &'a DshApiTenant,
}

#[derive(Debug)]
pub struct DshApiClientFactory {
  token_fetcher: RestTokenFetcher,
  generated_client: GeneratedClient,
  tenant: DshApiTenant,
}

#[derive(Debug)]
pub enum DshApiError {
  NotAuthorized,
  NotFound,
  Unexpected(String),
}

pub type DshApiResult<T> = Result<T, DshApiError>;

const TRIFONIUS_TARGET: &str = "TRIFONIUS_TARGET";
const TRIFONIUS_TARGET_TENANT: &str = "TRIFONIUS_TARGET_TENANT";
const TRIFONIUS_TARGET_PLATFORM: &str = "TRIFONIUS_TARGET_PLATFORM";

lazy_static! {
  pub static ref DEFAULT_DSH_API_CLIENT_FACTORY: DshApiClientFactory = {
    let tenant_name = get_env(TRIFONIUS_TARGET_TENANT);
    let tenant_env_name = tenant_name.to_ascii_uppercase().replace('-', "_");
    let user = get_env(format!("{}_TENANT_{}_USER", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let secret = get_env(format!("{}_TENANT_{}_SECRET", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let platform = DshPlatform::try_from(get_env(TRIFONIUS_TARGET_PLATFORM).as_str()).unwrap();
    let dsh_api_tenant = DshApiTenant { platform, name: tenant_name, user };
    DshApiClientFactory::create(dsh_api_tenant, secret).expect("could not create static target client factory")
  };
}

impl Display for DshApiError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshApiError::NotAuthorized => write!(f, "not authorized"),
      DshApiError::NotFound => write!(f, "not found"),
      DshApiError::Unexpected(message) => write!(f, "unexpected error ({})", message),
    }
  }
}

impl From<String> for DshApiError {
  fn from(value: String) -> Self {
    DshApiError::Unexpected(value)
  }
}

impl From<DshApiError> for String {
  fn from(value: DshApiError) -> Self {
    value.to_string()
  }
}

fn get_env(name: &str) -> String {
  match env::var(name) {
    Ok(value) => value,
    Err(_) => panic!("environment variable {} not set", name),
  }
}
