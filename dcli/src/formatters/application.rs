use lazy_static::lazy_static;
use regex::Regex;

use dsh_api::types::Application;

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub(crate) enum ApplicationLabel {
  Cpus,
  Env,
  ExposedPorts,
  HealthCheck,
  Image,
  Instances,
  Mem,
  Metrics,
  NeedsToken,
  ReadableStreams,
  Secrets,
  SingleInstance,
  SpreadGroup,
  Target,
  Tasks,
  Topics,
  User,
  Volumes,
  WritableStreams,
}

impl Label for ApplicationLabel {
  fn label_list(&self) -> &str {
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
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single",
      Self::SpreadGroup => "spread group",
      Self::Target => "application",
      Self::Tasks => "tasks",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }

  fn label_show(&self) -> &str {
    match self {
      Self::Target => "application",
      Self::Cpus => "cpus",
      Self::Env => "env",
      Self::ExposedPorts => "exposed ports",
      Self::HealthCheck => "health check",
      Self::Image => "image",
      Self::Instances => "instances",
      Self::Mem => "mem",
      Self::Metrics => "metrics",
      Self::NeedsToken => "needs token",
      Self::ReadableStreams => "readable streams",
      Self::Secrets => "secrets",
      Self::SingleInstance => "single instance",
      Self::SpreadGroup => "spread group",
      Self::Tasks => "tasks",
      Self::Topics => "topics",
      Self::User => "user",
      Self::Volumes => "volumes",
      Self::WritableStreams => "writable streams",
    }
  }
}

impl SubjectFormatter<ApplicationLabel> for Application {
  fn value(&self, label: &ApplicationLabel, application_id: &str) -> String {
    match label {
      ApplicationLabel::Cpus => self.cpus.to_string(),
      ApplicationLabel::Env => {
        let mut key = self.env.keys().map(|k| k.to_string()).collect::<Vec<String>>();
        key.sort();
        key
          .iter()
          .map(|key| format!("{} -> {}", key, self.env.get(key).unwrap()))
          .collect::<Vec<String>>()
          .join("\n")
      }
      ApplicationLabel::ExposedPorts => self.exposed_ports.keys().map(|port| port.to_string()).collect::<Vec<String>>().join(","),
      ApplicationLabel::HealthCheck => match self.health_check {
        Some(ref health_check) => match health_check.protocol {
          Some(protocol) => format!("{}:{}/{}", protocol.to_string(), health_check.port, health_check.path),
          None => format!("{}/{}", health_check.port, health_check.path),
        },
        None => "".to_string(),
      },
      ApplicationLabel::Image => match parse_image_string(self.image.as_str()) {
        Some((kind, image)) => format!("{}:{}", kind, image),
        None => self.image.clone(),
      },
      ApplicationLabel::Instances => self.instances.to_string(),
      ApplicationLabel::Mem => self.mem.to_string(),
      ApplicationLabel::Metrics => self
        .metrics
        .clone()
        .map(|ref metrics| format!("{}:{}", metrics.port, metrics.path))
        .unwrap_or_default(),
      ApplicationLabel::NeedsToken => self.needs_token.to_string(),
      ApplicationLabel::ReadableStreams => self
        .readable_streams
        .clone()
        .into_iter()
        .map(|readable_stream| readable_stream.to_string())
        .collect::<Vec<String>>()
        .join(", "),
      ApplicationLabel::Secrets => self.secrets.clone().into_iter().map(|secret| secret.name).collect::<Vec<String>>().join(", "),
      ApplicationLabel::SingleInstance => self.single_instance.to_string(),
      ApplicationLabel::SpreadGroup => self.spread_group.clone().unwrap_or_default(),
      ApplicationLabel::Target => application_id.to_string(),
      ApplicationLabel::Tasks => "".to_string(),
      ApplicationLabel::Topics => self.topics.clone().into_iter().map(|topic| topic.to_string()).collect::<Vec<String>>().join(", "),
      ApplicationLabel::User => self.user.clone(),
      ApplicationLabel::Volumes => self.volumes.keys().map(|k| k.to_string()).collect::<Vec<String>>().join(","),
      ApplicationLabel::WritableStreams => self
        .writable_streams
        .clone()
        .into_iter()
        .map(|writable_stream| writable_stream.to_string())
        .collect::<Vec<String>>()
        .join(", "),
    }
  }

