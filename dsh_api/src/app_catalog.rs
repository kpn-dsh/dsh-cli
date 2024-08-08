//! # Manage apps in the App Catalog
//!
//! Module that contains functions to manage pre-packaged,
//! easily configured apps that you can select from the App Catalog.
//!
//! * [`get_app(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_app)
//! * [`get_apps() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_apps)
//! * [`get_deployed_app(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_deployed_app)
//! * [`get_deployed_apps() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_deployed_apps)

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
/// * [`get_apps() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_apps)
/// * [`get_deployed_app(app_catalog_id) -> AppCatalogApp`](DshApiClient::get_deployed_app)
/// * [`get_deployed_apps() -> HashMap<String, AppCatalogApp>`](DshApiClient::get_deployed_apps)
impl DshApiClient<'_> {
  /// # Get an App Catalog App's configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// * `app_catalog_id` - app id of the requested configuration
  ///
  /// ## Returns
  /// * `Ok<`[`AppCatalogApp`]`>` - app configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant_name(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get all App Catalog App configurations
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

  /// # Get a deployed App Catalog App's configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/actual`
  ///
  /// ## Parameters
  /// * `app_catalog_id` - app id of the requested configuration
  ///
  /// ## Returns
  /// * `Ok<`[`AppCatalogApp`]`>` - app configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_deployed_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(self.tenant_name(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get all deployed App Catalog App configurations
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/actual`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`AppCatalogApp`]`>>` - hashmap containing the app configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_deployed_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
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
