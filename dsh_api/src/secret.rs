//! # Manage secrets
//!
//! Module that contains functions to manage secrets.
//!
//! * [`create_secret(secret) -> ()`](DshApiClient::create_secret)
//! * [`delete_secret(secret_id) -> ()`](DshApiClient::delete_secret)
//! * [`get_secret(secret_id) -> ByteStream`](DshApiClient::get_secret)
//! * [`get_secret_actual(secret_id) -> Empty`](DshApiClient::get_secret_actual)
//! * [`get_secret_configuration(secret_id) -> Empty`](DshApiClient::get_secret_configuration)
//! * [`get_secret_status(secret_id) -> AllocationStatus`](DshApiClient::get_secret_status)
//! * [`get_secrets() -> Vec<String>`](DshApiClient::get_secrets)
//! * [`update_secret(secret_id) -> ()`](DshApiClient::update_secret)

use progenitor_client::ByteStream;

use crate::types::{AllocationStatus, Empty, Secret};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

/// # Manage secrets
///
/// Module that contains functions to manage secrets.
///
/// * [`create_secret(secret) -> ()`](DshApiClient::create_secret)
/// * [`delete_secret(secret_id) -> ()`](DshApiClient::delete_secret)
/// * [`get_secret(secret_id) -> ByteStream`](DshApiClient::get_secret)
/// * [`get_secret_actual(secret_id) -> Empty`](DshApiClient::get_secret_actual)
/// * [`get_secret_configuration(secret_id) -> Empty`](DshApiClient::get_secret_configuration)
/// * [`get_secret_status(secret_id) -> AllocationStatus`](DshApiClient::get_secret_status)
/// * [`get_secrets() -> Vec<String>`](DshApiClient::get_secrets)
/// * [`update_secret(secret_id) -> ()`](DshApiClient::update_secret)
impl DshApiClient<'_> {
  /// # Create secret
  ///
  /// `POST /allocation/{tenant}/secret`
  ///
  /// ## Parameters
  /// * `secret` - secret to be created, consisting of a key/value pair
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the secret has been successfully created)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_secret(&self, secret: Secret) -> DshApiResult<()> {
    self
      .process(self.generated_client.secret_post_by_tenant_secret(self.tenant_name(), self.token(), &secret).await)
      .map(|result| result.1)
  }

  /// # Delete secret
  ///
  /// `DELETE /allocation/{tenant}/secret/{id}/configuration`
  ///
  /// ## Parameters
  /// * `secret_id` - id of the secret to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the secret has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_secret(&self, secret_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .secret_delete_by_tenant_secret_by_id_configuration(self.tenant_name(), secret_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return secret
  ///
  /// `GET /allocation/{tenant}/secret/{id}`
  ///
  /// ## Parameters
  /// * `secret_id` - id of the requested secret
  ///
  /// ## Returns
  /// * `Ok<`[`ByteStream`]`>` - secret
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_secret(&self, secret_id: &str) -> DshApiResult<ByteStream> {
    self
      .process_raw(
        self
          .generated_client
          .secret_get_by_tenant_secret_by_id(self.tenant_name(), secret_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return actual state of secret
  ///
  /// `GET /allocation/{tenant}/secret/{id}/actual`
  ///
  /// ## Parameters
  /// * `secret_id` - id of the requested secret
  ///
  /// ## Returns
  /// * `Ok<`[`Empty`]`>` - indicates that secret is ok, but the actual return value will be empty
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_secret_actual(&self, secret_id: &str) -> DshApiResult<Empty> {
    self
      .process(
        self
          .generated_client
          .secret_get_by_tenant_secret_by_id_actual(self.tenant_name(), secret_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return configuration of secret
  ///
  /// `GET /allocation/{tenant}/secret/{id}/configuration`
  ///
  /// ## Parameters
  /// * `secret_id` - id of the requested secret
  ///
  /// ## Returns
  /// * `Ok<`[`Empty`]`>` - indicates that secret is ok, but the actual return value will be empty
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_secret_configuration(&self, secret_id: &str) -> DshApiResult<Empty> {
    self
      .process(
        self
          .generated_client
          .secret_get_by_tenant_secret_by_id_configuration(self.tenant_name(), secret_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return secret allocation status
  ///
  /// `GET /allocation/{tenant}/secret/{id}/status`
  ///
  /// ## Parameters
  /// * `secret_id` - id of the requested secret
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - allocation status of the secret
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_secret_status(&self, secret_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .secret_get_by_tenant_secret_by_id_status(self.tenant_name(), secret_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return list of secret names
  ///
  /// `GET /allocation/{tenant}/secret`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - list of secret names
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_secrets(&self) -> DshApiResult<Vec<String>> {
    self
      .process(self.generated_client.secret_get_by_tenant_secret(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|secret_ids| secret_ids.iter().map(|secret_id| secret_id.to_string()).collect())
  }

  /// # Update secret value
  ///
  /// `PUT /allocation/{tenant}/secret/{id}`
  ///
  /// ## Parameters
  /// * `secret_id` - id of the secret to update
  /// * `secret` - new value of the secret
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the secret has been successfully updated)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn update_secret(&self, secret_id: &str, secret: String) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .secret_put_by_tenant_secret_by_id(self.tenant_name(), secret_id, self.token(), secret)
          .await,
      )
      .map(|result| result.1)
  }
}
