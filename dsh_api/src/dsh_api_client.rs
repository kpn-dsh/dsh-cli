//! Client for accessing the DSH api

use std::env;
use std::fmt::{Display, Formatter};

use bytes::Bytes;
use dsh_sdk::Platform as SdkPlatform;
use dsh_sdk::RestTokenFetcherBuilder;
use futures::TryStreamExt;
use lazy_static::lazy_static;
use progenitor_client::{ByteStream, Error as ProgenitorError, ResponseValue as ProgenitorResponseValue};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::Serialize;

use crate::platform::DshPlatform;
use crate::{generated::Client as GeneratedClient, DshApiClient, DshApiClientFactory, DshApiError, DshApiTenant};

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

pub(crate) enum DshApiResponseStatus {
  Accepted,
  Created,
  NoContent,
  Ok,
  Unknown,
}

pub(crate) type DshApiProcessResult<T> = Result<(DshApiResponseStatus, T), DshApiError>;

impl Display for DshApiResponseStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshApiResponseStatus::Accepted => write!(f, "accepted"),
      DshApiResponseStatus::Created => write!(f, "created"),
      DshApiResponseStatus::NoContent => write!(f, "no content"),
      DshApiResponseStatus::Ok => write!(f, "ok"),
      DshApiResponseStatus::Unknown => write!(f, "unknown"),
    }
  }
}

impl From<ReqwestStatusCode> for DshApiResponseStatus {
  fn from(status: ReqwestStatusCode) -> Self {
    match status {
      ReqwestStatusCode::ACCEPTED => Self::Accepted,
      ReqwestStatusCode::CREATED => Self::Created,
      ReqwestStatusCode::NO_CONTENT => Self::NoContent,
      ReqwestStatusCode::OK => Self::Ok,
      _ => Self::Unknown,
    }
  }
}

impl From<ProgenitorError> for DshApiError {
  fn from(progenitor_error: ProgenitorError) -> Self {
    match progenitor_error {
      ProgenitorError::InvalidRequest(string) => DshApiError::Unexpected(format!("invalid request ({})", string)),
      ProgenitorError::CommunicationError(reqwest_error) => DshApiError::Unexpected(format!("communication error (reqwest error: {})", reqwest_error)),
      ProgenitorError::InvalidUpgrade(reqwest_error) => DshApiError::Unexpected(format!("invalid upgrade (reqwest error: {})", reqwest_error)),
      ProgenitorError::ErrorResponse(progenitor_response_value) => DshApiError::Unexpected(format!("error response (progenitor response value: {:?})", progenitor_response_value)),
      ProgenitorError::ResponseBodyError(reqwest_error) => DshApiError::Unexpected(format!("response body error (reqwest error: {})", reqwest_error)),
      ProgenitorError::InvalidResponsePayload(_bytes, json_error) => DshApiError::Unexpected(format!("invalid response payload (json error: {})", json_error)),
      ProgenitorError::UnexpectedResponse(reqwest_response) => match reqwest_response.status() {
        ReqwestStatusCode::NOT_FOUND => DshApiError::NotFound,
        ReqwestStatusCode::UNAUTHORIZED | ReqwestStatusCode::FORBIDDEN | ReqwestStatusCode::METHOD_NOT_ALLOWED => DshApiError::NotAuthorized,
        other_status_code => DshApiError::Unexpected(format!(
          "unexpected response (status: {}, reqwest response: {:?})",
          other_status_code, reqwest_response
        )),
      },
      ProgenitorError::PreHookError(string) => DshApiError::Unexpected(format!("pre-hook error ({})", string)),
    }
  }
}

impl DshApiClient<'_> {
  pub async fn default_client() -> Self {
    DEFAULT_DSH_API_CLIENT_FACTORY.client().await.expect("could not create dsh api client")
  }

  pub fn api_version(&self) -> &'static str {
    self.generated_client.api_version()
  }

  pub(crate) fn process<T>(&self, progenitor_response: Result<ProgenitorResponseValue<T>, ProgenitorError>) -> DshApiProcessResult<T>
  where
    T: Serialize,
  {
    match progenitor_response {
      Ok::<ProgenitorResponseValue<T>, ProgenitorError>(response) => Ok((DshApiResponseStatus::from(response.status()), response.into_inner())),
      Err(progenitor_error) => Err(DshApiError::from(progenitor_error)),
    }
  }

  pub(crate) fn process_raw<T>(&self, progenitor_response: Result<ProgenitorResponseValue<T>, ProgenitorError>) -> DshApiProcessResult<T> {
    match progenitor_response {
      Ok::<ProgenitorResponseValue<T>, ProgenitorError>(response) => Ok((DshApiResponseStatus::from(response.status()), response.into_inner())),
      Err(progenitor_error) => Err(DshApiError::from(progenitor_error)),
    }
  }

  pub(crate) async fn process_string(&self, progenitor_response: Result<ProgenitorResponseValue<ByteStream>, ProgenitorError>) -> DshApiProcessResult<String> {
    match progenitor_response {
      Ok(response) => {
        let status = DshApiResponseStatus::from(response.status());
        let mut inner = response.into_inner();
        let mut string = String::new();
        while let Some::<Bytes>(ref bytes) = inner.try_next().await? {
          string.push_str(std::str::from_utf8(bytes)?)
        }
        Ok((status, string))
      }
      Err(progenitor_error) => Err(DshApiError::from(progenitor_error)),
    }
  }

  pub fn tenant(&self) -> &DshApiTenant {
    self.tenant
  }

  pub fn tenant_name(&self) -> &str {
    &self.tenant.name
  }

  pub fn platform(&self) -> &DshPlatform {
    &self.tenant.platform
  }

  pub fn user(&self) -> &str {
    &self.tenant.user
  }

  pub fn token(&self) -> &str {
    &self.token
  }
}

