use trifonius_dsh_api::types::Notification;

pub(crate) mod allocation_status;
pub(crate) mod app;
pub(crate) mod application;
pub(crate) mod bucket;
pub(crate) mod certificate;
pub(crate) mod formatter;
pub(crate) mod proxy;
pub(crate) mod secret;
pub(crate) mod stream;
pub(crate) mod topic;
pub(crate) mod usage;
pub(crate) mod volume;

pub(crate) fn notifications_to_string(notifications: &[Notification]) -> String {
  notifications.iter().map(notification_to_string).collect::<Vec<String>>().join(", ")
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
      .collect::<Vec<String>>()
      .join(", "),
  )
}
