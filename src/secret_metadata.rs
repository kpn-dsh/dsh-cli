use crate::subjects::secret::SecretWithMetadata;
use crate::DshCliResult;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::DshApiError;
use dsh_api::secret::SecretInjection;
use dsh_api::types::AllocationStatus;
use dsh_api::Dependant;
use futures::future::{join, join_all};
use futures::FutureExt;
use itertools::Itertools;
use pkcs1::{RsaPrivateKey, RsaPublicKey};
use pkcs8::PrivateKeyInfo;
use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};
use x509_cert::attr::AttributeType;
use x509_cert::name::Name;
use x509_cert::Certificate;

/// Certificate secret metadata.
///
/// # Fields
/// * `issuer_common_name` - Common name of the issuer of the certificate.
/// * `issuer_distinguished_name` - Distinguished name of the issuer of the certificate.
/// * `label` - Label in the pem file that contained the certificate.
/// * `not_after` - Timestamp value of the configuration item in seconds since epoch.
/// * `not_before` - Timestamp at which the configuration item becomes valid in seconds.
/// * `serial_number` - Certificate serial number.
/// * `subject_common_name` - Subject common name.
/// * `subject_distinguished_name` - Subject distinguished name.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct CertificateSecretMetadata {
  pub(crate) issuer_common_name: String,
  pub(crate) issuer_distinguished_name: String,
  pub(crate) not_after: u64,
  pub(crate) not_before: u64,
  pub(crate) serial_number: String,
  pub(crate) subject_common_name: String,
  pub(crate) subject_distinguished_name: String,
}

impl CertificateSecretMetadata {
  pub(crate) fn is_self_issued(&self) -> bool {
    self.issuer_distinguished_name == self.subject_distinguished_name
  }
}

