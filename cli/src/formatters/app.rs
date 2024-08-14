use trifonius_dsh_api::types::{AppCatalogApp, AppCatalogAppResourcesValue, Application};

/// ## Returns
/// * resource_id
/// * application
pub(crate) fn get_application_from_app(app: &AppCatalogApp) -> Option<(&String, &Application)> {
  app.resources.iter().find_map(|(resource_id, resource)| match resource {
    AppCatalogAppResourcesValue::Application(application) => Some((resource_id, application)),
    _ => None,
  })
}
