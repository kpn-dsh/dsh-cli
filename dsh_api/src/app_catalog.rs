//! # Manage apps in the App Catalog
//!
//! Module that contains functions to manage pre-packaged,
//! easily configured apps that you can select from the App Catalog.
//!
//! * [`get_app(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app)
//! * [`get_app_actual(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app_actual)
//! * [`get_apps() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_apps)
//! * [`get_apps_actual() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_apps_actual)

use std::collections::HashMap;

use crate::types::AppCatalogApp;
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

/// # Manage apps in the App Catalog
///
/// Module that contains functions to manage pre-packaged,
/// easily configured apps that you can select from the App Catalog.
///
/// * [`get_app(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app)
/// * [`get_app_actual(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app_actual)
/// * [`get_apps() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_apps)
/// * [`get_apps_actual() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_apps_actual)
impl DshApiClient<'_> {
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
  pub async fn get_app(&self, app_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant_name(), app_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return configuration of deployed App
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/actual`
  ///
  /// ## Parameters
  /// * `app_id` - app id of the requested configuration
  ///
  /// ## Returns
  /// * `Ok<`[`AppCatalogApp`]`>` - app configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_actual(&self, app_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(self.tenant_name(), app_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all Apps with their configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/configuration`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`AppCatalogApp`]`>>` - hashmap containing the app configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_configuration(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get all deployed Apps with their configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/actual`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`AppCatalogApp`]`>>` - hashmap containing the app configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_apps_actual(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_actual(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
