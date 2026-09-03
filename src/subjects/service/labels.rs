use crate::formatters::Value;
use crate::formatters::{hashmap_to_table, ColumnAlignment, Label, SubjectFormatter};
use dsh_api::parse::ImageString;
use dsh_api::types::{Application, ApplicationSecret};
use dsh_api::vhost::VhostString;
use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum ServiceLabel {
  Cpus,
  Env,
  ExposedPorts,
  HealthCheck,
  Image,
  Instances,
  Mem,
  Metrics,
  NeedsToken,
  NodepoolFeatures,
  ReadableStreams,
  Secrets,
  SingleInstance,
  SpreadGroup,
  Target,
  Topics,
  User,
  Volumes,
  WritableStreams,
}

impl Label for ServiceLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Target => "service id",
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "exposed ports",
      Self::HealthCheck => "health check",
      Self::Image => "image",
      Self::Instances => "instances",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "needs token",
      Self::NodepoolFeatures => "nodepool features",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single instance",
      Self::SpreadGroup => "spread group",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "ports",
      Self::HealthCheck => "health",
      Self::Image => "image",
      Self::Instances => "#",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "token",
      Self::NodepoolFeatures => "nodepool",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single",
      Self::SpreadGroup => "spread group",
      Self::Target => "service id",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }

  fn column_alignment(&self) -> ColumnAlignment {
    match self {
      Self::Mem => ColumnAlignment::Right,
      _ => ColumnAlignment::default(),
    }
  }
}

impl SubjectFormatter<ServiceLabel> for Application {
  fn value(&self, label: &ServiceLabel, target_id: &str) -> Value {
    match label {
      ServiceLabel::Cpus => Value::plain(self.cpus),
      ServiceLabel::Env => Value::plain(hashmap_to_table(&self.env)),
      ServiceLabel::ExposedPorts => {
        if self.exposed_ports.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            // TODO Format as table
            self
              .exposed_ports
              .iter()
              .map(|(port, port_mapping)| {
                format!(
                  "{} : {}",
                  port,
                  VhostString::try_from(port_mapping).map(|vhost_string| vhost_string.to_string()).unwrap_or_default()
                )
              })
              .collect_vec()
              .join("\n"),
          )
        }
      }
      ServiceLabel::HealthCheck => Value::some_or_hide(self.health_check.clone()),
      ServiceLabel::Image => Value::plain(ImageString::from(self.image.as_str())),
      ServiceLabel::Instances => Value::plain(self.instances),
      ServiceLabel::Mem => Value::plain(self.mem),
      ServiceLabel::Metrics => Value::some_or_hide(self.metrics.clone().map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))),
      ServiceLabel::NeedsToken => Value::plain(self.needs_token),
      ServiceLabel::NodepoolFeatures => Value::some_or_hide(self.node_features.clone()),
      ServiceLabel::ReadableStreams => {
        if self.readable_streams.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            self
              .readable_streams
              .clone()
              .into_iter()
              .map(|readable_stream| readable_stream.to_string())
              .collect_vec()
              .join("\n"),
          )
        }
      }
      ServiceLabel::Secrets => {
        if self.secrets.is_empty() {
          Value::hide()
        } else {
          Value::plain(secrets_to_table(&self.secrets))
        }
      }
      ServiceLabel::SingleInstance => Value::plain(self.single_instance),
      ServiceLabel::SpreadGroup => Value::some_or_hide(self.spread_group.clone()),
      ServiceLabel::Target => Value::target(target_id),
      ServiceLabel::Topics => {
        if self.topics.is_empty() {
          Value::hide()
        } else {
          Value::plain(self.topics.clone().into_iter().map(|topic| topic.to_string()).collect_vec().join("\n"))
        }
      }
      ServiceLabel::User => Value::plain(&self.user),
      ServiceLabel::Volumes => {
        if self.volumes.is_empty() {
          Value::hide()
        } else {
          Value::plain(self.volumes.keys().map(|key| key.to_string()).collect_vec().join("\n"))
        }
      }
      ServiceLabel::WritableStreams => {
        if self.writable_streams.is_empty() {
          Value::hide()
        } else {
          Value::plain(
            self
              .writable_streams
              .clone()
              .into_iter()
              .map(|writable_stream| writable_stream.to_string())
              .collect_vec()
              .join("\n"),
          )
        }
      }
    }
  }
}

fn secrets_to_table(secrets: &[ApplicationSecret]) -> String {
  let m: HashMap<String, String> = secrets
    .iter()
    .map(|application_secret| {
      (
        application_secret.name.clone(),
        application_secret
          .injections
          .iter()
          .map(|injection| injection.get("env").map(|s| s.to_string()).unwrap_or("".to_string()))
          .join(", "),
      )
    })
    .collect::<HashMap<_, _>>();
  hashmap_to_table(&m)
}
