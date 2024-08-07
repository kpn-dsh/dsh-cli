use crate::types::AppCatalogManifest;
use crate::DshApiClient;
use crate::DshApiResult;

/// # app catalog manifest
///
/// Query the App Catalog.
///
/// * `get_app_catalog_manifests() -> Vec<AppCatalogManifest>`
impl DshApiClient<'_> {
  /// # Returns a list of all App Catalog manifests
  ///
  /// `GET /appcatalog/{tenant}/manifest`
  pub async fn get_app_catalog_manifests(&self) -> DshApiResult<Vec<AppCatalogManifest>> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_manifest_get_appcatalog_by_tenant_manifest(self.tenant(), self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
