use dsh_sdk::RestTokenFetcherBuilder;
use progenitor_client::{Error as RestApiError, ResponseValue as RestApiResponseValue};
use reqwest::StatusCode;
use serde::Serialize;

use crate::platform::DshPlatform;
use crate::{Client as GeneratedClient, DshApiClient, DshApiClientFactory, DshApiError, DshApiTenant};

impl DshApiClient<'_> {
  pub fn api_version(&self) -> &'static str {
    &self.generated_client.api_version()
  }

  pub(crate) fn process_get<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(response.into_inner()),
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => Err(DshApiError::NotAuthorized),
        StatusCode::NOT_FOUND => Err(DshApiError::NotFound),
        other_status_code => Err(DshApiError::Unexpected(format!("unexpected status code {} from get request", other_status_code))),
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  pub(crate) fn process_get_raw<T>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => Ok(response.into_inner()),
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => Err(DshApiError::NotAuthorized),
        StatusCode::NOT_FOUND => Err(DshApiError::NotFound),
        other_status_code => Err(DshApiError::Unexpected(format!("unexpected status code {} from get request", other_status_code))),
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  #[allow(dead_code)]
  pub(crate) fn process_post<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => Ok(response.into_inner()),
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => Err(DshApiError::NotAuthorized),
        StatusCode::NOT_FOUND => Err(DshApiError::NotFound),
        other_status_code => Err(DshApiError::Unexpected(format!("unexpected status code {} from post request", other_status_code))),
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  pub(crate) fn process_put<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => Ok(response.into_inner()),
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => Err(DshApiError::NotAuthorized),
        StatusCode::NOT_FOUND => Err(DshApiError::NotFound),
        other_status_code => Err(DshApiError::Unexpected(format!("unexpected status code {} from put request", other_status_code))),
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
    }
  }

  pub(crate) fn process_delete<T: Serialize>(&self, response: Result<RestApiResponseValue<T>, RestApiError>) -> Result<T, DshApiError> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => Ok(response.into_inner()),
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => Err(DshApiError::NotAuthorized),
        StatusCode::NOT_FOUND => Err(DshApiError::NotFound),
        other_status_code => Err(DshApiError::Unexpected(format!("unexpected status code {} from delete request", other_status_code))),
      },
      Err(rest_api_error) => Err(DshApiError::from(rest_api_error)),
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