const TRIFONIUS_TARGET: &str = "TRIFONIUS_TARGET";
const TRIFONIUS_TARGET_TENANT: &str = "TRIFONIUS_TARGET_TENANT";
const TRIFONIUS_TARGET_PLATFORM: &str = "TRIFONIUS_TARGET_PLATFORM";

lazy_static! {
  static ref DEFAULT_DSH_API_CLIENT_FACTORY: DshApiClientFactory = {
    let tenant_name = get_env(TRIFONIUS_TARGET_TENANT);
    let tenant_env_name = tenant_name.to_ascii_uppercase().replace('-', "_");
    let user = get_env(format!("{}_TENANT_{}_USER", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let secret = get_env(format!("{}_TENANT_{}_SECRET", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let platform = DshPlatform::try_from(get_env(TRIFONIUS_TARGET_PLATFORM).as_str()).unwrap();
    let dsh_api_tenant = DshApiTenant { platform, name: tenant_name, user };
    DshApiClientFactory::create(dsh_api_tenant, secret).expect("could not create static target client factory")
  };
}

impl DshApiClientFactory {
  pub fn default_factory() -> &'static DshApiClientFactory {
    &DEFAULT_DSH_API_CLIENT_FACTORY
  }

  pub fn platform(&self) -> &DshPlatform {
    &self.tenant.platform
  }

  pub fn tenant(&self) -> &DshApiTenant {
    &self.tenant
  }

  pub fn tenant_name(&self) -> &str {
    &self.tenant.name
  }

  pub fn user(&self) -> &str {
    &self.tenant.user
  }

  pub fn create(dsh_api_tenant: DshApiTenant, client_secret: String) -> Result<Self, String> {
    match RestTokenFetcherBuilder::new(SdkPlatform::from(&dsh_api_tenant.platform))
      .tenant_name(dsh_api_tenant.name().clone())
      .client_secret(client_secret)
      .build()
    {
      Ok(token_fetcher) => {
        let generated_client = GeneratedClient::new(dsh_api_tenant.platform.endpoint_rest_api().as_str());
        Ok(DshApiClientFactory { token_fetcher, generated_client, tenant: dsh_api_tenant })
      }
      Err(e) => Err(format!("could not create token fetcher ({})", e)),
    }
  }

  pub async fn client(&self) -> Result<DshApiClient, String> {
    match self.token_fetcher.get_token().await {
      Ok(token) => Ok(DshApiClient { token, generated_client: &self.generated_client, tenant: &self.tenant }),
      Err(e) => Err(format!("could not create token ({})", e)),
    }
  }
}

impl DshApiTenant {
  pub fn platform(&self) -> &DshPlatform {
    &self.platform
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn user(&self) -> &String {
    &self.user
  }

  pub fn app_domain(&self) -> Option<String> {
    self.platform.app_domain(&self.name)
  }

  pub fn console_url(&self) -> Option<String> {
    self.platform.console_url()
  }

  pub fn dsh_internal_domain(&self) -> Option<String> {
    self.platform.dsh_internal_domain(&self.name)
  }

  pub fn monitoring_url(&self) -> Option<String> {
    self.platform.monitoring_url(&self.name)
  }

  pub fn public_vhosts_domain(&self) -> Option<String> {
    self.platform.public_vhosts_domain()
  }

  pub fn realm(&self) -> String {
    self.platform.realm()
  }

  pub fn endpoint_rest_access_token(&self) -> String {
    self.platform.endpoint_rest_access_token()
  }

  pub fn endpoint_rest_api(&self) -> String {
    self.platform.endpoint_rest_api()
  }
}

fn get_env(name: &str) -> String {
  match env::var(name) {
    Ok(value) => value,
    Err(_) => panic!("environment variable {} not set", name),
  }
}
