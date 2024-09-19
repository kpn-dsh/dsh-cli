use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum SecretUsageLabel {
  App,
  Application,
  Injections,
  Target,
}

impl Label for SecretUsageLabel {
  fn label_show(&self) -> &str {
    match self {
      Self::App => "app",
      Self::Application => "application",
      Self::Injections => "injection(s)",
      Self::Target => "secret id",
    }
  }
}

pub struct SecretUsage {
  pub target_id: String,
  pub application: Option<String>,
  pub app: Option<String>,
  pub injections: Vec<String>,
}

impl SecretUsage {
  pub fn app(target_id: String, app: String, injections: Vec<String>) -> Self {
    Self { target_id, application: None, app: Some(app), injections }
  }

  pub fn application(target_id: String, application: String, injections: Vec<String>) -> Self {
    Self { target_id, application: Some(application), app: None, injections }
  }

  pub fn empty(target_id: String) -> Self {
    Self { target_id, application: None, app: None, injections: vec![] }
  }
}

impl SubjectFormatter<SecretUsageLabel> for SecretUsage {
  fn value(&self, column: &SecretUsageLabel, target_id: &str) -> String {
    match column {
      SecretUsageLabel::App => self.app.clone().unwrap_or_default(),
      SecretUsageLabel::Application => self.application.clone().unwrap_or_default(),
      SecretUsageLabel::Injections => {
        if self.injections.is_empty() {
          "".to_string()
        } else {
          self.injections.clone().join("\n")
        }
      }
      SecretUsageLabel::Target => target_id.to_string(),
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(self.target_id.clone())
  }

  fn target_label(&self) -> Option<SecretUsageLabel> {
    Some(SecretUsageLabel::Target)
  }
}

pub static SECRET_USAGE_LABELS_LIST: [SecretUsageLabel; 4] = [SecretUsageLabel::Target, SecretUsageLabel::Application, SecretUsageLabel::App, SecretUsageLabel::Injections];

pub static SECRET_USAGE_IN_APPLICATIONS_LABELS_SHOW: [SecretUsageLabel; 2] = [SecretUsageLabel::Application, SecretUsageLabel::Injections];

pub static SECRET_USAGE_IN_APPS_LABELS_SHOW: [SecretUsageLabel; 2] = [SecretUsageLabel::Application, SecretUsageLabel::Injections];
