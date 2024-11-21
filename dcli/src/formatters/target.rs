use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum TargetFormatterLabel {
  Default,
  GroupUserId,
  Platform,
  Tenant,
}

impl Label for TargetFormatterLabel {
  fn label_for_show(&self) -> &str {
    match self {
      Self::Default => "default",
      Self::GroupUserId => "id",
      Self::Platform => "platform",
      Self::Tenant => "tenant",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

pub struct TargetFormatter {
  pub(crate) platform: String,
  pub(crate) tenant: String,
  pub(crate) group_user_id: u16,
  pub(crate) is_default: bool,
}

impl SubjectFormatter<TargetFormatterLabel> for TargetFormatter {
  fn value(&self, label: &TargetFormatterLabel, _target_id: &str) -> String {
    match label {
      TargetFormatterLabel::Default => {
        if self.is_default {
          "*".to_string()
        } else {
          "".to_string()
        }
      }
      TargetFormatterLabel::GroupUserId => format!("{}:{}", self.group_user_id, self.group_user_id),
      TargetFormatterLabel::Platform => self.platform.to_string(),
      TargetFormatterLabel::Tenant => self.tenant.clone(),
    }
  }

  fn target_label(&self) -> Option<TargetFormatterLabel> {
    None
  }
}

pub static TARGET_LABELS: [TargetFormatterLabel; 4] =
  [TargetFormatterLabel::Tenant, TargetFormatterLabel::Platform, TargetFormatterLabel::GroupUserId, TargetFormatterLabel::Default];
