use std::collections::HashMap;

use crate::{DshApiClient, DshApiResult};

use crate::types::AppCatalogApp;

/// get_app(app_catalog_id) -> AppCatalogApp
/// get_apps() -> HashMap<AppCatalogId, AppCatalogApp>
/// get_deployed_app(app_catalog_id) -> AppCatalogApp
/// get_deployed_apps() -> HashMap<AppCatalogId, AppCatalogApp>

impl DshApiClient<'_> {
  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn get_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(target_client.tenant(), app_catalog_id, target_client.token())
        .await,
    )
  }

  /// `GET /allocation/{tenant}/appcatalogapp/configuration`
  pub async fn get_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_get_by_tenant_appcatalogapp_configuration(target_client.tenant(), target_client.token())
        .await,
    )
  }

  /// `GET /allocation/{tenant}/appcatalogapp/{appcatalogappid}/actual`
  pub async fn get_deployed_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogApp> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(target_client.tenant(), app_catalog_id, target_client.token())
        .await,
    )
  }
  /// `GET /allocation/{tenant}/appcatalogapp/actual`
  pub async fn get_deployed_apps(&self) -> DshApiResult<HashMap<String, AppCatalogApp>> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_get_by_tenant_appcatalogapp_actual(target_client.tenant(), target_client.token())
        .await,
    )
  }
}
