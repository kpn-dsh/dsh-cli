#![doc(html_favicon_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]
#![doc(html_logo_url = "https://teamkpn.kpnnet.org/static/images/favicon.svg")]

use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;

use crate::dsh_api_tenant::DshApiTenant;
pub use crate::generated::types;

pub mod app_catalog;
pub mod app_catalog_app_configuration;
pub mod app_catalog_manifest;
pub mod application;
pub mod bucket;
pub mod dsh_api_client;
pub mod dsh_api_client_factory;
pub mod dsh_api_tenant;
pub mod platform;
pub mod secret;
pub mod topic;

// Private module `generated` will contain the generated Client code.
pub(crate) mod generated {
  include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

#[derive(Debug)]
pub enum DshApiError {
  NotAuthorized,
  NotFound,
  Unexpected(String),
}

pub type DshApiResult<T> = Result<T, DshApiError>;

impl Display for DshApiError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshApiError::NotAuthorized => write!(f, "not authorized"),
      DshApiError::NotFound => write!(f, "not found"),
      DshApiError::Unexpected(message) => write!(f, "unexpected error ({})", message),
    }
  }
}

impl From<SerdeJsonError> for DshApiError {
  fn from(error: SerdeJsonError) -> Self {
    DshApiError::Unexpected(error.to_string())
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

// API naming convention
//
// Configuration is what was configured
// Actual is what is actual deployed
// Naming conventions
// create_SUBJECT                        SUBJECT_id?, CONFIG    create SUBJECT
// delete_SUBJECT                        SUBJECT_id             delete SUBJECT
// deploy_SUBJECT                        SUBJECT_id?, CONFIG    deploy SUBJECT
// get_SUBJECT                           SUBJECT_id             get all actual/current SUBJECT data, by SUBJECT_id
// get_SUBJECT_[SUB]_allocation_status   SUBJECT_id, SUB_id     get SUB allocation status, by SUBJECT_id and SUB_id
// get_SUBJECT_actual_configuration      SUBJECT_id             get actual/current configuration, by SUBJECT_id
// get_SUBJECT_actual_configurations                            get actual/current configurations, for all SUBJECTs
// get_SUBJECT_allocation_status         SUBJECT_id             get SUBJECT allocation status, by SUBJECT_id
// get_SUBJECT_configuration             SUBJECT_id             get configuration provided at creation, by SUBJECT_id
// get_SUBJECT_configurations                                   get configurations provided at creation, for all SUBJECTs
// get_SUBJECT_derived_task_ids          SUBJECT_id             get all taskIids for all derived tasks, by SUBJECT_id
// get_SUBJECT_ids                                              get all ids, for all SUBJECTs
// get_SUBJECT_ids_with_derived_tasks                           get ids for all SUBJECTs that have derived tasks
// get_SUBJECT_SPECIFIC                  SUBJECT_id             get SUBJECT specific data, by SUBJECT_id
// get_SUBJECT_SPECIFICs                 SUBJECT_id             get SUBJECT specific data, for all SUBJECTs
// get_SUBJECT_SUB_allocation_status     SUBJECT_id, SUB_id     get SUB allocation status, by SUBJECT_id and SUB_id
// get_SUBJECTs                                                 get all actual/current SUBJECT data, for all SUBJECTs
// undeploy_SUBJECT                      SUBJECT_id             undeploy SUBJECT, by SUBJECT_id
// update_SUBJECT                        SUBJECT_id, CONFIG     deploy SUBJECT, by SUBJECT_id
