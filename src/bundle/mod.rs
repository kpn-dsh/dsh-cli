pub(crate) mod ca_signed;
pub(crate) mod csr;
pub(crate) mod proxy;
pub(crate) mod rock_ca;
pub(crate) mod self_signed;

use crate::bundle::csr::CsrBuilder;
use crate::bundle::rock_ca::RockCertificateAuthority;
use crate::context::Context;
use crate::environment_variables::{environment_variable, ENV_VAR_DSH_CLI_CERTIFICATE_AUTHORITY};
use crate::error::DshCliError;
use crate::settings::Settings;
use crate::subjects::vhost::CERTIFICATE_AUTHORITY_OPTION;
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use log::debug;
use rcgen::CertificateSigningRequest;
use rock_api::client::{PkiConnector, RockApiClient};
use rock_api::error::RockApiError;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[async_trait]
pub(crate) trait CertificateAuthority {
  /// Attach intermediate and root certificate.
  ///
  /// Attach the proper intermediate and root certificates to a provided certificate
  /// (in PEM format) to complete the full certificate chain.
  ///
  /// # Parameters
  /// * `certificate_pem` - Certificate in PEM format.
  ///
  /// # Returns
  /// Certificate with the intermediate and root certificate (full chain) attached.
  async fn attach_ca_chain(&self, certificate_pem: &str) -> DshCliResult<String>;

  /// Check authorization for a tenant domain.
  ///
  /// Checks whether the authenticated certificate authority is authorized for the provided
  /// tenant domain. The `tenant_domain` must be in the format `"my-tenant.my-domain.kpn.org"`.
  ///
  /// # Parameters
  /// * `tenant_domain` - Tenant domain string.
  ///
  /// # Returns
  /// Whether the authenticated ca is authorized to sign certificates for the tenant domain.
  async fn authorization_check(&self, tenant_domain: &str) -> DshCliResult<bool>;

  /// Get intermediate and root certificate.
  ///
  /// If they exist, get the proper intermediate and root certificates in PEM format.
  ///
  /// # Returns
  /// Intermediate and root certificate (full chain).
  #[allow(unused)]
  async fn ca_chain(&self) -> DshCliResult<Option<String>>;

  /// Check connection certificate authority api.
  ///
  /// Checks whether the certificate authority api can be reached and whether the client has valid
  /// credentials.
  ///
  /// # Returns
  /// `Ok(())` - Certificate authority api can be reached and client has valid credentials.
  /// `Err(DshCliError)` - Otherwise, the error message describes the reason.
  async fn check_connection(&self) -> DshCliResult<()>;

  /// Create default csr builder.
  ///
  /// Create a [CsrBuilder] with all default settings for the certificate authority.
  ///
  /// # Returns
  /// `Ok(())` - [CsrBuilder] with all default settings for this certificate authority.
  /// `Err(DshCliError)` - Otherwise, the error message describes the reason.
  fn default_csr_builder(&self) -> DshCliResult<CsrBuilder>;

  /// Check whether a certificate already exists.
  ///
  /// Checks whether the authenticated certificate authority already has a registered and
  /// signed certificate for the vhost domain. The `vhost_domain` must be in the format
  /// `"my-vhost.my-tenant.my-domain.kpn.org"`.
  ///
  /// # Parameters
  /// * `vhost_domain` - Tenant vhost domain string.
  /// * `(context, expiration_days)` - Optional `Context` and expiration days. When present,
  ///   the metadata of the signed certificate will be printed via a `UnitFormatter`.
  ///
  /// # Returns
  /// `Some(String)` - When the certificate exists, it is returned.
  /// `None` - When the certificate does not exist.
  async fn existing_certificate(&self, vhost_domain: &str, context: Option<(&Context, u64)>) -> DshCliResult<Option<String>>;

  /// List certificate authority certificates.
  ///
  /// # Parameters
  /// * `domain` - Tenant domain string.
  /// * `context` - Used for the `ListFormatter`.
  /// * `expiration_days` - Used for printing the list.
  async fn list(&self, domain: &str, context: &Context, expiration_days: u64) -> DshCliResult<()>;

  /// Get signed certificate.
  ///
  /// # Parameters
  /// * `csr` - Certificate signing request.
  /// * `(context, expiration_days)` - Optional `Context` and expiration days. When present,
  ///   the metadata of the signed certificate will be printed via a `UnitFormatter`.
  ///
  /// # Returns
  /// Tuple containing
  /// * `String` - Certificate id.
  /// * `String` - Certificate in pem format.
  async fn sign_certificate(&self, csr: &CertificateSigningRequest, context: Option<(&Context, u64)>) -> DshCliResult<(String, String)>;
}

