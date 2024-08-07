use std::fmt::{Display, Formatter};

use dsh_sdk::RestTokenFetcherBuilder;
use progenitor_client::{Error as ProgenitorError, ResponseValue as ProgenitorResponseValue};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::Serialize;

use crate::platform::DshPlatform;
use crate::{Client as GeneratedClient, DshApiClient, DshApiClientFactory, DshApiError, DshApiTenant};

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
  pub fn api_version(&self) -> &'static str {
    self.generated_client.api_version()
  }

  pub(crate) fn process<T: Serialize>(&self, progenitor_response: Result<ProgenitorResponseValue<T>, ProgenitorError>) -> DshApiProcessResult<T> {
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

  pub fn platform(&self) -> &DshPlatform {
    &self.dsh_api_client_factory.dsh_api_tenant.platform
  }

  pub fn tenant(&self) -> &str {
    &self.dsh_api_client_factory.dsh_api_tenant.tenant
  }

  pub fn user(&self) -> &str {
    &self.dsh_api_client_factory.dsh_api_tenant.user
  }

  pub fn generated_client(&self) -> &GeneratedClient {
    self.generated_client
  }

  pub fn token(&self) -> &str {
    &self.token
  }
}

impl DshApiClientFactory {
  pub fn platform(&self) -> &DshPlatform {
    &self.dsh_api_tenant.platform
  }

  pub fn target_tenant(&self) -> &DshApiTenant {
    &self.dsh_api_tenant
  }

  pub fn tenant(&self) -> &str {
    &self.dsh_api_tenant.tenant
  }

  pub fn user(&self) -> &str {
    &self.dsh_api_tenant.user
  }

  pub fn create(dsh_api_tenant: DshApiTenant, client_secret: String) -> Result<Self, String> {
    match RestTokenFetcherBuilder::new(dsh_api_tenant.platform.sdk_platform())
      .tenant_name(dsh_api_tenant.tenant.clone())
      .client_secret(client_secret)
      .build()
    {
      Ok(token_fetcher) => {
        let generated_client = GeneratedClient::new(dsh_api_tenant.platform.endpoint_rest_api().as_str());
        Ok(DshApiClientFactory { generated_client, token_fetcher, dsh_api_tenant })
      }
      Err(e) => Err(format!("could not create token fetcher ({})", e)),
    }
  }

  pub async fn client(&self) -> Result<DshApiClient, String> {
    match self.token_fetcher.get_token().await {
      Ok(token) => Ok(DshApiClient { generated_client: &self.generated_client, token, dsh_api_client_factory: self }),
      Err(e) => Err(format!("could not create token ({})", e)),
    }
  }
}

impl DshApiTenant {
  pub fn platform(&self) -> &DshPlatform {
    &self.platform
  }

  pub fn tenant(&self) -> &String {
    &self.tenant
  }

  pub fn user(&self) -> &String {
    &self.user
  }

  pub fn app_domain(&self) -> Option<String> {
    self.platform.app_domain(&self.tenant)
  }

  pub fn console_url(&self) -> Option<String> {
    self.platform.console_url()
  }

  pub fn dsh_internal_domain(&self) -> Option<String> {
    self.platform.dsh_internal_domain(&self.tenant)
  }

  pub fn monitoring_url(&self) -> Option<String> {
    self.platform.monitoring_url(&self.tenant)
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