/// Pki secret metadata.
///
/// # Fields
/// * `algorithm` - Key algorithm.
/// * `label` - Label in the pem file that contained the certificate.
/// * `private` - Whether the key is a private key or a public key.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PkiSecretMetadata {
  pub(crate) algorithm: Option<String>,
  pub(crate) labels: Vec<String>,
  pub(crate) private: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) enum SecretMetadata {
  /// Secret contains a certificate.
  ///
  /// # Fields
  /// * `secret_format` - Format of the secret.
  /// * `chain` - Chain of tuples containing the serial numbers and common names (in that order)
  ///   of certificate authorities that issued the certificate.
  /// * `parts` - Components of the certificates pem format.
  Certificate { secret_format: SecretFormat, chain: Vec<(String, String)>, parts: Vec<CertificateSecretMetadata> },

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
  /// * `metadata` - Pki secrete metadata.
  Pki { secret_format: SecretFormat, metadata: PkiSecretMetadata },

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
      Self::Certificate { .. } => None, // TODO
      Self::Empty => None,
      Self::Error { message } => Some(message.to_string()),
      Self::Misconfiguration { message } => Some(message.to_string()),
      Self::NotFound { message } => message.clone(),
      Self::Pki { metadata, .. } => Some(format!("{}, {}", metadata.labels.join("/"), if metadata.private { "private" } else { "public" })),
      Self::Regular { .. } => None,
      Self::Settings { .. } => None,
    }
  }

  /// Returns the kind of secret.
  pub(crate) fn kind(&self) -> Option<&str> {
    match self {
      Self::Certificate { .. } => Some("cert"),
      Self::Empty => Some("empty"),
      Self::Error { .. } => None,
      Self::Misconfiguration { .. } => None,
      Self::NotFound { .. } => None,
      Self::Pki { .. } => Some("pki"),
      Self::Regular { .. } => Some("regular"),
      Self::Settings { .. } => Some("settings"),
    }
  }

  /// Returns the secrets format.
  pub(crate) fn format(&self) -> SecretFormat {
    match self {
      Self::Certificate { secret_format, .. } => secret_format.clone(),
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
  pub(crate) fn value_description(&self) -> Option<String> {
    match self {
      Self::Certificate { .. } => None, // TODO
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
      Self::Certificate { secret_format, .. } => secret_format.secret_size(),
      Self::Pki { secret_format, .. } => secret_format.secret_size(),
      Self::Regular { secret_format, .. } => secret_format.secret_size(),
      Self::Settings { secret_format, .. } => secret_format.secret_size(),
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
  ///
  /// # Fields
  /// * `number_of_components` - Number of pem components.
  Pem { number_of_components: usize },

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
      Self::Encrypted { .. } => write!(f, "encrypted, "),
      Self::Error => write!(f, "error"),
      Self::JsonArray { number_of_elements, .. } => write!(f, "json array ({})", number_of_elements,),
      Self::JsonObject { .. } => write!(f, "json object"),
      Self::MultiLine { secret_size } => write!(f, "multi line ({})", secret_size.number_of_lines),
      Self::Pem { number_of_components } => {
        if *number_of_components == 1 {
          write!(f, "pem")
        } else {
          write!(f, "pem:{}", number_of_components)
        }
      }
      Self::PemLabel => write!(f, "pem"),
      Self::PemPkcs1 => write!(f, "pkcs1"),
      Self::PemPkcs8 => write!(f, "pkcs8"),
      Self::String { .. } => write!(f, "string"),
    }
  }
}

/// Describes the size of a secret.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct SecretSize {
  /// Number of lines after trimming whitespace.
  number_of_lines: usize,
  /// Number of characters after trimming whitespace.
  pub(crate) number_of_characters: usize,
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

/// Gets all secrets with metadata and allocation status.
///
/// # Returns
/// List of tuples, each consisting of:
/// * `String` - Secret name.
/// * `Option<String>` - Secret id when secret is a system secret, empty otherwise.
/// * `Vec<SecretMetadata>` - List containing the metadata.
/// * `Option<AllocationStatus>` - Secrets allocation status for non-system secrets.
/// * `Vec<Dependant<SecretInjection>>` - Apps, applications and proxies that depend on the secret.
pub(crate) async fn secrets_with_metadata(client: &DshApiClient) -> DshCliResult<Vec<SecretWithMetadata>> {
  let secrets: Vec<(String, Option<String>, Vec<Dependant<SecretInjection>>)> = client.secrets_with_dependants().await?;
  Ok(
    join_all(secrets.into_iter().map(|(secret_name, secret_id, dependants)| {
      secret_with_metadata(secret_name.clone(), client).map(|(metadata, allocation_status)| SecretWithMetadata {
        name: secret_name,
        id: secret_id,
        metadata,
        allocation_status,
        dependants,
      })
    }))
    .await,
  )
}

/// Gets secret with metadata.
///
/// Gets the secret value and optional allocation status. The value will be converted to a
/// `SecretMetadata` struct. Note that system secrets do not have an allocation status.
///
/// # Parameters
/// * `secret_name` - Secret name.
/// * `client` - Dsh api client.
///
/// # Returns
/// Tuple consisting of:
/// * `Vec<SecretMetadata>` - List containing the metadata items.
/// * `Option<AllocationStatus>` - Secrets allocation status for non-system secrets,
///   empty otherwise.
pub(crate) async fn secret_with_metadata(secret_name: String, client: &DshApiClient) -> (SecretMetadata, Option<AllocationStatus>) {
  match join(client.get_secret(&secret_name), client.get_secret_status(&secret_name)).await {
    (Ok(secret_value), Ok(allocation_status)) => (secret_metadata(&secret_value), Some(allocation_status)),
    (Ok(secret_value), Err(_)) => (secret_metadata(&secret_value), None),
    (Err(get_secret_error), Ok(allocation_status)) => match get_secret_error {
      DshApiError::NotFound { .. } => (
        SecretMetadata::Misconfiguration { message: "secret not found, possibly pending".to_string() },
        Some(allocation_status),
      ),
      other_error => (SecretMetadata::from(other_error), Some(allocation_status)),
    },
    (Err(get_secret_error), Err(_)) => match get_secret_error {
      DshApiError::NotFound { .. } => (
        SecretMetadata::Misconfiguration { message: "secret allocation status not found, possibly pending".to_string() },
        None,
      ),
      other_error => (SecretMetadata::from(other_error), None),
    },
  }
}

impl From<&str> for SecretMetadata {
  fn from(secret: &str) -> Self {
    secret_metadata(secret)
  }
}

/// Get metadata from secret.
///
/// # Parameters
/// * `secret` - Raw secret string.
fn secret_metadata(secret: &str) -> SecretMetadata {
  if secret.trim().is_empty() {
    SecretMetadata::Empty
  } else if let Some(secret_metadata) = find_certificate_secret_metadata(secret) {
    secret_metadata
  } else if let Some(encrypted_label) = get_encrypted_label(secret) {
    SecretMetadata::Pki {
      secret_format: SecretFormat::Encrypted { secret_size: SecretSize::from(secret) },
      metadata: PkiSecretMetadata { algorithm: None, labels: vec![encrypted_label], private: false },
    }
  } else if let Ok(pkcs1_private_key) = try_pkcs1_private_key_metadata(secret) {
    pkcs1_private_key
  } else if let Ok(pkcs1_public_key) = try_pkcs1_public_key_metadata(secret) {
    pkcs1_public_key
  } else if let Ok(pkcs8_private_key) = try_pkcs8_private_key_metadata(secret) {
    pkcs8_private_key
  } else if let Some(labels) = get_pem_labels(secret) {
    SecretMetadata::Pki {
      secret_format: SecretFormat::PemLabel,
      metadata: PkiSecretMetadata { algorithm: None, labels: labels.iter().map(|label| label.to_string()).collect_vec(), private: false },
    }
  } else if let Ok(secret_format) = try_json_format(secret) {
    SecretMetadata::Settings { secret_format }
  } else if let Ok(secret_format) = try_multi_line_format(secret) {
    SecretMetadata::Settings { secret_format }
  } else {
    SecretMetadata::Regular { secret_format: SecretFormat::String { secret_size: SecretSize { number_of_lines: 1, number_of_characters: secret.trim().len() } } }
  }
}

/// Find certificates in a secret.
///
/// # Parameters
/// `secret`
///
/// # Returns
/// Tuple consisting of:
/// * `Some(Vec<CertificateSecretMetadata>)` - When the string could be parsed and contains certificates.
/// * `None` - If the string does not contain certificates.
pub(crate) fn find_certificates(secret: &str) -> Option<Vec<CertificateSecretMetadata>> {
  match Certificate::load_pem_chain(secret.as_bytes()) {
    Ok(certificates) => {
      if certificates.is_empty() {
        None
      } else {
        Some(
          certificates
            .iter()
            .map(|certificate| CertificateSecretMetadata {
              issuer_common_name: get_common_name(&certificate.tbs_certificate.issuer),
              issuer_distinguished_name: certificate.tbs_certificate.issuer.to_string(),
              not_after: certificate.tbs_certificate.validity.not_after.to_unix_duration().as_secs(),
              not_before: certificate.tbs_certificate.validity.not_before.to_unix_duration().as_secs(),
              serial_number: certificate.tbs_certificate.serial_number.to_string().replace(":", ""),
              subject_common_name: get_common_name(&certificate.tbs_certificate.subject),
              subject_distinguished_name: certificate.tbs_certificate.subject.to_string(),
            })
            .collect_vec(),
        )
      }
    }
    Err(_) => None,
  }
}

/// Find certificates in a secret string.
///
/// # Parameters
/// `secret` - String containing the secrete.
///
/// # Returns
/// * `Some(SecretMetadata::Certificate)` - If string contains one or more certificates.
/// * `None` - Otherwise.
fn find_certificate_secret_metadata(secret: &str) -> Option<SecretMetadata> {
  match find_certificates(secret) {
    Some(parts) => {
      let chain = parts
        .iter()
        .map(|metadata| (metadata.serial_number.clone(), metadata.subject_common_name.clone()))
        .collect_vec();
      let secret_format = SecretFormat::Pem { number_of_components: parts.len() };
      Some(SecretMetadata::Certificate { secret_format, chain, parts })
    }
    None => None,
  }
}

fn get_common_name(name: &Name) -> String {
  let common_name_object_identifier = AttributeType::new("2.5.4.3").unwrap();
  name
    .as_ref()
    .iter()
    .find_map(|rdn| {
      rdn
        .as_ref()
        .iter()
        .find(|ss| ss.oid == common_name_object_identifier)
        .map(|sss| String::from_utf8(Vec::from(sss.value.value())).unwrap_or_default())
    })
    .unwrap_or_default()
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
    Ok(_) => {
      Ok(SecretMetadata::Pki { secret_format: SecretFormat::PemPkcs1, metadata: PkiSecretMetadata { algorithm: None, labels: vec!["RSA PRIVATE KEY".to_string()], private: true } })
    }
    Err(_) => Err(()),
  }
}

fn try_pkcs1_public_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match RsaPublicKey::try_from(secret.as_bytes()) {
    Ok(_) => {
      Ok(SecretMetadata::Pki { secret_format: SecretFormat::PemPkcs1, metadata: PkiSecretMetadata { algorithm: None, labels: vec!["RSA PUBLIC KEY".to_string()], private: false } })
    }
    Err(_) => Err(()),
  }
}

fn try_pkcs8_private_key_metadata(secret: &str) -> Result<SecretMetadata, ()> {
  match PrivateKeyInfo::try_from(secret.as_bytes()) {
    Ok(private_key_info) => Ok(SecretMetadata::Pki {
      secret_format: SecretFormat::PemPkcs8,
      metadata: PkiSecretMetadata { algorithm: Some(private_key_info.algorithm.oid.to_string()), labels: vec!["PRIVATE KEY".to_string()], private: true },
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
