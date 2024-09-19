//! Client for accessing the DSH api

use std::fmt::{Display, Formatter};

use bytes::Bytes;
use futures::TryStreamExt;
use progenitor_client::{ByteStream, Error as ProgenitorError, ResponseValue as ProgenitorResponseValue};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::Serialize;

use crate::platform::DshPlatform;
use crate::types::error::ConversionError;
use crate::{generated::Client as GeneratedClient, DshApiError, DshApiTenant};

#[derive(Debug)]
pub struct DshApiClient<'a> {
  token: String,
  pub(crate) generated_client: &'a GeneratedClient,
  tenant: &'a DshApiTenant,
}

pub(crate) enum DshApiResponseStatus {
  Accepted,
  Created,
  NoContent,
  Ok,
  Unknown,
}

pub(crate) type DshApiProcessResult<T> = Result<(DshApiResponseStatus, T), DshApiError>;

impl<'a> DshApiClient<'a> {
  pub fn new(token: String, generated_client: &'a GeneratedClient, tenant: &'a DshApiTenant) -> Self {
    Self { token, generated_client, tenant }
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
    self.tenant.name()
  }

  pub fn platform(&self) -> &DshPlatform {
    self.tenant.platform()
  }

  pub fn user(&self) -> &str {
    self.tenant.user()
  }

  pub fn token(&self) -> &str {
    &self.token
  }
}

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

impl From<ConversionError> for DshApiError {
  fn from(value: ConversionError) -> Self {
    DshApiError::Unexpected(value.to_string())
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
