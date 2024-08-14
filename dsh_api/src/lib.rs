#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::env;
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

use dsh_sdk::RestTokenFetcher;
use lazy_static::lazy_static;
use reqwest::Error as ReqwestError;

pub use crate::generated::types;
use crate::generated::Client as GeneratedClient;
use crate::platform::DshPlatform;

pub mod app_catalog;
pub mod app_catalog_app_configuration;
pub mod app_catalog_manifest;
pub mod application;
pub mod bucket;
pub mod dsh_api_client;
pub mod platform;
pub mod secret;
pub mod topic;

// Private module `generated` will contain the generated Client code.
mod generated {
  include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

// Naming conventions
// deploy_[SUBJECT]                       (subject_id),SUBJECT  deploy/create SUBJECT
// get_[SUBJECT]                          subject_id            get SUBJECT/configuration by subject_id
// get_[SUBJECT]_actual                   subject_id            get deployed SUBJECT/configuration by subject_id
// get_[SUBJECT]_allocation_status        subject_id            get SUBJECT allocation status by subject_id
// get_[SUBJECT]_ids                                            get all SUBJECTs/configurations
// get_[SUBJECT]_[SUB]                    subject_id,sub_id     get SUB/configuration by subject_id and sub_id
// get_[SUBJECT]_[SUB]_actual             subject_id,sub_id     get deployed SUB by subject_id and sub_id
// get_[SUBJECT]_[SUB]_allocation_status  subject_id,sub_id     get SUB allocation status by subject_id and sub_id
// get_[SUBJECT]_[SUB]_ids                subject_id,sub_id     get all SUB ids by subject_id
// get_[SUBJECT]s                                               get all SUBJECTs/configurations
// get_[SUBJECT]s_actual                                        get all deployed SUBJECTs/configurations
// get_[SUBJECT]s_with_[SUB]_ids          subject_id            get ids of all SUBJECTs with SUB
// undeploy_[SUBJECT]                     subject_id            undeploy/delete SUBJECT by subject_id

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

impl From<ReqwestError> for DshApiError {
  fn from(error: ReqwestError) -> Self {
    DshApiError::Unexpected(error.to_string())
  }
}

impl From<Utf8Error> for DshApiError {
  fn from(error: Utf8Error) -> Self {
    DshApiError::Unexpected(error.to_string())
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
