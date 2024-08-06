use crate::types::{AllocationStatus, AppCatalogAppConfiguration, AppCatalogManifest};
use crate::DshApiClient;
use crate::DshApiResult;

/// delete_app_catalog_app(app_catalog_id) -> ()
/// deploy_app_catalog_app(app_catalog_id, body) -> ()
/// get_app_catalog_app(app_catalog_id) -> `AppCatalogAppConfiguration`
/// get_app_catalog_app_status(app_catalog_id) -> `AllocationStatus`
/// get_app_catalog_manifests() -> `Vec<AppCatalogManifest>`
impl DshApiClient<'_> {
  /// `DELETE /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn delete_app_catalog_app(&self, app_catalog_id: &str) -> DshApiResult<()> {
    self.process_delete(
      self
        .generated_client()
        .app_catalog_app_configuration_delete_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token())
        .await,
    )
  }

  /// `PUT /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn deploy_app_catalog_app(&self, app_catalog_id: &str, body: &AppCatalogAppConfiguration) -> DshApiResult<()> {
    self.process_put(
      self
        .generated_client()
        .app_catalog_app_configuration_put_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token(), body)
        .await,
    )
  }

  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn get_app_catalog_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogAppConfiguration> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token())
        .await,
    )
  }

  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/status`
  pub async fn get_app_catalog_app_status(&self, app_catalog_id: &str) -> DshApiResult<AllocationStatus> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_status(self.tenant(), app_catalog_id, self.token())
        .await,
    )
  }

  /// `GET /appcatalog/{tenant}/manifest`
  pub async fn get_app_catalog_manifests(&self) -> DshApiResult<Vec<AppCatalogManifest>> {
    self.process_get(
      self
        .generated_client()
        .app_catalog_manifest_get_appcatalog_by_tenant_manifest(self.tenant(), self.token())
        .await,
    )
  }
}
