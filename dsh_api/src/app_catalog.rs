use crate::types::{AllocationStatus, AppCatalogAppConfiguration, AppCatalogManifest};
use crate::{DshApiClient, DshApiResult};

/// delete_app_catalog_app(app_catalog_id) -> ()
/// deploy_app_catalog_app(app_catalog_id, body) -> ()
/// get_app_catalog_app(app_catalog_id) -> AppCatalogAppConfiguration
/// get_app_catalog_app_status(app_catalog_id) -> AllocationStatus
/// get_app_catalog_manifests() -> Vec<AppCatalogManifest>

impl DshApiClient<'_> {
  /// `DELETE /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn delete_app_catalog_app(&self, app_catalog_id: &str) -> DshApiResult<()> {
    let target_client = self.target_client_factory.client().await?;
    self.process_delete(
      target_client
        .client()
        .app_catalog_app_configuration_delete_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(target_client.tenant(), app_catalog_id, target_client.token())
        .await,
    )
  }

  /// `PUT /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn deploy_app_catalog_app(&self, app_catalog_id: &str, body: &AppCatalogAppConfiguration) -> DshApiResult<()> {
    let target_client = self.target_client_factory.client().await?;
    self.process_put(
      target_client
        .client()
        .app_catalog_app_configuration_put_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(target_client.tenant(), app_catalog_id, target_client.token(), body)
        .await,
    )
  }

  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  pub async fn get_app_catalog_app(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogAppConfiguration> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(target_client.tenant(), app_catalog_id, target_client.token())
        .await,
    )
  }

  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/status`
  pub async fn get_app_catalog_app_status(&self, app_catalog_id: &str) -> DshApiResult<AllocationStatus> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_status(target_client.tenant(), app_catalog_id, target_client.token())
        .await,
    )
  }

  /// `GET /appcatalog/{tenant}/manifest`
  pub async fn get_app_catalog_manifests(&self) -> DshApiResult<Vec<AppCatalogManifest>> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .app_catalog_manifest_get_appcatalog_by_tenant_manifest(target_client.tenant(), target_client.token())
        .await,
    )
  }
}
