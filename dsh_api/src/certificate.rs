//! # Manage certificates
//!
//! Module that contains functions to manage certificates.
//!
//! * [`create_certificate(certificate_id, certificate)`](DshApiClient::create_certificate)
//! * [`delete_certificate(certificate_id)`](DshApiClient::delete_certificate)
//! * [`get_certificate(certificate_id) -> certificateStatus`](DshApiClient::get_certificate)
//! * [`get_certificate_actual_configuration(certificate_id) -> certificate`](DshApiClient::get_certificate_actual_configuration)
//! * [`get_certificate_allocation_status(certificate_id) -> AllocationStatus`](DshApiClient::get_certificate_allocation_status)
//! * [`get_certificate_configuration(certificate_id) -> certificate`](DshApiClient::get_certificate_configuration)
//! * [`get_certificate_ids(&self) -> Vec<String>`](DshApiClient::get_certificate_ids)

use crate::dsh_api_client::DshApiClient;
use crate::types::{AllocationStatus, Certificate, CertificateStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage certificates
///
/// Module that contains functions to manage certificates.
///
/// * [`create_certificate(certificate_id, certificate)`](DshApiClient::create_certificate)
/// * [`delete_certificate(certificate_id)`](DshApiClient::delete_certificate)
/// * [`get_certificate(certificate_id) -> certificateStatus`](DshApiClient::get_certificate)
/// * [`get_certificate_actual_configuration(certificate_id) -> certificate`](DshApiClient::get_certificate_actual_configuration)
/// * [`get_certificate_allocation_status(certificate_id) -> AllocationStatus`](DshApiClient::get_certificate_allocation_status)
/// * [`get_certificate_configuration(certificate_id) -> certificate`](DshApiClient::get_certificate_configuration)
/// * [`get_certificate_ids(&self) -> Vec<String>`](DshApiClient::get_certificate_ids)
impl DshApiClient<'_> {
  /// # Create certificate
  ///
  /// `PUT /allocation/{tenant}/certificate/{id}/configuration`
  ///
  /// ## Parameters
  /// * `certificate_id` - id of the certificate to update
  /// * `certificate` - new value of the certificate
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the certificate has been successfully updated)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_certificate(&self, certificate_id: &str, certificate: Certificate) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .certificate_put_by_tenant_certificate_by_id_configuration(self.tenant_name(), certificate_id, self.token(), &certificate)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete certificate
  ///
  /// `DELETE /allocation/{tenant}/certificate/{id}/configuration`
  ///
  /// ## Parameters
  /// * `certificate_id` - id of the certificate to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the certificate has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_certificate(&self, certificate_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .certificate_delete_by_tenant_certificate_by_id_configuration(self.tenant_name(), certificate_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return certificate
  ///
  /// `GET /allocation/{tenant}/certificate/{id}`
  ///
  /// This method combines the results of the methods
  /// [`get_certificate_actual()`](DshApiClient::get_certificate_actual_configuration),
  /// [`get_certificate_allocation_status()`](DshApiClient::get_certificate_allocation_status) and
  /// [`get_certificate_configuration()`](DshApiClient::get_certificate_configuration)
  /// into one method call.
  ///
  /// ## Parameters
  /// * `certificate_id` - id of the requested certificate
  ///
  /// ## Returns
  /// * `Ok<`[`CertificateStatus`]`>` - certificate
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_certificate(&self, certificate_id: &str) -> DshApiResult<CertificateStatus> {
    self
      .process_raw(
        self
          .generated_client
          .certificate_get_by_tenant_certificate_by_id(self.tenant_name(), certificate_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return actual state of certificate
  ///
  /// `GET /allocation/{tenant}/certificate/{id}/actual`
  ///
  /// ## Parameters
  /// * `certificate_id` - id of the requested certificate
  ///
  /// ## Returns
  /// * `Ok<`[`Certificate`]`>` - indicates that certificate is ok
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_certificate_actual_configuration(&self, certificate_id: &str) -> DshApiResult<Certificate> {
    self
      .process(
        self
          .generated_client
          .certificate_get_by_tenant_certificate_by_id_actual(self.tenant_name(), certificate_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return certificate allocation status
  ///
  /// `GET /allocation/{tenant}/certificate/{id}/status`
  ///
  /// ## Parameters
  /// * `certificate_id` - id of the requested certificate
  ///
  /// ## Returns
  /// * `Ok<`[`CertificateStatus`]`>` - allocation status of the certificate
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_certificate_allocation_status(&self, certificate_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .certificate_get_by_tenant_certificate_by_id_status(self.tenant_name(), certificate_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return certificate configuration
  ///
  /// `GET /allocation/{tenant}/certificate/{id}/configuration`
  ///
  /// ## Parameters
  /// * `certificate_id` - id of the requested certificate
  ///
  /// ## Returns
  /// * `Ok<`[`Certificate`]`>` - indicates that certificate is ok
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_certificate_configuration(&self, certificate_id: &str) -> DshApiResult<Certificate> {
    self
      .process(
        self
          .generated_client
          .certificate_get_by_tenant_certificate_by_id_configuration(self.tenant_name(), certificate_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return certificate ids
  ///
  /// `GET /allocation/{tenant}/certificate`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>` - certificate ids
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_certificate_ids(&self) -> DshApiResult<Vec<String>> {
    let mut certificate_ids: Vec<String> = self
      .process(self.generated_client.certificate_get_by_tenant_certificate(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|certificate_ids| certificate_ids.iter().map(|certificate_id| certificate_id.to_string()).collect())?;
    certificate_ids.sort();
    Ok(certificate_ids)
  }
}
