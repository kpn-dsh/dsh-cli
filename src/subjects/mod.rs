pub(crate) mod api;
pub(crate) mod app;
pub(crate) mod bucket;
pub(crate) mod certificate;
pub(crate) mod env;
pub(crate) mod image;
pub(crate) mod manifest;
pub(crate) mod metric;
pub(crate) mod nodepool;
pub(crate) mod platform;
pub(crate) mod proxy;
pub(crate) mod robot;
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

use crate::formatters::Value;
use crate::formatters::{Label, SubjectFormatter};
use dsh_api::types::AllocationStatus;
use dsh_api::{Dependant, DependantApp, DependantApplication, DependantCertificate, DependantProxy, DependantTrifonius};
use itertools::Itertools;
use serde::Serialize;
use std::fmt::Display;

#[derive(Eq, Hash, PartialEq, Serialize)]
enum AllocationStatusLabel {
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
  fn value(&self, label: &AllocationStatusLabel, target_id: &str) -> Value {
    match label {
      AllocationStatusLabel::DerivedFrom => Value::option(self.derived_from.as_ref()),
      AllocationStatusLabel::Notifications => Value::warn(self.notifications.iter().map(|notification| notification.to_string()).join("\n")),
      AllocationStatusLabel::Provisioned => Value::plain(self.provisioned),
      AllocationStatusLabel::Target => Value::target(target_id),
    }
  }
}

static _ALLOCATION_STATUS_LABELS: [AllocationStatusLabel; 4] =
  [AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::Notifications, AllocationStatusLabel::DerivedFrom];

static DEFAULT_ALLOCATION_STATUS_LABELS: [AllocationStatusLabel; 4] =
  [AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::Notifications, AllocationStatusLabel::DerivedFrom];

#[derive(Eq, Hash, PartialEq, Serialize)]
enum DependantLabel {
  Dependencies,
  DependantId,
  Injections,
  Instances,
  DependantKind,
  Resources,
  Target,
}

impl Label for DependantLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::DependantId => "id",
      Self::DependantKind => "app/service",
      Self::Dependencies => "dependencies",
      Self::Injections => "injections",
      Self::Instances => "instances",
      Self::Resources => "app/resources",
      Self::Target => "target id",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::DependantId => "dependant",
      Self::DependantKind => "kind",
      Self::Dependencies => "dependencies",
      Self::Injections => "injections",
      Self::Instances => "#",
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
  fn value(&self, label: &DependantLabel, target_id: &str) -> Value {
    match label {
      DependantLabel::DependantId => Value::target(&self.application_id),
      DependantLabel::DependantKind => Value::plain("service"),
      DependantLabel::Dependencies => Value::plain(self.injections.iter().map(|injection| injection.to_string()).join("\n")),
      DependantLabel::Injections => Value::plain(self.injections.iter().map(|injection| injection.to_string()).join("\n")),
      DependantLabel::Instances => Value::plain(self.instances),
      DependantLabel::Resources => Value::empty(),
      DependantLabel::Target => Value::target(target_id),
    }
  }
}

impl SubjectFormatter<DependantLabel> for DependantApp {
  fn value(&self, label: &DependantLabel, target_id: &str) -> Value {
    match label {
      DependantLabel::DependantId => Value::target(&self.app_id),
      DependantLabel::DependantKind => Value::plain("app"),
      DependantLabel::Dependencies => Value::plain(self.resources.iter().map(|resource| resource.to_string()).join("\n")),
      DependantLabel::Resources => Value::plain(self.resources.iter().map(|resource| resource.to_string()).join("\n")),
      DependantLabel::Target => Value::target(target_id),
      _ => Value::empty(),
    }
  }
}

impl SubjectFormatter<DependantLabel> for DependantCertificate {
  fn value(&self, label: &DependantLabel, target_id: &str) -> Value {
    match label {
      DependantLabel::DependantId => Value::target(&self.certificate_id),
      DependantLabel::DependantKind => Value::plain("certificate"),
      DependantLabel::Dependencies => Value::plain(self.secret_kind.to_string()),
      DependantLabel::Target => Value::target(target_id),
      _ => Value::empty(),
    }
  }
}

impl SubjectFormatter<DependantLabel> for DependantProxy {
  fn value(&self, label: &DependantLabel, target_id: &str) -> Value {
    match label {
      DependantLabel::DependantId => Value::target(&self.proxy_id),
      DependantLabel::DependantKind => Value::plain("proxy"),
      DependantLabel::Instances => Value::plain(self.instances),
      DependantLabel::Target => Value::target(target_id),
      _ => Value::empty(),
    }
  }
}

impl<T> SubjectFormatter<DependantLabel> for DependantTrifonius<T>
where
  T: Display,
{
  fn value(&self, label: &DependantLabel, target_id: &str) -> Value {
    match label {
      DependantLabel::DependantId => Value::target(&self.trifonius_id),
      DependantLabel::DependantKind => Value::plain("trifonius"),
      DependantLabel::Dependencies => Value::plain(self.injections.iter().map(|injection| injection.to_string()).join("\n")),
      DependantLabel::Injections => Value::plain(self.injections.iter().map(|injection| injection.to_string()).join("\n")),
      DependantLabel::Instances => Value::plain(self.instances),
      DependantLabel::Resources => Value::empty(),
      DependantLabel::Target => Value::target(target_id),
    }
  }
}

impl<T> SubjectFormatter<DependantLabel> for Dependant<T>
where
  T: Display,
{
  fn value(&self, label: &DependantLabel, target_id: &str) -> Value {
    match self {
      Dependant::App { app } => app.value(label, target_id),
      Dependant::Application { application } => application.value(label, target_id),
      Dependant::Certificate { certificate } => certificate.value(label, target_id),
      Dependant::Proxy { proxy } => proxy.value(label, target_id),
      Dependant::Trifonius { trifonius } => trifonius.value(label, target_id),
    }
  }
}

static DEPENDANT_LABELS_LIST: [DependantLabel; 5] =
  [DependantLabel::Target, DependantLabel::DependantKind, DependantLabel::DependantId, DependantLabel::Instances, DependantLabel::Dependencies];
static _DEPENDANT_LABELS_SERVICES_LIST: [DependantLabel; 4] = [DependantLabel::Target, DependantLabel::DependantId, DependantLabel::Instances, DependantLabel::Injections];
static _DEPENDANT_LABELS_APPS_LIST: [DependantLabel; 3] = [DependantLabel::Target, DependantLabel::DependantId, DependantLabel::Resources];

static DEPENDANT_LABELS: [DependantLabel; 4] = [DependantLabel::DependantId, DependantLabel::DependantKind, DependantLabel::Instances, DependantLabel::Dependencies];
static DEPENDANT_LABELS_SERVICES: [DependantLabel; 3] = [DependantLabel::DependantId, DependantLabel::Instances, DependantLabel::Injections];
static DEPENDANT_LABELS_APPS: [DependantLabel; 3] = [DependantLabel::DependantId, DependantLabel::Dependencies, DependantLabel::Resources];
