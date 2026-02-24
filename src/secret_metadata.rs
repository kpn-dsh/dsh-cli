use itertools::Itertools;
use pkcs1::{RsaPrivateKey, RsaPublicKey};
use pkcs8::PrivateKeyInfo;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use x509_cert::Certificate;

/// Describes the size of a secret
#[derive(Clone, Serialize)]
pub struct SecretSize {
  /// Number of lines after trimming whitespace
  number_of_lines: usize,
  /// Number of characters after trimming whitespace
  number_of_characters: usize,
}

impl From<&str> for SecretSize {
  fn from(secret: &str) -> Self {
    let trimmed_secret = secret.trim();
    if trimmed_secret.is_empty() {
      SecretSize { number_of_lines: 0, number_of_characters: 0 }
    } else {
      SecretSize { number_of_lines: trimmed_secret.lines().count(), number_of_characters: secret.len() }
    }
  }
}

impl Display for SecretSize {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match (self.number_of_lines, self.number_of_characters) {
      (0, 0) => write!(f, "empty"),
      (1, number_of_characters) => write!(f, "{} chars", number_of_characters),
      (number_of_lines, number_of_characters) => write!(f, "{} lns, {} chars", number_of_lines, number_of_characters),
    }
  }
}

/// Describes the format of a secret
#[derive(Clone, Serialize)]
pub enum SecretFormat {
  /// Secret is empty after trimming white space
  Empty,
  /// Secret contains an encrypted value
  ///
  /// Parameter: `secret_size`
  Encrypted(SecretSize),
  /// Secret could not be processed
  Error,
  /// Secret contains a json array
  ///
  /// Parameters: `number_of_elements`, `secret_size`
  JsonArray(usize, SecretSize),
  /// Secret contains a json object
  ///
  /// Parameters: `number_of_fields`, `secret_size`
  JsonObject(usize, SecretSize),
  /// Secret contains a multi line string
  ///
  /// Parameter: `secret_size`
  MultiLine(SecretSize),
  /// Secret contains a pem formatted string
  Pem,
  /// Secret contains a pem label
  PemLabel,
  /// Secret contains a pkcs1 formatted string
  PemPkcs1,
  /// Secret contains a pkcs8 formatted string
  PemPkcs8,
  /// Secret contains a single line string
  ///
  /// Parameter: `secret_size`
  String(SecretSize),
}

impl SecretFormat {
  pub fn kind(&self) -> &str {
    match self {
      Self::Empty => "empty",
      Self::Encrypted(_) => "encrypted",
      Self::Error => "error",
      Self::JsonArray(_, _) => "json array",
      Self::JsonObject(_, _) => "json object",
      Self::MultiLine(_) => "multi line",
      Self::Pem => "pem",
      Self::PemLabel => "pem label",
      Self::PemPkcs1 => "pkcs1",
      Self::PemPkcs8 => "pkcs8",
      Self::String(_) => "string",
    }
  }

  pub fn number_of_entries(&self) -> Option<&usize> {
    match self {
      Self::JsonArray(number_of_elements, _) => Some(number_of_elements),
      Self::JsonObject(number_of_fields, _) => Some(number_of_fields),
      _ => None,
    }
  }

  pub fn secret_size(&self) -> Option<&SecretSize> {
    match self {
      Self::Encrypted(secret_size) => Some(secret_size),
      Self::JsonArray(_, secret_size) => Some(secret_size),
      Self::JsonObject(_, secret_size) => Some(secret_size),
      Self::MultiLine(secret_size) => Some(secret_size),
      Self::String(secret_size) => Some(secret_size),
      _ => None,
    }
  }
}

impl Display for SecretFormat {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Empty => write!(f, "empty"),
      Self::Encrypted(secret_size) => {
        write!(f, "encrypted, ")?;
        secret_size.fmt(f)
      }
      Self::Error => write!(f, "error"),
      Self::JsonArray(number_of_elements, secret_size) => {
        write!(f, "json array, {} elements, ", number_of_elements,)?;
        secret_size.fmt(f)
      }
      Self::JsonObject(number_of_fields, secret_size) => {
        write!(f, "json object, {} fields, ", number_of_fields)?;
        secret_size.fmt(f)
      }
      Self::MultiLine(secret_size) => {
        write!(f, "multi line, ")?;
        secret_size.fmt(f)
      }
      Self::Pem => write!(f, "pem"),
      Self::PemLabel => write!(f, "pem label"),
      Self::PemPkcs1 => write!(f, "pkcs1"),
      Self::PemPkcs8 => write!(f, "pkcs8"),
      Self::String(length) => write!(f, "string, {} chars", length),
    }
  }
}

