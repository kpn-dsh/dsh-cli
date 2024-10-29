//! # View the App Catalog manifests
//!
//! Module that contains a function to query the App Catalog for all manifest files.
//!
//! * [`get_app_catalog_manifest_ids() -> Vec<String>`](DshApiClient::get_app_catalog_manifest_ids)
//! * [`get_app_catalog_manifest_ids_with_versions() -> Vec<(String, Vec<String>)>`](DshApiClient::get_app_catalog_manifest_ids_with_versions)
//! * [`get_app_catalog_manifests() -> Vec<AppCatalogManifest>`](DshApiClient::get_app_catalog_manifests)
use std::collections::{HashMap, HashSet};

use crate::dsh_api_client::DshApiClient;
use serde_json::{from_str, Value};

use crate::types::AppCatalogManifest;
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

pub const API_VERSION: &str = "apiVersion";
pub const CONFIGURATION: &str = "configuration";
pub const CONTACT: &str = "contact";
pub const DESCRIPTION: &str = "description";
pub const ID: &str = "id";
pub const KIND: &str = "kind";
pub const MORE_INFO: &str = "moreInfo";
pub const NAME: &str = "name";
pub const RESOURCES: &str = "resources";
pub const VENDOR: &str = "vendor";
pub const VERSION: &str = "version";

/// # View the App Catalog manifests
///
/// Module that contains a function to query the App Catalog for all manifest files.
///
/// * [`get_app_catalog_manifest_ids() -> Vec<String>`](DshApiClient::get_app_catalog_manifest_ids)
/// * [`get_app_catalog_manifest_ids_with_versions() -> Vec<(String, Vec<String>)>`](DshApiClient::get_app_catalog_manifest_ids_with_versions)
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

  /// # Return sorted list of all App Catalog manifest ids
  ///
  /// `GET /appcatalog/{tenant}/manifest`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing all manifest ids
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_catalog_manifest_ids(&self) -> DshApiResult<Vec<String>> {
    let (_, manifests) = self.process(
      self
        .generated_client
        .app_catalog_manifest_get_appcatalog_by_tenant_manifest(self.tenant_name(), self.token())
        .await,
    )?;
    let mut unique_ids: HashSet<String> = HashSet::new();
    for manifest in manifests {
      let payload = from_str::<Value>(manifest.payload.as_str())?;
      let payload_object = payload.as_object().unwrap();
      if let Some(id) = payload_object.get(ID).and_then(|id| id.as_str().map(|id| id.to_string())) {
        unique_ids.insert(id);
      }
    }
    let mut ids: Vec<String> = unique_ids.iter().map(|id| id.to_string()).collect();
    ids.sort();
    Ok(ids)
  }

  /// # Return list of all App Catalog manifest ids with versions
  ///
  /// `GET /appcatalog/{tenant}/manifest`
  ///
  /// ## Returns
  /// * `Ok<Vec<(String, Vec<String>)>>` - vector containing pairs of ids and versions
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_app_catalog_manifest_ids_with_versions(&self) -> DshApiResult<Vec<(String, Vec<String>)>> {
    let (_, manifests) = self.process(
      self
        .generated_client
        .app_catalog_manifest_get_appcatalog_by_tenant_manifest(self.tenant_name(), self.token())
        .await,
    )?;
    let mut id_versions: HashMap<String, Vec<String>> = HashMap::new();
    for manifest in manifests {
      let payload = from_str::<Value>(manifest.payload.as_str())?;
      let payload_object = payload.as_object().unwrap();
      if let Some(id) = payload_object.get(ID).and_then(|id| id.as_str().map(|id| id.to_string())) {
        if let Some(version) = payload_object.get(VERSION).and_then(|version| version.as_str().map(|version| version.to_string())) {
          id_versions.entry(id).and_modify(|versions| versions.push(version.clone())).or_insert(vec![version]);
        }
      }
    }
    let mut id_versions_pairs: Vec<(String, Vec<String>)> = id_versions.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    id_versions_pairs.sort_by_key(|(id, _)| id.clone());
    for (_, versions) in id_versions_pairs.iter_mut() {
      versions.sort(); // TODO Sort as semver instead of lexicographically
    }
    Ok(id_versions_pairs)
  }
}
