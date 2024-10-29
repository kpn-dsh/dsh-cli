//! # Manage volumes
//!
//! Module that contains functions to manage volumes.
//!
//! * [`create_volume(volume_id, configuration) -> ()`](DshApiClient::create_volume)
//! * [`delete_volume(volume_id) -> ()`](DshApiClient::delete_volume)
//! * [`get_volume(volume_id) -> volumeStatus`](DshApiClient::get_volume)
//! * [`get_volume_allocation_status(volume_id) -> AllocationStatus`](DshApiClient::get_volume_allocation_status)
//! * [`get_volume_configuration(volume_id) -> volume`](DshApiClient::get_volume_configuration)
//! * [`get_volume_actual_configuration(volume_id) -> volume`](DshApiClient::get_volume_actual_configuration)
//! * [`get_volume_ids() -> Vec<String>`](DshApiClient::get_volume_ids)

use crate::dsh_api_client::DshApiClient;
use crate::types::{AllocationStatus, Volume, VolumeStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage volumes
///
/// Module that contains functions to manage volumes.
///
/// * [`create_volume(volume_id, configuration) -> ()`](DshApiClient::create_volume)
/// * [`delete_volume(volume_id) -> ()`](DshApiClient::delete_volume)
/// * [`get_volume(volume_id) -> volumeStatus`](DshApiClient::get_volume)
/// * [`get_volume_allocation_status(volume_id) -> AllocationStatus`](DshApiClient::get_volume_allocation_status)
/// * [`get_volume_configuration(volume_id) -> volume`](DshApiClient::get_volume_configuration)
/// * [`get_volume_actual_configuration(volume_id) -> volume`](DshApiClient::get_volume_actual_configuration)
/// * [`get_volume_ids() -> Vec<String>`](DshApiClient::get_volume_ids)
impl DshApiClient<'_> {
  /// # Create volume
  ///
  /// `PUT /allocation/{tenant}/volume/{id}/configuration`
  ///
  /// ## Parameters
  /// * `volume_id` - name of the created volume
  /// * `configuration` - configuration for the created volume
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the volume has been successfully created)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_volume(&self, volume_id: &str, configuration: &Volume) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .volume_put_by_tenant_volume_by_id_configuration(self.tenant_name(), volume_id, self.token(), configuration)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete volume
  ///
  /// `DELETE /allocation/{tenant}/volume/{id}/configuration`
  ///
  /// ## Parameters
  /// * `volume_id` - name of the volume to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the volume has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_volume(&self, volume_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .volume_delete_by_tenant_volume_by_id_configuration(self.tenant_name(), volume_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return volume
  ///
  /// `GET /allocation/{tenant}/volume/{id}`
  ///
  /// This method combines the results of the methods
  /// [`get_volume_allocation_status()`](DshApiClient::get_volume_allocation_status),
  /// [`get_volume_configuration()`](DshApiClient::get_volume_configuration) and
  /// [`get_volume_configuration_actual()`](DshApiClient::get_volume_actual_configuration)
  /// into one method call.
  ///
  /// ## Parameters
  /// * `volume_id` - name of the requested volume
  ///
  /// ## Returns
  /// * `Ok<`[`volumeStatus`]`>` - volume status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_volume(&self, volume_id: &str) -> DshApiResult<VolumeStatus> {
    self
      .process(
        self
          .generated_client
          .volume_get_by_tenant_volume_by_id(self.tenant_name(), volume_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return volume allocation status
  ///
  /// `GET /allocation/{tenant}/volume/{id}/status`
  ///
  /// ## Parameters
  /// * `volume_id` - name of the requested volume
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - volume allocation status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_volume_allocation_status(&self, volume_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .volume_get_by_tenant_volume_by_id_status(self.tenant_name(), volume_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return volume configuration
  ///
  /// `GET /allocation/{tenant}/volume/{id}/configuration`
  ///
  /// ## Parameters
  /// * `volume_id` - name of the requested volume
  ///
  /// ## Returns
  /// * `Ok<`[`volume`]`>` - volume configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_volume_configuration(&self, volume_id: &str) -> DshApiResult<Volume> {
    self
      .process(
        self
          .generated_client
          .volume_get_by_tenant_volume_by_id_configuration(self.tenant_name(), volume_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return actual volume configuration
  ///
  /// `GET /allocation/{tenant}/volume/{id}/actual`
  ///
  /// ## Parameters
  /// * `volume_id` - name of the requested volume
  ///
  /// ## Returns
  /// * `Ok<`[`volume`]`>` - volume configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_volume_actual_configuration(&self, volume_id: &str) -> DshApiResult<Volume> {
    self
      .process(
        self
          .generated_client
          .volume_get_by_tenant_volume_by_id_actual(self.tenant_name(), volume_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return sorted list of volume names
  ///
  /// `GET /allocation/{tenant}/volume`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - list of volume names
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_volume_ids(&self) -> DshApiResult<Vec<String>> {
    let mut volume_ids: Vec<String> = self
      .process(self.generated_client.volume_get_by_tenant_volume(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|secret_ids| secret_ids.iter().map(|secret_id| secret_id.to_string()).collect())?;
    volume_ids.sort();
    Ok(volume_ids)
  }
}
