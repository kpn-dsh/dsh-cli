use dsh_api::query_processor::Part;
use dsh_api::query_processor::Part::{Matching, NonMatching};
use dsh_api::types::Notification;

pub(crate) mod allocation_status;
pub(crate) mod app;
pub(crate) mod application;
pub(crate) mod bucket;
pub(crate) mod certificate;
pub(crate) mod formatter;
pub(crate) mod list_table;
pub(crate) mod manifest;
pub(crate) mod proxy;
pub(crate) mod setting;
pub(crate) mod show_table;
#[cfg(feature = "stream")]
pub(crate) mod stream;
pub(crate) mod string_table;
pub(crate) mod target;
pub(crate) mod task;
pub(crate) mod topic;
pub(crate) mod usage;
pub(crate) mod used_by;
pub(crate) mod volume;

pub(crate) fn notifications_to_string(notifications: &[Notification]) -> String {
  notifications.iter().map(notification_to_string).collect::<Vec<_>>().join(", ")
}

pub(crate) fn notification_to_string(notification: &Notification) -> String {
  format!(
    "{}, {}, {}",
    if notification.remove { "remove".to_string() } else { "create/update".to_string() },
    notification.message,
    notification
      .args
      .iter()
      .map(|(key, value)| format!("{}:{}", key, value))
      .collect::<Vec<_>>()
      .join(", "),
  )
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum TerminalStyle {
  Normal = 0,
  Bold = 1,
  Dim = 2,
  Italic = 3,
  Underlined = 4,
  // Blinking = 5, doesn't seem to work
  Reverse = 7,
  // Invisible = 8, doesn't seem to work
}

#[allow(dead_code)]
pub fn wrap_bold(string: &str) -> String {
  format!("\x1b[1m{}\x1b[0m", string)
}

#[allow(dead_code)]
pub fn wrap_italic(string: &str) -> String {
  format!("\x1b[3m{}\x1b[0m", string)
}

#[allow(dead_code)]
pub fn wrap_underlined(string: &str) -> String {
  format!("\x1b[4m{}\x1b[0m", string)
}

pub fn wrap_style(style: TerminalStyle, string: &str) -> String {
  format!("\x1b[{}m{}\x1b[0m", style as usize, string)
}

pub fn wrap_vec_parts(style: TerminalStyle, parts: &[Part]) -> String {
  parts
    .iter()
    .map(|part| match part {
      Matching(p) => wrap_style(style.clone(), p.as_str()),
      NonMatching(p) => p.to_string(),
    })
    .collect::<Vec<_>>()
    .join("")
}
