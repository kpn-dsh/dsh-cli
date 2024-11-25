use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::settings::Settings;

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum SettingLabel {
  DefaultPlatform,
  DefaultTenant,
  FileName,
  NoBorder,
  ShowExecutionTime,
  Target,
  Verbosity,
}

impl Label for SettingLabel {
  fn label_for_show(&self) -> &str {
    match self {
      Self::DefaultPlatform => "default-platform",
      Self::DefaultTenant => "default-tenant",
      Self::FileName => "settings file name",
      Self::NoBorder => "no-border",
      Self::ShowExecutionTime => "show-execution-time",
      Self::Target => "setting",
      Self::Verbosity => "verbosity",
    }
  }

  fn is_target_label(&self) -> bool {
    match self {
      Self::Target => true,
      _ => false,
    }
  }
}

impl SubjectFormatter<SettingLabel> for Settings {
  fn value(&self, label: &SettingLabel, target_id: &str) -> String {
    match label {
      SettingLabel::DefaultPlatform => self.default_platform.clone().unwrap_or("not set".to_string()),
      SettingLabel::DefaultTenant => self.default_tenant.clone().unwrap_or("not set".to_string()),
      SettingLabel::FileName => self.file_name.clone().unwrap_or_default(),
      SettingLabel::NoBorder => self
        .no_border
        .map(|no_border| no_border.to_string())
        .unwrap_or("not set (defaults to 'false')".to_string()),
      SettingLabel::ShowExecutionTime => self
        .show_execution_time
        .map(|show_execution_time| show_execution_time.to_string())
        .unwrap_or("not set".to_string()),
      SettingLabel::Target => target_id.to_string(),
      SettingLabel::Verbosity => self
        .verbosity
        .clone()
        .map(|verbosity| verbosity.to_string())
        .unwrap_or("not set (defaults to 'low')".to_string()),
    }
  }

  fn target_label(&self) -> Option<SettingLabel> {
    Some(SettingLabel::Target)
  }
}

pub static SETTING_LABELS: [SettingLabel; 6] =
  [SettingLabel::DefaultPlatform, SettingLabel::DefaultTenant, SettingLabel::ShowExecutionTime, SettingLabel::NoBorder, SettingLabel::Verbosity, SettingLabel::FileName];
