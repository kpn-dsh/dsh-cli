use trifonius_dsh_api::types::AppCatalogApp;

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum AppCatalogAppLabel {
  Configuration,
  ManifestUrl,
  Target,
}

impl Label for AppCatalogAppLabel {
  fn label_show(&self) -> &str {
    match self {
      AppCatalogAppLabel::Configuration => "app configuration",
      AppCatalogAppLabel::ManifestUrl => "manifest url",
      AppCatalogAppLabel::Target => "app",
    }
  }
}

impl SubjectFormatter<AppCatalogAppLabel> for AppCatalogApp {
  fn value(&self, column: &AppCatalogAppLabel, target_id: &str) -> String {
    match column {
      AppCatalogAppLabel::Configuration => self.configuration.clone().unwrap_or_default().to_string(),
      AppCatalogAppLabel::ManifestUrl => self.manifest_urn.clone(),
      AppCatalogAppLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<AppCatalogAppLabel> {
    Some(AppCatalogAppLabel::Target)
  }
}

pub static APP_CATALOG_APP_LABELS: [AppCatalogAppLabel; 3] = [AppCatalogAppLabel::Target, AppCatalogAppLabel::ManifestUrl, AppCatalogAppLabel::Configuration];
