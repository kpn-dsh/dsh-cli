use std::collections::HashMap;

use crate::types::AppCatalogApp;
use crate::DshApiClient;
use crate::DshApiResult;

/// # app catalog
///
/// Manage pre-packaged, easily configured apps that you can select from the App Catalog.
///
/// * `get_app(app_catalog_id) -> AppCatalogApp`
/// * `get_apps() -> HashMap<AppCatalogId, AppCatalogApp>`
/// * `get_deployed_app(app_catalog_id) -> AppCatalogApp`
/// * `get_deployed_apps() -> HashMap<AppCatalogId, AppCatalogApp>`
impl DshApiClient<'_> {
  /// # Get an App Catalog App's configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// `app_catalog_id` - app id of the requested configuration
  pub async fn get_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get all App Catalog App configurations
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/configuration`
  pub async fn get_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_get_by_tenant_appcatalogapp_configuration(self.tenant(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get a deployed App Catalog App's configuration
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/actual`
  ///
  /// ## Parameters
  /// `app_catalog_id` - app catalog app id of the requested configuration
  pub async fn get_deployed_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(self.tenant(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get all deployed App Catalog App configurations
  ///
  /// `GET /allocation/{tenant}/appcatalogapp/actual`
  pub async fn get_deployed_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_get_by_tenant_appcatalogapp_actual(self.tenant(), self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
