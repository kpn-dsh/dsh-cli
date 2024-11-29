use crate::formatters::formatter::{Label, SubjectFormatter};
use dsh_api::UsedBy;

#[derive(Eq, Hash, PartialEq)]
pub enum UsedByLabel {
  Target,
  Instances,
  Injections,
}

impl Label for UsedByLabel {
  fn label_for_list(&self) -> &str {
    match self {
      UsedByLabel::Target => "app/application",
      UsedByLabel::Instances => "#",
      UsedByLabel::Injections => "injections",
    }
  }

  fn label_for_show(&self) -> &str {
    match self {
      UsedByLabel::Target => "app/application",
      UsedByLabel::Instances => "instances",
      UsedByLabel::Injections => "injections",
    }
  }

  fn is_target_label(&self) -> bool {
    *self == Self::Target
  }
}

impl SubjectFormatter<UsedByLabel> for UsedBy {
  fn value(&self, label: &UsedByLabel, _: &str) -> String {
    match label {
      UsedByLabel::Target => match self {
        UsedBy::App(app_id, application_id, _, _) => format!("app: {}, application: {}", app_id, application_id),
        UsedBy::Application(application_id, _, _) => format!("application: {}", application_id),
      },
      UsedByLabel::Instances => match self {
        UsedBy::App(_, _, instances, _) => instances.to_string(),
        UsedBy::Application(_, instances, _) => instances.to_string(),
      },
      UsedByLabel::Injections => match self {
        UsedBy::App(_, _, _, injections) => injections.iter().map(|inj| inj.to_string()).collect::<Vec<_>>().join(", "),
        UsedBy::Application(_, _, injections) => injections.iter().map(|inj| inj.to_string()).collect::<Vec<_>>().join(", "),
      },
    }
  }

  fn target_id(&self) -> Option<String> {
    None
  }

  fn target_label(&self) -> Option<UsedByLabel> {
    None
  }
}

pub static USED_BY_LABELS_LIST: [UsedByLabel; 3] = [UsedByLabel::Target, UsedByLabel::Instances, UsedByLabel::Injections];