#[derive(Clone, Serialize)]
pub enum SecretMetadata {
  /// Secret contains a certificate
  ///
  /// Parameters: `subject`, `not_after`, `not_before`, `issuer`, `label`
  Certificate(String, u64, u64, String, String),
  /// Secret is empty
  Empty,
  /// Secret metadata could not be determined
  ///
  /// Parameter: `error_message`
  Error(String),
  /// Secrets is a private or public key
  ///
  /// Parameters: `secret_format`, `private`, `label`, `algorithm`
  Pki(SecretFormat, bool, String, Option<String>),
  /// Regular secret
  ///
  /// Parameter: `secret_format`
  Regular(SecretFormat),
  /// Secret contains settings data
  ///
  /// Parameter: `secret_format`
  Settings(SecretFormat),
}

impl SecretMetadata {
  /// Returns additional information
  pub fn additional_info(&self) -> Option<String> {
    match self {
      Self::Certificate(_, _, _, _, label) => Some(label.to_string()),
      Self::Empty => None,
      Self::Error(message) => Some(message.to_string()),
      Self::Pki(_, private, label, _) => Some(format!("{}, {}", label, if *private { "private" } else { "public" })),
      Self::Regular(_) => None,
      Self::Settings(_) => None,
    }
  }

  /// Returns the kind of secret
  pub fn kind(&self) -> &str {
    match self {
      Self::Certificate(_, _, _, _, _) => "cert",
      Self::Empty => "empty",
      Self::Error(_) => "error",
      Self::Pki(_, _, _, _) => "pki",
      Self::Regular(_) => "regular",
      Self::Settings(_) => "settings",
    }
  }

  /// Returns the secrets expiration timestamp, if applicable
  pub fn not_after(&self) -> Option<u64> {
    match self {
      Self::Certificate(_, not_after, _, _, _) => Some(not_after.clone()),
      _ => None,
    }
  }

  /// Returns the secrets starting timestamp, if applicable
  pub fn not_before(&self) -> Option<u64> {
    match self {
      Self::Certificate(_, _, not_before, _, _) => Some(not_before.clone()),
      _ => None,
    }
  }

  /// Returns the secrets format
  pub fn format(&self) -> SecretFormat {
    match self {
      Self::Certificate(_, _, _, _, _) => SecretFormat::Pem,
      Self::Empty => SecretFormat::Empty,
      Self::Error(_) => SecretFormat::Error,
      Self::Pki(format, _, _, _) => format.clone(),
      Self::Regular(format) => format.clone(),
      Self::Settings(format) => format.clone(),
    }
  }

  /// Returns the secrets kind of format
  pub fn format_kind(&self) -> Option<&str> {
    match self {
      Self::Certificate(_, _, _, _, _) => None,
      Self::Empty => None,
      Self::Error(_) => None,
      Self::Pki(format, _, _, _) => Some(format.kind()),
      Self::Regular(format) => Some(format.kind()),
      Self::Settings(format) => Some(format.kind()),
    }
  }

  /// Returns the secrets size
  pub fn secret_size(&self) -> Option<&SecretSize> {
    match self {
      Self::Pki(secret_format, _, _, _) => secret_format.secret_size(),
      Self::Regular(secret_format) => secret_format.secret_size(),
      Self::Settings(secret_format) => secret_format.secret_size(),
      _ => None,
    }
  }

  /// Returns the number of entries, if applicable
  pub fn number_of_entries(&self) -> Option<&usize> {
    match self {
      Self::Pki(secret_format, _, _, _) => secret_format.number_of_entries(),
      Self::Regular(secret_format) => secret_format.number_of_entries(),
      Self::Settings(secret_format) => secret_format.number_of_entries(),
      _ => None,
    }
  }
}

impl Display for SecretMetadata {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Certificate(_subject, _not_after, _not_before, _issuer, _label) => write!(f, "cert"),
      Self::Empty => write!(f, "empty"),
      Self::Error(_message) => write!(f, "error"),
      Self::Pki(_format, _private, _label, _algorithm) => write!(f, "pki"),
      Self::Regular(_format) => write!(f, "regular"),
      Self::Settings(_format) => write!(f, "settings"),
    }
  }
}

