//! # View the App Catalog manifests
//!
//! Module that contains a function to query the App Catalog for all manifest files.
//!
//! * [`get_app_catalog_manifests() -> Vec<AppCatalogManifest>`](DshApiClient::get_app_catalog_manifests)

use crate::types::AppCatalogManifest;
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

/// # View the App Catalog manifests
///
/// Module that contains a function to query the App Catalog for all manifest files.
///
/// * [`get_app_catalog_manifests() -> Vec<AppCatalogManifest>`](DshApiClient::get_app_catalog_manifests)
impl DshApiClient<'_> {
  /// # Return a list of all App Catalog manifests
  ///
  /// `GET /appcatalog/{tenant}/manifest`
  ///
  /// ## Returns
  /// * `Ok<Vec`[`AppCatalogManifest`]`>` - vector containing all app manifests
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_catalog_manifests(&self) -> DshApiResult<Vec<AppCatalogManifest>> {
    self
      .process(
        self
          .generated_client
          .app_catalog_manifest_get_appcatalog_by_tenant_manifest(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
