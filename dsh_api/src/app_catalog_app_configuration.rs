//! # Manage the App Catalog
//!
//! Module that contains functions to configure apps you start from the App Catalog.
//!
//! * [`create_app_catalog_app(app_catalog_id, body) -> ()`](DshApiClient::create_app_catalog_app)
//! * [`delete_app_catalog_app(app_catalog_id) -> ()`](DshApiClient::delete_app_catalog_app)
//! * [`get_app_catalog_app_allocation_status(app_catalog_id) -> AllocationStatus`](DshApiClient::get_app_catalog_app_allocation_status)
//! * [`get_app_catalog_app_configuration(app_catalog_id) -> AppCatalogAppConfiguration`](DshApiClient::get_app_catalog_app_configuration)

use crate::dsh_api_client::DshApiClient;
use crate::types::{AllocationStatus, AppCatalogAppConfiguration};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage the App Catalog
///
/// Module that contains functions to configure apps you start from the App Catalog.
///
/// * [`create_app_catalog_app(app_catalog_id, body) -> ()`](DshApiClient::create_app_catalog_app)
/// * [`delete_app_catalog_app(app_catalog_id) -> ()`](DshApiClient::delete_app_catalog_app)
/// * [`get_app_catalog_app_allocation_status(app_catalog_id) -> AllocationStatus`](DshApiClient::get_app_catalog_app_allocation_status)
/// * [`get_app_catalog_app_configuration(app_catalog_id) -> AppCatalogAppConfiguration`](DshApiClient::get_app_catalog_app_configuration)
impl DshApiClient<'_> {
  /// # Create or update a new App Catalog App
  ///
  /// `PUT /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// * `app_catalog_id` - id of the app that must be deleted
  /// * `configuration` - configuration of the app that must created or updated
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the app has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_app_catalog_app(&self, app_catalog_id: &str, body: &AppCatalogAppConfiguration) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .app_catalog_app_configuration_put_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant_name(), app_catalog_id, self.token(), body)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete an App Catalog App
  ///
  /// `DELETE /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// * `app_catalog_id` - id of the app that must be deleted
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the app has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_app_catalog_app(&self, app_catalog_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .app_catalog_app_configuration_delete_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant_name(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get an App Catalog App status
  ///
  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/status`
  ///
  /// ## Parameters
  /// * `app_catalog_id` - id of the requested app
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - app status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_catalog_app_allocation_status(&self, app_catalog_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_status(self.tenant_name(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get an App Catalog App configuration
  ///
  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// * `app_catalog_id` - id of the requested app
  ///
  /// ## Returns
  /// * `Ok<`[`AppCatalogAppConfiguration`]`>` - app configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_catalog_app_configuration(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogAppConfiguration> {
    self
      .process(
        self
          .generated_client
          .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant_name(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