/// Try if the secret contains certificates
///
/// # Parameters
/// `secret`
///
/// # Returns
/// List of certificates
pub fn try_certificates(secret: &str) -> Result<Vec<SecretMetadata>, ()> {
  match Certificate::load_pem_chain(secret.as_bytes()) {
    Ok(certificates) => {
      let certificate_enums = certificates
        .iter()
        .map(|certificate| {
          let subject = certificate.tbs_certificate.subject.to_string();
          let not_after = certificate.tbs_certificate.validity.not_after.to_unix_duration().as_secs();
          let not_before = certificate.tbs_certificate.validity.not_before.to_unix_duration().as_secs();
          let issuer = certificate.tbs_certificate.issuer.to_string();
          SecretMetadata::Certificate(subject, not_after, not_before, issuer, "".to_string())
        })
        .collect_vec();
      if certificate_enums.is_empty() {
        Err(())
      } else {
        Ok(certificate_enums)
      }
    }
    Err(_) => Err(()),
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

pub fn try_pkcs1_private_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match RsaPrivateKey::try_from(secret.as_bytes()) {
    Ok(_) => Ok(SecretMetadata::Pki(SecretFormat::PemPkcs1, true, "RSA PRIVATE KEY".to_string(), None)),
    Err(_) => Err(()),
  }
}

pub fn try_pkcs1_public_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match RsaPublicKey::try_from(secret.as_bytes()) {
    Ok(_) => Ok(SecretMetadata::Pki(SecretFormat::PemPkcs1, false, "RSA PUBLIC KEY".to_string(), None)),
    Err(_) => Err(()),
  }
}

pub fn try_pkcs8_private_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match PrivateKeyInfo::try_from(secret.as_bytes()) {
    Ok(private_key_info) => Ok(SecretMetadata::Pki(
      SecretFormat::PemPkcs8,
      true,
      "PRIVATE KEY".to_string(),
      Some(private_key_info.algorithm.oid.to_string()),
    )),
    Err(_) => Err(()),
  }
}

fn try_json_format(secret: &str) -> Result<SecretFormat, ()> {
  match serde_json::from_str::<Value>(secret) {
    Ok(Value::Array(array)) => Ok(SecretFormat::JsonArray(array.len(), SecretSize::from(secret))),
    Ok(Value::Object(object)) => Ok(SecretFormat::JsonObject(object.len(), SecretSize::from(secret))),
    _ => Err(()),
  }
}

fn try_multi_line_format(secret: &str) -> Result<SecretFormat, ()> {
  let secret_size = SecretSize::from(secret);
  if secret_size.number_of_lines > 1 {
    Ok(SecretFormat::MultiLine(secret_size))
  } else {
    Err(())
  }
}

pub fn secret_metadata(secret: &str) -> Vec<SecretMetadata> {
  if secret.trim().is_empty() {
    vec![SecretMetadata::Empty]
  } else if let Ok(pkcs1_private_key) = try_certificates(secret) {
    pkcs1_private_key
  } else if let Some(encrypted_label) = get_encrypted_label(secret) {
    vec![SecretMetadata::Pki(SecretFormat::Encrypted(SecretSize::from(secret)), false, encrypted_label, None)]
  } else if let Ok(pkcs1_private_key) = try_pkcs1_private_key_metadata(secret) {
    vec![pkcs1_private_key]
  } else if let Ok(pkcs1_public_key) = try_pkcs1_public_key_metadata(secret) {
    vec![pkcs1_public_key]
  } else if let Ok(pkcs8_private_key) = try_pkcs8_private_key_metadata(secret) {
    vec![pkcs8_private_key]
  } else if let Some(labels) = get_pem_labels(secret) {
    labels
      .iter()
      .map(|label| SecretMetadata::Pki(SecretFormat::PemLabel, false, label.to_string(), None))
      .collect_vec()
  } else if let Ok(json_format) = try_json_format(secret) {
    vec![SecretMetadata::Settings(json_format)]
  } else if let Ok(multi_line_format) = try_multi_line_format(secret) {
    vec![SecretMetadata::Settings(multi_line_format)]
  } else {
    vec![SecretMetadata::Regular(SecretFormat::String(SecretSize { number_of_lines: 1, number_of_characters: secret.trim().len() }))]
  }
}
