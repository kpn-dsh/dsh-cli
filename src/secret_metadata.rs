use crate::subjects::certificate::distinguished_name_to_map;
use dsh_api::error::DshApiError;
use itertools::Itertools;
use pkcs1::{RsaPrivateKey, RsaPublicKey};
use pkcs8::PrivateKeyInfo;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use x509_cert::Certificate;

#[derive(Clone, Debug, Serialize)]
pub(crate) enum SecretMetadata {
  /// Secret contains a certificate.
  ///
  /// # Fields
  /// * `subject` - Subject or distinguished name.
  /// * `not_after` - Timestamp value of the configuration item in seconds since epoch.
  /// * `not_before` - Timestamp at which the configuration item becomes valid in seconds.
  /// * `issuer` - Issuer of the certificate.
  /// * `label` - Label in the pem file that contained the certificate.
  /// * `chain` - Chain of certificate authorities that issued the certificate. This will
  ///   always be a `SecretMetadata::Certificate` variant.
  Certificate { subject: String, not_after: u64, not_before: u64, issuer: String, label: String, chain: Vec<SecretMetadata>, serial_number: String },

  /// Secret is empty.
  Empty,

  /// Secret metadata could not be determined
  ///
  /// # Fields
  /// * `message` - Message that describes the error.
  Error { message: String },

  /// Secret metadata could not be determined caused by misconfiguration.
  ///
  /// # Fields
  /// * `message` - Message that describes the misconfiguration.
  Misconfiguration { message: String },

  /// Secret could not be found.
  ///
  /// # Fields
  /// * `message` - Message that describes the error.
  NotFound { message: Option<String> },

  /// Secrets is a private or public key.
  ///
  /// # Fields
  /// * `secret_format` - Format of the secret.
  /// * `private` - Whether the key is a private key or a public key.
  /// * `label` - Label in the pem file that contained the certificate.
  /// * `algorithm` - Key algorithm.
  Pki { secret_format: SecretFormat, private: bool, labels: Vec<String>, algorithm: Option<String> },

  /// Regular secret.
  ///
  /// # Fields
  /// * `secret_format` - Describes the format of the key.
  Regular { secret_format: SecretFormat },

  /// Secret contains settings data.
  ///
  /// # Fields
  /// * `secret_format` - Describes the format of the key.
  Settings { secret_format: SecretFormat },
}

impl SecretMetadata {
  /// Returns additional information.
  pub(crate) fn additional_info(&self) -> Option<String> {
    match self {
      Self::Certificate { label, .. } => {
        if label.is_empty() {
          None
        } else {
          Some(label.to_string())
        }
      }
      Self::Empty => None,
      Self::Error { message } => Some(message.to_string()),
      Self::Misconfiguration { message } => Some(message.to_string()),
      Self::NotFound { message } => message.clone(),
      Self::Pki { private, labels, .. } => Some(format!("{}, {}", labels.join("/"), if *private { "private" } else { "public" })),
      Self::Regular { .. } => None,
      Self::Settings { .. } => None,
    }
  }

  /// Returns the kind of secret.
  pub(crate) fn kind(&self) -> Option<&str> {
    match self {
      Self::Certificate { .. } => Some("certificate"),
      Self::Empty => Some("empty"),
      Self::Error { .. } => None,
      Self::Misconfiguration { .. } => None,
      Self::NotFound { .. } => None,
      Self::Pki { .. } => Some("pki"),
      Self::Regular { .. } => Some("regular"),
      Self::Settings { .. } => Some("settings"),
    }
  }

  /// Returns the secrets starting timestamp, if applicable.
  #[allow(dead_code)]
  pub(crate) fn not_before(&self) -> Option<u64> {
    match self {
      Self::Certificate { not_before, .. } => Some(*not_before),
      _ => None,
    }
  }

