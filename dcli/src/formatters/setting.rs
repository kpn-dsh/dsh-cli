use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::settings::Settings;

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum SettingLabel {
  DefaultPlatform,
  DefaultTenant,
  ShowExecutionTime,
  NoBorder,
  Verbosity,
}

impl Label for SettingLabel {
  fn label_for_show(&self) -> &str {
    match self {
      SettingLabel::DefaultPlatform => "default-platform",
      SettingLabel::DefaultTenant => "default-tenant",
      SettingLabel::ShowExecutionTime => "show-execution-time",
      SettingLabel::NoBorder => "no-border",
      SettingLabel::Verbosity => "verbosity",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

impl SubjectFormatter<SettingLabel> for Settings {
  fn value(&self, label: &SettingLabel, _target_id: &str) -> String {
    match label {
      SettingLabel::DefaultPlatform => self
        .default_platform
        .clone()
        .unwrap_or("not set, provide platform as a command line argument, else the user will be promted".to_string()),
      SettingLabel::DefaultTenant => self
        .default_tenant
        .clone()
        .unwrap_or("not set, provide platform as a command line argument, else the user will be promted".to_string()),
      SettingLabel::ShowExecutionTime => self
        .show_execution_time
        .map(|show_execution_time| show_execution_time.to_string())
        .unwrap_or("not set, default depends on 'verbosity' setting".to_string()),
      SettingLabel::NoBorder => self
        .no_border
        .map(|no_border| no_border.to_string())
        .unwrap_or("not set, default is to show borders".to_string()),
      SettingLabel::Verbosity => self
        .verbosity
        .clone()
        .map(|verbosity| verbosity.to_string())
        .unwrap_or("not set, defaults to 'low'".to_string()),
    }
  }

  fn target_label(&self) -> Option<SettingLabel> {
    None
  }
}

pub static SETTING_LABELS: [SettingLabel; 5] =
  [SettingLabel::DefaultPlatform, SettingLabel::DefaultTenant, SettingLabel::ShowExecutionTime, SettingLabel::NoBorder, SettingLabel::Verbosity];
