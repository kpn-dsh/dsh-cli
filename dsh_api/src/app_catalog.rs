//! # Manage apps in the App Catalog
//!
//! Module that contains functions to manage pre-packaged,
//! easily configured apps that you can select from the App Catalog.
//!
//! * [`get_app_actual_configuration(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app_actual_configuration)
//! * [`get_app_actual_configurations() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_app_actual_configurations)
//! * [`get_app_configuration(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app_configuration)
//! * [`get_app_configurations() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_app_configurations)
//! * [`get_app_ids() -> Vec<String>`](DshApiClient::get_app_ids)

use crate::dsh_api_client::DshApiClient;
use std::collections::HashMap;

use crate::types::AppCatalogApp;
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage apps in the App Catalog
///
/// Module that contains functions to manage pre-packaged,
/// easily configured apps that you can select from the App Catalog.
///
/// * [`get_app_actual_configuration(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app_actual_configuration)
/// * [`get_app_actual_configurations() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_app_actual_configurations)
/// * [`get_app_configuration(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app_configuration)
/// * [`get_app_configurations() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_app_configurations)
/// * [`get_app_ids() -> Vec<String>`](DshApiClient::get_app_ids)
impl DshApiClient<'_> {
  /// # Return actual configuration of deployed App
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/actual`
  ///
  /// ## Parameters
  /// * `app_id` - app id of the requested configuration
  ///
  /// ## Returns
  /// * `Ok<`[`AppCatalogApp`]`>` - app configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_actual_configuration(&self, app_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(self.tenant_name(), app_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get all actual configurations of deployed Apps
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/actual`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`AppCatalogApp`]`>>` - hashmap containing the app configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_actual_configurations(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_actual(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return App configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// * `app_id` - app id of the requested configuration
  ///
  /// ## Returns
  /// * `Ok<`[`AppCatalogApp`]`>` - app configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_configuration(&self, app_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant_name(), app_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all App configurations
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/configuration`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`AppCatalogApp`]`>>` - hashmap containing the app configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_configurations(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_configuration(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all app ids
  ///
  /// If you also need the app configuration, use
  /// [`get_app_configurations()`](Self::get_app_configurations) instead.
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing the sorted app ids
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_ids(&self) -> DshApiResult<Vec<String>> {
    let mut app_ids: Vec<String> = self.get_app_configurations().await?.keys().map(|app_id| app_id.to_string()).collect();
    app_ids.sort();
    Ok(app_ids)
  }
}
