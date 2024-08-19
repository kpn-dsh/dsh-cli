//! # Manage buckets
//!
//! Module that contains functions to manage buckets.
//!
//! * [`create_bucket(bucket_id, bucket)`](DshApiClient::create_bucket)
//! * [`delete_bucket(bucket_id)`](DshApiClient::delete_bucket)
//! * [`get_bucket(bucket_id) -> BucketStatus`](DshApiClient::get_bucket)
//! * [`get_bucket_actual(bucket_id) -> Bucket`](DshApiClient::get_bucket_actual)
//! * [`get_bucket_allocation_status(bucket_id) -> AllocationStatus`](DshApiClient::get_bucket_allocation_status)
//! * [`get_bucket_configuration(bucket_id) -> Bucket`](DshApiClient::get_bucket_configuration)
//! * [`get_bucket_ids(&self) -> Vec<String>`](DshApiClient::get_bucket_ids)

use crate::types::{AllocationStatus, Bucket, BucketStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

/// # Manage buckets
///
/// Module that contains functions to manage buckets.
///
/// * [`create_bucket(bucket_id, bucket)`](DshApiClient::create_bucket)
/// * [`delete_bucket(bucket_id)`](DshApiClient::delete_bucket)
/// * [`get_bucket(bucket_id) -> BucketStatus`](DshApiClient::get_bucket)
/// * [`get_bucket_actual(bucket_id) -> Bucket`](DshApiClient::get_bucket_actual)
/// * [`get_bucket_allocation_status(bucket_id) -> AllocationStatus`](DshApiClient::get_bucket_allocation_status)
/// * [`get_bucket_configuration(bucket_id) -> Bucket`](DshApiClient::get_bucket_configuration)
/// * [`get_bucket_ids(&self) -> Vec<String>`](DshApiClient::get_bucket_ids)
impl DshApiClient<'_> {
  /// # Create bucket
  ///
  /// `PUT /allocation/{tenant}/bucket/{id}/configuration`
  ///
  /// ## Parameters
  /// * `bucket_id` - id of the bucket to update
  /// * `bucket` - new value of the bucket
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the bucket has been successfully updated)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_bucket(&self, bucket_id: &str, bucket: Bucket) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .bucket_put_by_tenant_bucket_by_id_configuration(self.tenant_name(), bucket_id, self.token(), &bucket)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete bucket
  ///
  /// `DELETE /allocation/{tenant}/bucket/{id}/configuration`
  ///
  /// ## Parameters
  /// * `bucket_id` - id of the bucket to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the bucket has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_bucket(&self, bucket_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .bucket_delete_by_tenant_bucket_by_id_configuration(self.tenant_name(), bucket_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return bucket
  ///
  /// `GET /allocation/{tenant}/bucket/{id}`
  ///
  /// This method combines the results of the methods
  /// [`get_bucket_actual()`](DshApiClient::get_bucket_actual),
  /// [`get_bucket_allocation_status()`](DshApiClient::get_bucket_allocation_status) and
  /// [`get_bucket_configuration()`](DshApiClient::get_bucket_configuration)
  /// into one method call.
  ///
  /// ## Parameters
  /// * `bucket_id` - id of the requested bucket
  ///
  /// ## Returns
  /// * `Ok<`[`BucketStatus`]`>` - bucket
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_bucket(&self, bucket_id: &str) -> DshApiResult<BucketStatus> {
    self
      .process_raw(
        self
          .generated_client
          .bucket_get_by_tenant_bucket_by_id(self.tenant_name(), bucket_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return actual state of bucket
  ///
  /// `GET /allocation/{tenant}/bucket/{id}/actual`
  ///
  /// ## Parameters
  /// * `bucket_id` - id of the requested bucket
  ///
  /// ## Returns
  /// * `Ok<`[`Bucket`]`>` - indicates that bucket is ok
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_bucket_actual(&self, bucket_id: &str) -> DshApiResult<Bucket> {
    self
      .process(
        self
          .generated_client
          .bucket_get_by_tenant_bucket_by_id_actual(self.tenant_name(), bucket_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return bucket allocation status
  ///
  /// `GET /allocation/{tenant}/bucket/{id}/status`
  ///
  /// ## Parameters
  /// * `bucket_id` - id of the requested bucket
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - allocation status of the bucket
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_bucket_allocation_status(&self, bucket_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .bucket_get_by_tenant_bucket_by_id_status(self.tenant_name(), bucket_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return bucket configuration
  ///
  /// `GET /allocation/{tenant}/bucket/{id}/configuration`
  ///
  /// ## Parameters
  /// * `bucket_id` - id of the requested bucket
  ///
  /// ## Returns
  /// * `Ok<`[`Bucket`]`>` - indicates that bucket is ok
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_bucket_configuration(&self, bucket_id: &str) -> DshApiResult<Bucket> {
    self
      .process(
        self
          .generated_client
          .bucket_get_by_tenant_bucket_by_id_configuration(self.tenant_name(), bucket_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return bucket ids
  ///
  /// `GET /allocation/{tenant}/bucket`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>` - bucket ids
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_bucket_ids(&self) -> DshApiResult<Vec<String>> {
    let mut bucket_ids: Vec<String> = self
      .process(self.generated_client.bucket_get_by_tenant_bucket(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|bucket_ids| bucket_ids.iter().map(|bucket_id| bucket_id.to_string()).collect())?;
    bucket_ids.sort();
    Ok(bucket_ids)
  }
}
