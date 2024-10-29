use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum UsageLabel {
  Target,
  App,
  Application,
  Instances,
  Usages,
}

impl Label for UsageLabel {
  fn label_for_list(&self) -> &str {
    match self {
      Self::App => "app",
      Self::Application => "application",
      Self::Instances => "#",
      Self::Target => "target id",
      Self::Usages => "usage",
    }
  }

  fn label_for_show(&self) -> &str {
    match self {
      Self::App => "app",
      Self::Application => "application",
      Self::Instances => "instances",
      Self::Target => "target id",
      Self::Usages => "usage",
    }
  }

  fn is_target_label(&self) -> bool {
    *self == Self::Target
  }
}

pub struct Usage {
  pub target_id: String,
  pub application: Option<String>,
  pub app: Option<String>,
  pub instances: u64,
  pub usages: Vec<String>,
}

impl Usage {
  pub fn app(target_id: String, app: String, instances: u64, usages: Vec<String>) -> Self {
    Self { target_id, application: None, app: Some(app), instances, usages }
  }

  pub fn application(target_id: String, application: String, instances: u64, usages: Vec<String>) -> Self {
    Self { target_id, application: Some(application), app: None, instances, usages }
  }

  pub fn empty(target_id: String) -> Self {
    Self { target_id, application: None, app: None, instances: 0, usages: vec![] }
  }
}

impl SubjectFormatter<UsageLabel> for Usage {
  fn value(&self, label: &UsageLabel, target_id: &str) -> String {
    match label {
      UsageLabel::Target => target_id.to_string(),
      UsageLabel::App => self.app.clone().unwrap_or_default(),
      UsageLabel::Application => self.application.clone().unwrap_or_default(),
      UsageLabel::Instances => self.instances.to_string(),
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

pub static USAGE_LABELS_LIST: [UsageLabel; 5] = [UsageLabel::Target, UsageLabel::Application, UsageLabel::App, UsageLabel::Instances, UsageLabel::Usages];

pub static USAGE_IN_APPS_LABELS_LIST: [UsageLabel; 4] = [UsageLabel::Target, UsageLabel::App, UsageLabel::Instances, UsageLabel::Usages];

pub static USAGE_IN_APPLICATIONS_LABELS_LIST: [UsageLabel; 4] = [UsageLabel::Target, UsageLabel::Application, UsageLabel::Instances, UsageLabel::Usages];

pub static USAGE_LABELS_SHOW: [UsageLabel; 2] = [UsageLabel::Application, UsageLabel::Usages];

pub static USAGE_IN_APPS_LABELS_SHOW: [UsageLabel; 2] = [UsageLabel::Application, UsageLabel::Usages];

pub static USAGE_IN_APPLICATIONS_LABELS_SHOW: [UsageLabel; 2] = [UsageLabel::Application, UsageLabel::Usages];
