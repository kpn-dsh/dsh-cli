// use dsh_sdk::error::DshRestTokenError::StatusCode;
use std::fmt::{Display, Formatter};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

use log::debug;
use progenitor_client::{Error as RestApiError, ResponseValue as RestApiResponseValue};
use reqwest::StatusCode;
use serde::Serialize;

use crate::target_client::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTORY};

pub mod app;
pub mod app_catalog;
pub mod application;
pub mod platform;
pub mod target_client;
pub mod task;

// tenant is implicit in the client

// get_[SUBJECT]s              get all SUBJECTs
// get_xxx                     get one SUBJECT by implicit subject key
// get_xxx_by_yyy              get one SUBJECT by explicit subject key

pub struct DshApiClient<'a> {
  target_client_factory: &'a TargetClientFactory,
}

impl DshApiClient<'_> {
  pub fn new() -> Self {
    DshApiClient { target_client_factory: &DEFAULT_TARGET_CLIENT_FACTORY }
  }

  pub(crate) fn process_get<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(response.into_inner()),
        other_status_code => {
          debug!(
            "unexpected status code {} from get request\n{}",
            other_status_code,
            serde_json::to_string_pretty(&response.into_inner()).unwrap_or("invalid json".to_string())
          );
          Err(DshApiError::Unexpected(format!("unexpected status code {} from get request", other_status_code)))
        }
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  #[allow(dead_code)]
  pub(crate) fn process_post<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(response.into_inner()),
        StatusCode::CREATED => Ok(response.into_inner()),
        StatusCode::ACCEPTED => Ok(response.into_inner()),
        other_status_code => {
          debug!(
            "unexpected status code {} from post request\n{}",
            other_status_code,
            serde_json::to_string_pretty(&response.into_inner()).unwrap_or("invalid json".to_string())
          );
          Err(DshApiError::Unexpected(format!("unexpected status code {} from post request", other_status_code)))
        }
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  pub(crate) fn process_put<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(response.into_inner()),
        StatusCode::CREATED => Ok(response.into_inner()),
        StatusCode::ACCEPTED => Ok(response.into_inner()),
        StatusCode::NO_CONTENT => Ok(response.into_inner()),
        other_status_code => {
          debug!(
            "unexpected status code {} from put request\n{}",
            other_status_code,
            serde_json::to_string_pretty(&response.into_inner()).unwrap_or("invalid json".to_string())
          );
          Err(DshApiError::Unexpected(format!("unexpected status code {} from put request", other_status_code)))
        }
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  pub(crate) fn process_delete<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(response.into_inner()),
        StatusCode::ACCEPTED => Ok(response.into_inner()),
        StatusCode::NO_CONTENT => Ok(response.into_inner()),
        other_status_code => {
          debug!(
            "unexpected status code {} from delete request\n{}",
            other_status_code,
            serde_json::to_string_pretty(&response.into_inner()).unwrap_or("invalid json".to_string())
          );
          Err(DshApiError::Unexpected(format!("unexpected status code {} from delete request", other_status_code)))
        }
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }
}

impl Default for DshApiClient<'_> {
  fn default() -> Self {
    Self::new()
  }
}

type DshApiResult<T> = Result<T, DshApiError>;

#[derive(Debug)]
pub enum DshApiError {
  NotAuthenticated,
  NotFound,
  Unexpected(String),
}

impl Display for DshApiError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshApiError::NotAuthenticated => write!(f, "not authenticated"),
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
      RestApiError::UnexpectedResponse(response) => match response.status() {
        StatusCode::NOT_FOUND => DshApiError::NotFound,
        other => DshApiError::Unexpected(format!("unexpected status code {}", other)),
      },
      RestApiError::PreHookError(message) => DshApiError::Unexpected(format!("pre-hook error: {}", message)),
    }
  }
}