  fn target_label(&self) -> Option<ApplicationLabel> {
    Some(ApplicationLabel::Target)
  }
}

pub static APPLICATION_LABELS_LIST: [ApplicationLabel; 8] = [
  ApplicationLabel::Target,
  ApplicationLabel::NeedsToken,
  ApplicationLabel::Instances,
  ApplicationLabel::Cpus,
  ApplicationLabel::Mem,
  ApplicationLabel::ExposedPorts,
  ApplicationLabel::Metrics,
  ApplicationLabel::Image,
];

pub static APPLICATION_LABELS_SHOW: [ApplicationLabel; 18] = [
  ApplicationLabel::Target,
  ApplicationLabel::NeedsToken,
  ApplicationLabel::Instances,
  ApplicationLabel::Cpus,
  ApplicationLabel::Mem,
  ApplicationLabel::ExposedPorts,
  ApplicationLabel::Volumes,
  ApplicationLabel::Metrics,
  ApplicationLabel::Image,
  ApplicationLabel::HealthCheck,
  ApplicationLabel::ReadableStreams,
  ApplicationLabel::WritableStreams,
  ApplicationLabel::Secrets,
  ApplicationLabel::SingleInstance,
  ApplicationLabel::SpreadGroup,
  ApplicationLabel::Topics,
  ApplicationLabel::User,
  ApplicationLabel::Env,
];

lazy_static! {
  static ref APP_CATALOG_IMAGE_REGEX: Regex =
    Regex::new(r"APPCATALOG_REGISTRY/dsh-appcatalog/tenant/([a-z0-9-_]+)/([0-9]+)/([0-9]+)/(release|draft)/(klarrio|kpn)/([a-zA-Z0-9-_:.]+)").unwrap();
  static ref REGISTRY_IMAGE_REGEX: Regex = Regex::new(r"registry.cp.kpn-dsh.com/([a-z0-9-_]+)/([a-zA-Z0-9-_:.]+)").unwrap();
}

fn parse_image_string(image_string: &str) -> Option<(String, String)> {
  match APP_CATALOG_IMAGE_REGEX.captures(image_string) {
    Some(app_catalog_captures) => Some((
      format!(
        "app:{}:{}",
        app_catalog_captures.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
        app_catalog_captures.get(5).map(|m| m.as_str().to_string()).unwrap_or_default()
      ),
      app_catalog_captures.get(6).map(|m| m.as_str().to_string()).unwrap_or_default(),
    )),
    None => REGISTRY_IMAGE_REGEX
      .captures(image_string)
      .map(|registry_captures| ("registry".to_string(), registry_captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default())),
  }
}

#[test]
fn test_app_catalog_image_draft_kpn() {
  const APP_CATALOG_IMAGE: &str = "APPCATALOG_REGISTRY/dsh-appcatalog/tenant/greenbox-dev/1903/1903/draft/kpn/schema-store-proxy:0.2.3-0";
  assert_eq!(
    parse_image_string(APP_CATALOG_IMAGE).unwrap(),
    ("app:draft:kpn".to_string(), "schema-store-proxy:0.2.3-0".to_string())
  );
}

#[test]
fn test_app_catalog_image_release_klarrio() {
  const APP_CATALOG_IMAGE: &str = "APPCATALOG_REGISTRY/dsh-appcatalog/tenant/greenbox-dev/1903/1903/release/klarrio/whoami:1.6.1";
  assert_eq!(
    parse_image_string(APP_CATALOG_IMAGE).unwrap(),
    ("app:release:klarrio".to_string(), "whoami:1.6.1".to_string())
  );
}

#[test]
fn test_registry_image() {
  const REGISTRY_IMAGE: &str = "registry.cp.kpn-dsh.com/greenbox-dev/cck-ingestor:0.0.18";
  assert_eq!(
    parse_image_string(REGISTRY_IMAGE).unwrap(),
    ("registry".to_string(), "cck-ingestor:0.0.18".to_string())
  );
}