  /// Returns the secrets format.
  pub(crate) fn format(&self) -> SecretFormat {
    match self {
      Self::Certificate { .. } => SecretFormat::Pem,
      Self::Empty => SecretFormat::Empty,
      Self::Error { .. } => SecretFormat::Error,
      Self::Misconfiguration { .. } => SecretFormat::Error,
      Self::NotFound { .. } => SecretFormat::Error,
      Self::Pki { secret_format, .. } => secret_format.clone(),
      Self::Regular { secret_format, .. } => secret_format.clone(),
      Self::Settings { secret_format, .. } => secret_format.clone(),
    }
  }

  /// Returns the secrets kind of format.
  pub(crate) fn format_kind(&self) -> Option<&str> {
    match self {
      Self::Pki { secret_format, .. } => Some(secret_format.kind()),
      Self::Regular { secret_format, .. } => Some(secret_format.kind()),
      Self::Settings { secret_format, .. } => Some(secret_format.kind()),
      _ => None,
    }
  }

  /// Returns the secrets kind of format.
  pub(crate) fn value_description(&self) -> Option<String> {
    match self {
      Self::Certificate { subject, .. } => match distinguished_name_to_map(subject).get("CN") {
        Some(common_name) => Some(common_name.to_string()),
        None => Some(subject.to_string()),
      },
      Self::Empty => None,
      Self::Error { .. } => None,
      Self::Misconfiguration { .. } => None,
      Self::NotFound { .. } => None,
      Self::Pki { secret_format, .. } => Some(secret_format.to_string()),
      Self::Regular { secret_format } => Some(secret_format.to_string()),
      Self::Settings { secret_format } => Some(secret_format.to_string()),
    }
  }

  /// Returns the secrets size.
  pub(crate) fn secret_size(&self) -> Option<&SecretSize> {
    match self {
      Self::Pki { secret_format, .. } => secret_format.secret_size(),
      Self::Regular { secret_format, .. } => secret_format.secret_size(),
      Self::Settings { secret_format, .. } => secret_format.secret_size(),
      _ => None,
    }
  }

  /// Returns the number of entries, if applicable.
  pub(crate) fn number_of_entries(&self) -> Option<&usize> {
    match self {
      Self::Pki { secret_format, .. } => secret_format.number_of_entries(),
      Self::Regular { secret_format, .. } => secret_format.number_of_entries(),
      Self::Settings { secret_format, .. } => secret_format.number_of_entries(),
      _ => None,
    }
  }
}

impl From<DshApiError> for SecretMetadata {
  fn from(dsh_api_error: DshApiError) -> Self {
    match dsh_api_error {
      DshApiError::BadRequest { .. } => SecretMetadata::Error { message: "bad request".to_string() },
      DshApiError::Configuration { .. } => SecretMetadata::Error { message: "configuration error".to_string() },
      DshApiError::Conversion { .. } => SecretMetadata::Error { message: "conversion error".to_string() },
      DshApiError::NotAuthorized { .. } => SecretMetadata::Error { message: "not authorized".to_string() },
      DshApiError::Parameter { .. } => SecretMetadata::Error { message: "parameter error".to_string() },
      DshApiError::Unexpected { .. } => SecretMetadata::Error { message: "unexpected error, possibly a network failure".to_string() },
      DshApiError::Unprocessable { .. } => SecretMetadata::Error { message: "unprocessable".to_string() },
      DshApiError::NotFound { .. } => SecretMetadata::NotFound { message: Some("unprocessable".to_string()) },
    }
  }
}

/// Describes the format of a secret.
#[derive(Clone, Debug, Serialize)]
pub(crate) enum SecretFormat {
  /// Secret is empty after trimming white space.
  Empty,

  /// Secret contains an encrypted value.
  ///
  /// # Fields
  /// * `secret_size` - Size of the encrypted value.
  Encrypted { secret_size: SecretSize },

  /// Secret could not be processed
  Error,

  /// Secret contains a json array.
  ///
  /// # Fields
  /// * `number_of_elements` - Number of array elements.
  /// * `secret_size` - Size of the json array.
  JsonArray { number_of_elements: usize, secret_size: SecretSize },

  /// Secret contains a json object.
  ///
  /// # Fields
  /// * `number_of_fields` - Number of object elements.
  /// * `secret_size` - Size of the json object.
  JsonObject { number_of_fields: usize, secret_size: SecretSize },

