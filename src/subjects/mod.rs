pub(crate) mod api;
pub(crate) mod app;
pub(crate) mod bucket;
pub(crate) mod certificate;
pub(crate) mod env;
pub(crate) mod image;
pub(crate) mod manifest;
pub(crate) mod metric;
pub(crate) mod platform;
pub(crate) mod proxy;
pub(crate) mod secret;
pub(crate) mod service;
pub(crate) mod setting;
#[cfg(feature = "manage")]
pub(crate) mod stream;
pub(crate) mod target;
#[cfg(feature = "manage")]
pub(crate) mod tenant;
pub(crate) mod token;
pub(crate) mod topic;
pub(crate) mod vhost;
pub(crate) mod volume;

use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::notifications_to_string;
use dsh_api::types::AllocationStatus;
use dsh_api::UsedBy;
use serde::Serialize;

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum AllocationStatusLabel {
  DerivedFrom,
  Notifications,
  Provisioned,
  Target,
}

impl Label for AllocationStatusLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => "target id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<AllocationStatusLabel> for AllocationStatus {
  fn value(&self, label: &AllocationStatusLabel, target_id: &str) -> String {
    match label {
      AllocationStatusLabel::DerivedFrom => self.clone().derived_from.unwrap_or_default(),
      AllocationStatusLabel::Notifications => {
        if self.notifications.is_empty() {
          "none".to_string()
        } else {
          notifications_to_string(&self.notifications)
        }
      }
      AllocationStatusLabel::Provisioned => self.provisioned.to_string(),
      AllocationStatusLabel::Target => target_id.to_string(),
    }
  }
}

pub static _ALLOCATION_STATUS_LABELS: [AllocationStatusLabel; 4] =
  [AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::Notifications, AllocationStatusLabel::DerivedFrom];

pub static DEFAULT_ALLOCATION_STATUS_LABELS: [AllocationStatusLabel; 4] =
  [AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::Notifications, AllocationStatusLabel::DerivedFrom];

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum UsedByLabel {
  Target,
  User,
  Instances,
  Injections,
}

impl Label for UsedByLabel {
  fn as_str(&self) -> &str {
    match self {
      UsedByLabel::Injections => "injections",
      UsedByLabel::Instances => "instances",
      UsedByLabel::Target => "target id",
      UsedByLabel::User => "app/service",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      UsedByLabel::Injections => "injections",
      UsedByLabel::Instances => "#",
      UsedByLabel::Target => "target id",
      UsedByLabel::User => "app/service",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<UsedByLabel> for UsedBy {
  fn value(&self, label: &UsedByLabel, target_id: &str) -> String {
    match label {
      UsedByLabel::Injections => match self {
        UsedBy::App(_, resources) => resources.iter().map(|resource| resource.to_string()).collect::<Vec<_>>().join("\n"),
        UsedBy::Application(_, _, injections) => injections.iter().map(|injection| injection.to_string()).collect::<Vec<_>>().join("\n"),
      },
      UsedByLabel::Instances => match self {
        UsedBy::App(_, _) => "".to_string(),
        UsedBy::Application(_, instances, _) => instances.to_string(),
      },
      UsedByLabel::Target => target_id.to_string(),
      UsedByLabel::User => match self {
        UsedBy::App(app_id, _) => app_id.to_string(),
        UsedBy::Application(service_id, _, _) => service_id.to_string(),
      },
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(match self {
      UsedBy::App(app_id, _) => app_id.to_string(),
      UsedBy::Application(service_id, _, _) => service_id.to_string(),
    })
  }
}

pub static USED_BY_LABELS_LIST: [UsedByLabel; 4] = [UsedByLabel::Target, UsedByLabel::Instances, UsedByLabel::User, UsedByLabel::Injections];

pub static USED_BY_LABELS: [UsedByLabel; 3] = [UsedByLabel::User, UsedByLabel::Instances, UsedByLabel::Injections];
