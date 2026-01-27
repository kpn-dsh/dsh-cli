use itertools::Itertools;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use rsa::RsaPublicKey;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use x509_parser::pem::Pem;
use x509_parser::prelude::X509Certificate;
use x509_parser::x509::X509Name;

#[derive(Clone, Serialize)]
pub(crate) struct CertificateEntry {
  pub(crate) subject: String,
  pub(crate) not_after: String,
  pub(crate) not_before: String,
  pub(crate) issuer: String,
  pub(crate) label: String,
}

fn x509_name_to_string(x509_name: &X509Name) -> String {
  [
    x509_name
      .iter_common_name()
      .next()
      .and_then(|common_name| common_name.as_str().ok().map(|name| format!("CN={}", name))),
    x509_name
      .iter_organization()
      .next()
      .and_then(|organization| organization.as_str().ok().map(|org| format!("O={}", org))),
  ]
  .iter()
  .flatten()
  .join(", ")
}

impl From<(X509Certificate<'_>, String)> for CertificateEntry {
  fn from((certificate, label): (X509Certificate, String)) -> Self {
    Self {
      subject: x509_name_to_string(&certificate.subject),
      not_after: certificate.validity.not_after.to_string(),
      not_before: certificate.validity.not_before.to_string(),
      issuer: x509_name_to_string(&certificate.issuer),
      label,
    }
  }
}

#[derive(Clone, Serialize)]
pub(crate) struct KeyEntry {
  pub(crate) kind: String,
  pub(crate) private: bool,
  pub(crate) size: usize,
  pub(crate) validated: Option<bool>,
  pub(crate) label: String,
}

impl From<(String, String, RsaPrivateKey)> for KeyEntry {
  fn from((kind, label, rsa_private_key): (String, String, RsaPrivateKey)) -> Self {
    Self { kind, private: true, size: rsa_private_key.size(), validated: Some(rsa_private_key.validate().is_ok()), label }
  }
}

impl From<(String, String, RsaPublicKey)> for KeyEntry {
  fn from((kind, label, rsa_public_key): (String, String, RsaPublicKey)) -> Self {
    Self { kind, private: false, size: rsa_public_key.size(), validated: None, label }
  }
}

#[derive(Clone, Serialize)]
pub(crate) enum SecretKind {
  Certificate,
  Empty,
  Error,
  Pki,
  Regular,
  Settings,
  System,
}

impl Display for SecretKind {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Certificate => write!(f, "cert"),
      Self::Empty => write!(f, "empty"),
      Self::Error => write!(f, "error"),
      Self::Pki => write!(f, "pki"),
      Self::Regular => write!(f, "regular"),
      Self::Settings => write!(f, "settings"),
      Self::System => write!(f, "system"),
    }
  }
}

#[derive(Clone, Serialize)]
pub(crate) struct SecretEntry {
  pub(crate) kind: SecretKind,
  pub(crate) format: String,
  pub(crate) description: Vec<String>,
  pub(crate) expires: Option<String>,
}

impl SecretEntry {
  pub(crate) fn new(kind: SecretKind, format: String, description: Option<String>, expires: Option<String>) -> Self {
    Self { kind, format, description: description.map(|desc| vec![desc]).unwrap_or_default(), expires }
  }
}

impl From<(SecretKind, String, X509Certificate<'_>)> for SecretEntry {
  fn from((kind, format, certificate): (SecretKind, String, X509Certificate)) -> Self {
    Self { kind, format, description: vec![x509_name_to_string(&certificate.subject)], expires: Some(certificate.validity().not_after.to_string()) }
  }
}

impl From<(SecretKind, String, RsaPrivateKey)> for SecretEntry {
  fn from((kind, format, _rsa_private_key): (SecretKind, String, RsaPrivateKey)) -> Self {
    Self::new(kind, format, None, None)
  }
}

impl From<(SecretKind, String, RsaPublicKey)> for SecretEntry {
  fn from((kind, format, _rsa_public_key): (SecretKind, String, RsaPublicKey)) -> Self {
    Self::new(kind, format, None, None)
  }
}

pub(crate) fn get_certificates_from_pem(pem: &str) -> Option<Vec<CertificateEntry>> {
  let certificates = Pem::iter_from_buffer(pem.as_bytes())
    .flat_map(|pem_entry| pem_entry.ok())
    .flat_map(|pem| pem.parse_x509().map(|certificate| CertificateEntry::from((certificate, pem.label.clone()))))
    .collect_vec();
  if certificates.is_empty() {
    None
  } else {
    Some(certificates)
  }
}

fn get_begin_label(line: &str) -> Option<&str> {
  if let Some(prefix_stripped) = line.strip_prefix("-----BEGIN ") {
    if let Some(suffix_stripped) = prefix_stripped.strip_suffix("-----") {
      return Some(suffix_stripped);
    }
  }
  None
}

fn get_end_label(line: &str) -> Option<&str> {
  if let Some(prefix_stripped) = line.strip_prefix("-----END ") {
    if let Some(suffix_stripped) = prefix_stripped.strip_suffix("-----") {
      return Some(suffix_stripped);
    }
  }
  None
}

fn get_pem_labels(pem: &str) -> Option<Vec<&str>> {
  let mut labels = vec![];
  let mut current_label: Option<&str> = None;
  for line in pem.lines() {
    match current_label {
      Some(label) => {
        if let Some(end_label) = get_end_label(line) {
          if label == end_label {
            labels.push(end_label);
            current_label = None;
          }
        }
      }
      None => {
        if let Some(begin_label) = get_begin_label(line) {
          current_label = Some(begin_label)
        }
      }
    }
  }
  if labels.is_empty() {
    None
  } else {
    Some(labels)
  }
}

