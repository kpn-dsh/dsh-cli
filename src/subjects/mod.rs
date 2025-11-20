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
use dsh_api::{Dependant, DependantApp, DependantApplication};
use itertools::Itertools;
use serde::Serialize;
use std::fmt::Display;

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
pub enum DependantLabel {
  Dependencies,
  Id,
  Injections,
  Instances,
  Kind,
  Resources,
  Target,
}

impl Label for DependantLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Dependencies => "dependencies",
      Self::Id => "id",
      Self::Injections => "injections",
      Self::Instances => "instances",
      Self::Kind => "app/service",
      Self::Resources => "app/resources",
      Self::Target => "target id",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::Dependencies => "dependencies",
      Self::Id => "id",
      Self::Injections => "injections",
      Self::Instances => "#",
      Self::Kind => "app/service",
      Self::Resources => "resources",
      Self::Target => "target id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl<T> SubjectFormatter<DependantLabel> for DependantApplication<T>
where
  T: Display,
{
  fn value(&self, label: &DependantLabel, target_id: &str) -> String {
    match label {
      DependantLabel::Dependencies => self.injections.iter().map(|injection| injection.to_string()).collect_vec().join("\n"),
      DependantLabel::Id => self.application_id.to_string(),
      DependantLabel::Injections => self.injections.iter().map(|injection| injection.to_string()).collect_vec().join("\n"),
      DependantLabel::Instances => self.instances.to_string(),
      DependantLabel::Kind => "service".to_string(),
      DependantLabel::Resources => "".to_string(),
      DependantLabel::Target => target_id.to_string(),
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(self.application_id.to_string())
  }
}

impl SubjectFormatter<DependantLabel> for DependantApp {
  fn value(&self, label: &DependantLabel, target_id: &str) -> String {
    match label {
      DependantLabel::Dependencies => self.resources.iter().map(|resource| resource.to_string()).collect_vec().join("\n"),
      DependantLabel::Id => self.app_id.to_string(),
      DependantLabel::Injections => "".to_string(),
      DependantLabel::Instances => "".to_string(),
      DependantLabel::Kind => "app".to_string(),
      DependantLabel::Resources => self.resources.iter().map(|resource| resource.to_string()).collect_vec().join("\n"),
      DependantLabel::Target => target_id.to_string(),
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(self.app_id.to_string())
  }
}

impl<T> SubjectFormatter<DependantLabel> for Dependant<T>
where
  T: Display,
{
  fn value(&self, label: &DependantLabel, target_id: &str) -> String {
    match self {
      Dependant::App(app) => app.value(label, target_id),
      Dependant::Application(application) => application.value(label, target_id),
    }
  }

  fn target_id(&self) -> Option<String> {
    match self {
      Dependant::App(app) => Some(app.app_id.to_string()),
      Dependant::Application(application) => Some(application.application_id.to_string()),
    }
  }
}

pub static DEPENDANT_LABELS_LIST: [DependantLabel; 4] = [DependantLabel::Target, DependantLabel::Kind, DependantLabel::Id, DependantLabel::Dependencies];
pub static _DEPENDANT_LABELS_SERVICES_LIST: [DependantLabel; 4] = [DependantLabel::Target, DependantLabel::Id, DependantLabel::Instances, DependantLabel::Injections];
pub static _DEPENDANT_LABELS_APPS_LIST: [DependantLabel; 3] = [DependantLabel::Target, DependantLabel::Id, DependantLabel::Resources];

pub static DEPENDANT_LABELS: [DependantLabel; 3] = [DependantLabel::Kind, DependantLabel::Id, DependantLabel::Dependencies];
pub static DEPENDANT_LABELS_SERVICES: [DependantLabel; 3] = [DependantLabel::Id, DependantLabel::Instances, DependantLabel::Injections];
pub static DEPENDANT_LABELS_APPS: [DependantLabel; 3] = [DependantLabel::Id, DependantLabel::Dependencies, DependantLabel::Resources];