  /// Secret contains a multi line string.
  ///
  /// # Fields
  /// * `secret-size` - Size of the multi line secret.
  MultiLine { secret_size: SecretSize },

  /// Secret contains a pem formatted string.
  Pem,

  /// Secret contains a pem label.
  PemLabel,

  /// Secret contains a pkcs1 formatted string.
  PemPkcs1,

  /// Secret contains a pkcs8 formatted string.
  PemPkcs8,

  /// Secret contains a single line string.
  ///
  /// # Fields
  /// * `secret_size` - Size of the single line secret.
  String { secret_size: SecretSize },
}

impl SecretFormat {
  pub(crate) fn kind(&self) -> &str {
    match self {
      Self::Empty => "empty",
      Self::Encrypted { .. } => "encrypted",
      Self::Error => "error",
      Self::JsonArray { .. } => "json array",
      Self::JsonObject { .. } => "json object",
      Self::MultiLine { .. } => "multi line",
      Self::Pem => "pem",
      Self::PemLabel => "pem label",
      Self::PemPkcs1 => "pkcs1",
      Self::PemPkcs8 => "pkcs8",
      Self::String { .. } => "string",
    }
  }

  pub(crate) fn number_of_entries(&self) -> Option<&usize> {
    match self {
      Self::JsonArray { number_of_elements, .. } => Some(number_of_elements),
      Self::JsonObject { number_of_fields, .. } => Some(number_of_fields),
      _ => None,
    }
  }

  pub(crate) fn secret_size(&self) -> Option<&SecretSize> {
    match self {
      Self::Encrypted { secret_size } => Some(secret_size),
      Self::JsonArray { secret_size, .. } => Some(secret_size),
      Self::JsonObject { secret_size, .. } => Some(secret_size),
      Self::MultiLine { secret_size, .. } => Some(secret_size),
      Self::String { secret_size, .. } => Some(secret_size),
      _ => None,
    }
  }
}

impl Display for SecretFormat {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Empty => write!(f, "empty"),
      Self::Encrypted { secret_size } => {
        write!(f, "encrypted, ")?;
        secret_size.fmt(f)
      }
      Self::Error => write!(f, "error"),
      Self::JsonArray { number_of_elements, secret_size } => {
        write!(f, "json array, {} elements, ", number_of_elements,)?;
        secret_size.fmt(f)
      }
      Self::JsonObject { number_of_fields, secret_size } => {
        write!(f, "json object, {} fields, ", number_of_fields)?;
        secret_size.fmt(f)
      }
      Self::MultiLine { secret_size } => {
        write!(f, "multi line, ")?;
        secret_size.fmt(f)
      }
      Self::Pem => write!(f, "pem"),
      Self::PemLabel => write!(f, "pem label"),
      Self::PemPkcs1 => write!(f, "pkcs1"),
      Self::PemPkcs8 => write!(f, "pkcs8"),
      Self::String { secret_size } => write!(f, "string, {} chars", secret_size),
    }
  }
}

/// Describes the size of a secret.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct SecretSize {
  /// Number of lines after trimming whitespace.
  number_of_lines: usize,
  /// Number of characters after trimming whitespace.
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

/// Gets metadata from secret.
///
/// # Parameters
/// * `secret` - Raw secret string.
pub(crate) fn secret_metadata(secret: &str) -> SecretMetadata {
  if secret.trim().is_empty() {
    SecretMetadata::Empty
  } else if let Ok(certificate) = try_certificates(secret) {
    certificate
  } else if let Some(encrypted_label) = get_encrypted_label(secret) {
    SecretMetadata::Pki { secret_format: SecretFormat::Encrypted { secret_size: SecretSize::from(secret) }, private: false, labels: vec![encrypted_label], algorithm: None }
  } else if let Ok(pkcs1_private_key) = try_pkcs1_private_key_metadata(secret) {
    pkcs1_private_key
  } else if let Ok(pkcs1_public_key) = try_pkcs1_public_key_metadata(secret) {
    pkcs1_public_key
  } else if let Ok(pkcs8_private_key) = try_pkcs8_private_key_metadata(secret) {
    pkcs8_private_key
  } else if let Some(labels) = get_pem_labels(secret) {
    SecretMetadata::Pki { secret_format: SecretFormat::PemLabel, private: false, labels: labels.iter().map(|label| label.to_string()).collect_vec(), algorithm: None }
  } else if let Ok(secret_format) = try_json_format(secret) {
    SecretMetadata::Settings { secret_format }
  } else if let Ok(secret_format) = try_multi_line_format(secret) {
    SecretMetadata::Settings { secret_format }
  } else {
    SecretMetadata::Regular { secret_format: SecretFormat::String { secret_size: SecretSize { number_of_lines: 1, number_of_characters: secret.trim().len() } } }
  }
}