#[derive(Default, Deserialize, clap::ValueEnum, Clone, Debug, Serialize)]
pub(crate) enum CertificateAuthorityId {
  #[cfg(feature = "rock")]
  #[clap(name = "kpn-ca")]
  #[default]
  RockKpnCa,
  #[cfg(feature = "rock")]
  #[clap(name = "kpn-digic-rsdv")]
  RockKpnDigicRsdv,
}

impl From<CertificateAuthorityId> for PkiConnector {
  fn from(id: CertificateAuthorityId) -> Self {
    match id {
      CertificateAuthorityId::RockKpnCa => PkiConnector::Internal,
      CertificateAuthorityId::RockKpnDigicRsdv => PkiConnector::ExternalRsdv,
    }
  }
}

pub(crate) fn create_certificate_authority(id: CertificateAuthorityId) -> DshCliResult<Box<dyn CertificateAuthority + Send + Sync>> {
  debug!("create certificate authority {}", id);
  match id {
    CertificateAuthorityId::RockKpnCa | CertificateAuthorityId::RockKpnDigicRsdv => match RockApiClient::header_based_from_auth_token_file() {
      Ok(client) => RockCertificateAuthority::create(client, id.into()),
      Err(rock_api_error) => {
        debug!("{}", rock_api_error);
        match &rock_api_error {
          RockApiError::Configuration { .. } | RockApiError::ConfigurationFile { .. } => {
            err!("could not read authorization file for rocki api, please log in using \"rock-client get_auth_token\" command")
          }
          RockApiError::Json { .. } => err!("corrupted rock api authorization file, please log again in using \"rock-client get_auth_token\" command"),
          _ => Err(DshCliError::from(rock_api_error)),
        }
      }
    },
  }
}

impl Display for CertificateAuthorityId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CertificateAuthorityId::RockKpnCa => f.write_str("kpn-ca"),
      CertificateAuthorityId::RockKpnDigicRsdv => f.write_str("kpn-digic-rsdv"),
    }
  }
}

impl FromStr for CertificateAuthorityId {
  type Err = DshCliError;

  fn from_str(representation: &str) -> DshCliResult<Self> {
    match representation {
      "kpn-ca" => Ok(CertificateAuthorityId::RockKpnCa),
      "kpn-digic-rsdv" => Ok(CertificateAuthorityId::RockKpnDigicRsdv),
      _ => err!("certificate authority '{}' not recognized", representation),
    }
  }
}

/// Gets certificate authority
///
/// 1. Try command line argument --certificate-authority
/// 1. Try environment variable `DSH_CLI_CERTIFICATE_AUTHORITY`
/// 1. Try value `certificate-authority` in settings file
/// 1. Return `None`
pub(crate) fn get_certificate_authority(matches: &ArgMatches, settings: &Settings) -> DshCliResult<Option<CertificateAuthorityId>> {
  match matches.get_one::<CertificateAuthorityId>(CERTIFICATE_AUTHORITY_OPTION) {
    Some(option) => Ok(Some(option.to_owned())),
    None => match environment_variable(ENV_VAR_DSH_CLI_CERTIFICATE_AUTHORITY, Some(matches))? {
      Some(env_var) => Ok(Some(CertificateAuthorityId::from_str(env_var.as_str())?)),
      None => match &settings.certificate_authority {
        Some(setting) => Ok(Some(setting.to_owned())),
        None => Ok(None),
      },
    },
  }
}

// #[derive(Clone, Deserialize, Serialize)]
// pub(crate) struct Validation {
//   #[serde(rename = "common-name", skip_serializing_if = "Option::is_none")]
//   pub common_name: Option<String>,
//   #[serde(skip_serializing_if = "Option::is_none")]
//   pub country: Option<String>,
//   #[serde(skip_serializing_if = "Option::is_none")]
//   pub locality: Option<String>,
//   #[serde(skip_serializing_if = "Option::is_none")]
//   pub organization: Option<String>,
//   #[serde(rename = "organizational-unit", skip_serializing_if = "Option::is_none")]
//   pub organizational_unit: Option<String>,
//   #[serde(default, skip_serializing_if = "Option::is_none")]
//   pub province: Option<String>,
//   #[serde(rename = "subject-type", skip_serializing_if = "Option::is_none")]
//   pub subject_type: Option<String>,
// }
