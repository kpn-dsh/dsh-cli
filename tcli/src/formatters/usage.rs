use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum UsageLabel {
  Target,
  App,
  Application,
  Usages,
}

impl Label for UsageLabel {
  fn label_show(&self) -> &str {
    match self {
      Self::App => "app",
      Self::Application => "application",
      Self::Target => "target id",
      Self::Usages => "usage",
    }
  }
}

pub struct Usage {
  pub target_id: String,
  pub application: Option<String>,
  pub app: Option<String>,
  pub usages: Vec<String>,
}

impl Usage {
  pub fn app(target_id: String, app: String, usages: Vec<String>) -> Self {
    Self { target_id, application: None, app: Some(app), usages }
  }

  pub fn application(target_id: String, application: String, usages: Vec<String>) -> Self {
    Self { target_id, application: Some(application), app: None, usages }
  }

  pub fn empty(target_id: String) -> Self {
    Self { target_id, application: None, app: None, usages: vec![] }
  }
}

impl SubjectFormatter<UsageLabel> for Usage {
  fn value(&self, column: &UsageLabel, target_id: &str) -> String {
    match column {
      UsageLabel::Target => target_id.to_string(),
      UsageLabel::App => self.app.clone().unwrap_or_default(),
      UsageLabel::Application => self.application.clone().unwrap_or_default(),
      UsageLabel::Usages => {
        if self.usages.is_empty() {
          "-".to_string()
        } else {
          self.usages.clone().join("\n")
        }
      }
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(self.target_id.clone())
  }

  fn target_label(&self) -> Option<UsageLabel> {
    Some(UsageLabel::Target)
  }
}

pub static USAGE_LABELS_LIST: [UsageLabel; 4] = [UsageLabel::Target, UsageLabel::Application, UsageLabel::App, UsageLabel::Usages];

pub static USAGE_LABELS_SHOW: [UsageLabel; 2] = [UsageLabel::Application, UsageLabel::Usages];
