use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::settings::Settings;

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum SettingsLabel {
  DefaultPlatform,
  DefaultTenant,
  ShowExecutionTime,
  NoBorder,
  Verbosity,
}

impl Label for SettingsLabel {
  fn label_for_show(&self) -> &str {
    match self {
      SettingsLabel::DefaultPlatform => "default-platform",
      SettingsLabel::DefaultTenant => "default-tenant",
      SettingsLabel::ShowExecutionTime => "show-execution-time",
      SettingsLabel::NoBorder => "no-border",
      SettingsLabel::Verbosity => "verbosity",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

impl SubjectFormatter<SettingsLabel> for Settings {
  fn value(&self, label: &SettingsLabel, _target_id: &str) -> String {
    match label {
      SettingsLabel::DefaultPlatform => self
        .default_platform
        .clone()
        .unwrap_or("not set, provide platform as a command line argument, else the user will be promted".to_string()),
      SettingsLabel::DefaultTenant => self
        .default_tenant
        .clone()
        .unwrap_or("not set, provide platform as a command line argument, else the user will be promted".to_string()),
      SettingsLabel::ShowExecutionTime => self
        .show_execution_time
        .map(|show_execution_time| show_execution_time.to_string())
        .unwrap_or("not set, default depends on 'verbosity' setting".to_string()),
      SettingsLabel::NoBorder => self
        .no_border
        .map(|no_border| no_border.to_string())
        .unwrap_or("not set, default is to show borders".to_string()),
      SettingsLabel::Verbosity => self
        .verbosity
        .clone()
        .map(|verbosity| verbosity.to_string())
        .unwrap_or("not set, defaults to 'low'".to_string()),
    }
  }

  fn target_label(&self) -> Option<SettingsLabel> {
    None
  }
}

pub static SETTING_LABELS: [SettingsLabel; 5] =
  [SettingsLabel::DefaultPlatform, SettingsLabel::DefaultTenant, SettingsLabel::ShowExecutionTime, SettingsLabel::NoBorder, SettingsLabel::Verbosity];
