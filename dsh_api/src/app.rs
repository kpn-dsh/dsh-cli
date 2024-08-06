use std::collections::HashMap;

use crate::types::AppCatalogApp;
use crate::DshApiClient;
use crate::DshApiResult;

/// get_app(app_catalog_id) -> AppCatalogApp
/// get_apps() -> HashMap<AppCatalogId, AppCatalogApp>
/// get_deployed_app(app_catalog_id) -> AppCatalogApp
/// get_deployed_apps() -> HashMap<AppCatalogId, AppCatalogApp>
impl DshApiClient<'_> {
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn get_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token())
        .await,
    )
  }

  /// `GET /allocation/{tenant}/appcatalogapp/configuration`
  pub async fn get_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_get_by_tenant_appcatalogapp_configuration(self.tenant(), self.token())
        .await,
    )
  }

  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/actual`
  pub async fn get_deployed_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(self.tenant(), app_catalog_id, self.token())
        .await,
    )
  }
  /// `GET /allocation/{tenant}/appcatalogapp/actual`
  pub async fn get_deployed_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_get_by_tenant_appcatalogapp_actual(self.tenant(), self.token())
        .await,
    )
  }
}
