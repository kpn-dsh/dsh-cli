use crate::types::{AllocationStatus, AppCatalogAppConfiguration};
use crate::DshApiClient;
use crate::DshApiResult;

/// # app catalog app configuration
///
/// Configure apps you start from the App Catalog.
///
/// * `delete_app_catalog_app(app_catalog_id) -> ()`
/// * `deploy_app_catalog_app(app_catalog_id, body) -> ()`
/// * `get_app_catalog_app_configuration(app_catalog_id) -> AppCatalogAppConfiguration`
/// * `get_app_catalog_app_status(app_catalog_id) -> AllocationStatus`
impl DshApiClient<'_> {
  /// # Delete an App Catalog App
  ///
  /// `DELETE /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// `app_catalog_id` - id of the app catalog app
  pub async fn delete_app_catalog_app(&self, app_catalog_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_app_configuration_delete_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Create or update a new App Catalog App
  ///
  /// `PUT /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// `app_catalog_id` - id of the app catalog app
  /// `body`           - configuration of the app catalog app
  pub async fn deploy_app_catalog_app(&self, app_catalog_id: &str, body: &AppCatalogAppConfiguration) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_app_configuration_put_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token(), body)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get an App Catalog App configuration
  ///
  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/configuration`
  ///
  /// ## Parameters
  /// `app_catalog_id` - app catalog app id of the requested configuration
  pub async fn get_app_catalog_app_configuration(&self, app_catalog_id: &str) -> DshApiResult<AppCatalogAppConfiguration> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(self.tenant(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Get an App Catalog App status
  ///
  /// `GET /appcatalog/{tenant}/appcatalogapp/{appcatalogappid}/status`
  ///
  /// ## Parameters
  /// `app_catalog_id` - app catalog app id of the requested status
  pub async fn get_app_catalog_app_status(&self, app_catalog_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client()
          .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_status(self.tenant(), app_catalog_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