/// Tries if the secret contains certificates.
///
/// # Parameters
/// `secret`
///
/// # Returns
/// List of certificates
fn try_certificates(secret: &str) -> Result<SecretMetadata, ()> {
  match Certificate::load_pem_chain(secret.as_bytes()) {
    Ok(certificates) => {
      if let Some(certificate) = certificates.first() {
        let certificate_chain = certificates
          .iter()
          .map(|certificate| SecretMetadata::Certificate {
            subject: certificate.tbs_certificate.subject.to_string(),
            not_after: certificate.tbs_certificate.validity.not_after.to_unix_duration().as_secs(),
            not_before: certificate.tbs_certificate.validity.not_before.to_unix_duration().as_secs(),
            issuer: certificate.tbs_certificate.issuer.to_string(),
            label: "".to_string(),
            chain: vec![],
            serial_number: certificate.tbs_certificate.serial_number.to_string(),
          })
          .collect_vec();
        Ok(SecretMetadata::Certificate {
          subject: certificate.tbs_certificate.subject.to_string(),
          not_after: certificate.tbs_certificate.validity.not_after.to_unix_duration().as_secs(),
          not_before: certificate.tbs_certificate.validity.not_before.to_unix_duration().as_secs(),
          issuer: certificate.tbs_certificate.issuer.to_string(),
          label: "".to_string(),
          chain: certificate_chain,
          serial_number: certificate.tbs_certificate.serial_number.to_string(),
        })
      } else {
        Err(())
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

fn try_pkcs1_private_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match RsaPrivateKey::try_from(secret.as_bytes()) {
    Ok(_) => Ok(SecretMetadata::Pki { secret_format: SecretFormat::PemPkcs1, private: true, labels: vec!["RSA PRIVATE KEY".to_string()], algorithm: None }),
    Err(_) => Err(()),
  }
}

fn try_pkcs1_public_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match RsaPublicKey::try_from(secret.as_bytes()) {
    Ok(_) => Ok(SecretMetadata::Pki { secret_format: SecretFormat::PemPkcs1, private: false, labels: vec!["RSA PUBLIC KEY".to_string()], algorithm: None }),
    Err(_) => Err(()),
  }
}

fn try_pkcs8_private_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match PrivateKeyInfo::try_from(secret.as_bytes()) {
    Ok(private_key_info) => Ok(SecretMetadata::Pki {
      secret_format: SecretFormat::PemPkcs8,
      private: true,
      labels: vec!["PRIVATE KEY".to_string()],
      algorithm: Some(private_key_info.algorithm.oid.to_string()),
    }),
    Err(_) => Err(()),
  }
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

fn try_json_format(secret: &str) -> Result<SecretFormat, ()> {
  match serde_json::from_str::<Value>(secret) {
    Ok(Value::Array(array)) => Ok(SecretFormat::JsonArray { number_of_elements: array.len(), secret_size: SecretSize::from(secret) }),
    Ok(Value::Object(object)) => Ok(SecretFormat::JsonObject { number_of_fields: object.len(), secret_size: SecretSize::from(secret) }),
    _ => Err(()),
  }
}

fn try_multi_line_format(secret: &str) -> Result<SecretFormat, ()> {
  let secret_size = SecretSize::from(secret);
  if secret_size.number_of_lines > 1 {
    Ok(SecretFormat::MultiLine { secret_size })
  } else {
    Err(())
  }
}
