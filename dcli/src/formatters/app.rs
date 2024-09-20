use crate::formatters::formatter::{Label, SubjectFormatter};
use dsh_api::types::AppCatalogApp;
use serde_json::de::from_str;
use std::collections::HashMap;

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
  fn value(&self, label: &AppCatalogAppLabel, target_id: &str) -> String {
    match label {
      AppCatalogAppLabel::Configuration => match &self.configuration {
        Some(configuration) => match from_str::<HashMap<String, String>>(configuration) {
          Ok(map) => {
            let mut keys: Vec<String> = map
              .keys()
              .filter_map(|key| if !key.starts_with("@") { Some(key.to_string()) } else { None })
              .collect();
            keys.sort();
            keys
              .iter()
              .map(|key| format!("{} : {}", key, map.get(key).map(|v| v.to_string()).unwrap_or("".to_string())))
              .collect::<Vec<String>>()
              .join("\n")
          }
          Err(_) => "error".to_string(),
        },
        None => "empty".to_string(),
      },
      AppCatalogAppLabel::ManifestUrl => self.manifest_urn.clone(),
      AppCatalogAppLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<AppCatalogAppLabel> {
    Some(AppCatalogAppLabel::Target)
  }
}

pub static APP_CATALOG_APP_LABELS: [AppCatalogAppLabel; 3] = [AppCatalogAppLabel::Target, AppCatalogAppLabel::ManifestUrl, AppCatalogAppLabel::Configuration];