fn get_encrypted_label(pem: &str) -> Option<String> {
  let lines = pem.lines().collect_vec();
  if lines.get(1).is_some_and(|line| *line == "Proc-Type: 4,ENCRYPTED") {
    if let Some(prefix_stripped) = lines.first().unwrap().strip_prefix("-----BEGIN ") {
      if let Some(suffix_stripped) = prefix_stripped.strip_suffix("-----") {
        return Some(suffix_stripped.to_string());
      }
    }
  }
  None
}

pub(crate) fn get_keys_from_pem(secret_value: &str) -> Option<Vec<(RsaPrivateKey, String, &str)>> {
  let keys: Vec<(RsaPrivateKey, String, &str)> = Pem::iter_from_buffer(secret_value.as_bytes())
    .filter_map(|pem_entry| match pem_entry {
      Ok(pem) => match RsaPrivateKey::from_pkcs1_pem(secret_value) {
        Ok(pkcs1_pem_private_key) => Some((pkcs1_pem_private_key, pem.label.clone(), "pkcs1")),
        Err(_) => match RsaPrivateKey::from_pkcs8_pem(secret_value) {
          Ok(pkcs8_pem_private_key) => Some((pkcs8_pem_private_key, pem.label.clone(), "pkcs8")),
          Err(_) => None,
        },
      },
      Err(_) => None,
    })
    .collect_vec();
  if keys.is_empty() {
    None
  } else {
    Some(keys)
  }
}

fn secret_size(secret: &str) -> String {
  let trimmed_secret = secret.trim();
  if trimmed_secret.is_empty() {
    "empty".to_string()
  } else {
    let number_of_lines = trimmed_secret.lines().count();
    if number_of_lines == 1 {
      format!("single line, {} characters", secret.len())
    } else {
      format!("{} lines, {} characters", number_of_lines, secret.len())
    }
  }
}

fn get_json(secret_value: &str) -> Option<String> {
  match serde_json::from_str::<Value>(secret_value) {
    Ok(Value::Array(array)) => Some(format!("json array with {} elements ({})", array.len(), secret_size(secret_value))),
    Ok(Value::Object(_)) => Some(format!("json object ({})", secret_size(secret_value))),
    _ => None,
  }
}

fn get_toml(secret_value: &str) -> Option<String> {
  match toml::from_str::<Value>(secret_value) {
    Ok(Value::Array(array)) => Some(format!("toml array with {} elements ({})", array.len(), secret_size(secret_value))),
    Ok(Value::Object(_)) => Some(format!("toml object ({})", secret_size(secret_value))),
    _ => None,
  }
}

fn get_yaml(secret_value: &str) -> Option<String> {
  match serde_yaml::from_str::<Value>(secret_value) {
    Ok(Value::Array(array)) => Some(format!("yaml array with {} elements ({})", array.len(), secret_size(secret_value))),
    Ok(Value::Object(_)) => Some(format!("yaml object ({})", secret_size(secret_value))),
    _ => None,
  }
}

fn get_multiline(secret_value: &str) -> Option<String> {
  if secret_value.lines().count() > 1 {
    Some(secret_size(secret_value))
  } else {
    None
  }
}

pub(crate) fn secret_entries_from(secret_value: &str, is_system: bool) -> Vec<SecretEntry> {
  if secret_value.trim().is_empty() {
    vec![SecretEntry::new(SecretKind::Empty, "".to_string(), None, None)]
  } else if is_system {
    vec![SecretEntry::new(SecretKind::System, "plain".to_string(), None, None)]
  } else if let Some(certificate_entries) = get_certificates_from_pem(secret_value) {
    certificate_entries
      .into_iter()
      .map(|certificate_entry| {
        SecretEntry::new(
          SecretKind::Certificate,
          "pem".to_string(),
          Some(certificate_entry.subject),
          Some(certificate_entry.not_after),
        )
      })
      .collect_vec()
  } else if let Some(encrypted_label) = get_encrypted_label(secret_value) {
    vec![SecretEntry::new(SecretKind::Pki, "encrypted".to_string(), Some(encrypted_label), None)]
  } else if let Some(keys) = get_keys_from_pem(secret_value) {
    keys
      .iter()
      .map(|(key, label, kind)| SecretEntry::new(SecretKind::Pki, format!("pem.{}", kind), Some(format!("{}, {} bit", label, key.size())), None))
      .collect_vec()
  } else if let Some(labels) = get_pem_labels(secret_value) {
    labels
      .iter()
      .map(|label| SecretEntry::new(SecretKind::Pki, "pem.label".to_string(), Some(label.to_string()), None))
      .collect_vec()
  } else if let Some(description) = get_json(secret_value) {
    vec![SecretEntry::new(SecretKind::Settings, "json".to_string(), Some(description), None)]
  } else if let Some(description) = get_toml(secret_value) {
    vec![SecretEntry::new(SecretKind::Settings, "toml".to_string(), Some(description), None)]
  } else if let Some(description) = get_yaml(secret_value) {
    vec![SecretEntry::new(SecretKind::Settings, "yaml".to_string(), Some(description), None)]
  } else if let Some(description) = get_multiline(secret_value) {
    vec![SecretEntry::new(SecretKind::Settings, "multi-line".to_string(), Some(description), None)]
  } else {
    vec![SecretEntry::new(SecretKind::Regular, "plain".to_string(), None, None)]
  }
}
