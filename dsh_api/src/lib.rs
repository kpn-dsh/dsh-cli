use std::env;
use std::fmt::{Display, Formatter};

use dsh_sdk::RestTokenFetcher;
use lazy_static::lazy_static;
use progenitor_client::Error as RestApiError;

use crate::platform::DshPlatform;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub mod app;
pub mod app_catalog;
pub mod application;
pub mod dsh_api_client;
pub mod platform;
pub mod secret;
pub mod task;

// 200 - OK
// 201 - CREATED
// 202 - ACCEPTED
// 204 - NO_CONTENT
// 400 - BAD_REQUEST
// 401 - UNAUTHORIZED
// 403 - FORBIDDEN
// 404 - NOT_FOUND
// 405 - NOT_ALLOWED
// 500 - INTERNAL_SERVER_ERROR

// DELETE  200,204  resource successfully deleted
//         202      request accepted, result unknown
// GET     200      resource successfully retrieved
// POST    200      resource created successfully
//         201      created new resource
//         202      request accepted, result unknown
// PUT     200,204  resource updated successfully
//         201      created new resource
//         202      request accepted, result unknown

// tenant is implicit in the client

// get_[SUBJECT]s              get all SUBJECTs
// get_xxx                     get one SUBJECT by implicit subject key
// get_xxx_by_yyy              get one SUBJECT by explicit subject key

// pub struct DshApiClient<'a> {
//   dsh_api_client_factory: &'a DshApiClientFactory,
// }

type DshApiResult<T> = Result<T, DshApiError>;

#[derive(Clone, Debug)]
pub struct DshApiTenant {
  platform: DshPlatform,
  tenant: String,
  user: String,
}

#[derive(Debug)]
pub struct DshApiClient<'a> {
  pub(crate) generated_client: &'a Client,
  token: String,
  dsh_api_client_factory: &'a DshApiClientFactory,
}

#[derive(Debug)]
pub struct DshApiClientFactory {
  generated_client: Client,
  token_fetcher: RestTokenFetcher,
  dsh_api_tenant: DshApiTenant,
}

#[derive(Debug)]
pub enum DshApiError {
  NotAuthorized,
  NotFound,
  Unexpected(String),
}

const TRIFONIUS_TARGET: &str = "TRIFONIUS_TARGET";
const TRIFONIUS_TARGET_TENANT: &str = "TRIFONIUS_TARGET_TENANT";
const TRIFONIUS_TARGET_PLATFORM: &str = "TRIFONIUS_TARGET_PLATFORM";

lazy_static! {
  pub static ref DEFAULT_DSH_API_CLIENT_FACTORY: DshApiClientFactory = {
    let tenant = get_env(TRIFONIUS_TARGET_TENANT);
    let tenant_env_name = tenant.to_ascii_uppercase().replace('-', "_");
    let user = get_env(format!("{}_TENANT_{}_USER", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let secret = get_env(format!("{}_TENANT_{}_SECRET", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let platform = DshPlatform::try_from(get_env(TRIFONIUS_TARGET_PLATFORM).as_str()).unwrap();
    let dsh_api_tenant = DshApiTenant { platform, tenant, user };
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

impl From<RestApiError> for DshApiError {
  fn from(rest_api_error: RestApiError) -> Self {
    match rest_api_error {
      RestApiError::InvalidRequest(message) => DshApiError::Unexpected(format!("invalid request: {}", message)),
      RestApiError::CommunicationError(error) => DshApiError::Unexpected(format!("communication error: {}", error)),
      RestApiError::InvalidUpgrade(error) => DshApiError::Unexpected(format!("invalid upgrade error: {}", error)),
      RestApiError::ErrorResponse(response) => DshApiError::Unexpected(format!("error response: {:?}", response)),
      RestApiError::ResponseBodyError(error) => DshApiError::Unexpected(format!("response body error: {}", error)),
      RestApiError::InvalidResponsePayload(_, json_error) => DshApiError::Unexpected(format!("invalid response payload: {}", json_error)),
      RestApiError::UnexpectedResponse(response) => DshApiError::Unexpected(format!("error response: {:?}", response)),
      RestApiError::PreHookError(message) => DshApiError::Unexpected(format!("pre-hook error: {}", message)),
    }
  }
}

fn get_env(name: &str) -> String {
  match env::var(name) {
    Ok(value) => value,
    Err(_) => panic!("environment variable {} not set", name),
  }
}
