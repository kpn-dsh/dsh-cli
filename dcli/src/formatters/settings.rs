use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::settings::Target;

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum TargetLabel {
  Platform,
  Tenant,
  GroupUserId,
}

impl Label for TargetLabel {
  fn label_for_show(&self) -> &str {
    match self {
      Self::GroupUserId => "id",
      Self::Platform => "platform",
      Self::Tenant => "tenant",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

impl SubjectFormatter<TargetLabel> for Target {
  fn value(&self, label: &TargetLabel, _target_id: &str) -> String {
    match label {
      TargetLabel::GroupUserId => self.group_user_id.clone(),
      TargetLabel::Platform => self.platform.to_string(),
      TargetLabel::Tenant => self.tenant.clone()
    }
  }

  fn target_label(&self) -> Option<TargetLabel> {
    None
  }
}

pub static TARGET_LABELS: [TargetLabel; 3] = [TargetLabel::Platform, TargetLabel::Tenant, TargetLabel::GroupUserId];
