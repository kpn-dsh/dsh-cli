pub(crate) mod csr;
pub(crate) mod proxy;
pub(crate) mod rock_ca;
pub(crate) mod self_ca;
pub(crate) mod vhost;

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
use rcgen::{
  Certificate as RcgenCertificate, CertificateParams, CertificateSigningRequest, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue, DnValue::PrintableString,
  ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, RsaKeySize,
};
use rock_api::client::{PkiConnector, RockApiClient};
use rock_api::error::RockApiError;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use time::OffsetDateTime;

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

  /// Check connection with _RoCK API_.
  ///
  /// Checks whether the _RoCK API_ can be reached and whether the client has valid credentials.
  ///
  /// # Returns
  /// `Ok(())` - _RoCK API_ can be reached and client has valid credentials.
  /// `Err(DshCliError)` - Otherwise, the error message describes the reason.
  async fn check_connection(&self) -> DshCliResult<()>;

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

  /// List RoCK API certificates.
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
  async fn signed_certificate(&self, csr: &CertificateSigningRequest, context: Option<(&Context, u64)>) -> DshCliResult<(String, String)>;
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

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct _Validation {
  #[serde(rename = "common-name", skip_serializing_if = "Option::is_none")]
  pub common_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub country: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locality: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub organization: Option<String>,
  #[serde(rename = "organizational-unit", skip_serializing_if = "Option::is_none")]
  pub organizational_unit: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub province: Option<String>,
  #[serde(rename = "subject-type", skip_serializing_if = "Option::is_none")]
  pub subject_type: Option<String>,
}

/// Contains certificate and key-pair.
///
/// * `certificate: Certificate`
/// * `key_pair: KeyPair`
pub(crate) struct DshCertificate {
  pub certificate: RcgenCertificate,
  pub key_pair: KeyPair,
}

// /// Contains certificate signing request and key-pair.
// ///
// /// * `csr` - Certificate signing request.
// /// * `key_pair` - Public and private key pair
// pub(crate) struct DshCsr {
//   pub csr: CertificateSigningRequest,
//   pub key_pair: KeyPair,
// }

// impl Debug for DshCsr {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     let mut builder = f.debug_struct("DshCsr");
//     // TODO
//     builder.field("csr", &"TODO");
//     builder.field("key_pair", &self.key_pair);
//     builder.finish()
//   }
// }

impl Debug for DshCertificate {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("DshCertificate");
    builder.field("key", &self.key_pair);
    builder.field("cert", self.certificate.params());
    builder.finish()
  }
}

/// Generates self-signed certificate authority certificate.
///
/// Certificate contents:
/// * Subject alt names: CA common name from parameter `ca_common_name.
/// * Distinguished name:
///   * `CN` - CA common name from parameter `ca_common_name
///   * `O` - `"Koninklijke KPN N.V."`
///   * `L` - `"Rotterdam"`
///   * `ST` - `"Zuid-Holland"`
///   * `C` - `"NL"`
///
/// # Parameters
/// * `ca_common_name` - Name to use for the self-signed certificate.
fn generate_ca_certificate<T>(ca_common_name: T) -> DshCliResult<DshCertificate>
where
  T: Into<String> + Copy,
{
  let distinguished_name = kpn_distinguished_name(ca_common_name.into(), None::<T>);
  let mut params: CertificateParams = CertificateParams::new(vec![ca_common_name.into()])?;
  params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
  params.distinguished_name = distinguished_name;
  (params.not_before, params.not_after) = not_before_not_after(365);
  params.use_authority_key_identifier_extension = true;
  params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign, KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth, ExtendedKeyUsagePurpose::ClientAuth];
  let key_pair = generate_key_pair()?;
  let certificate = params.self_signed(&key_pair)?;
  Ok(DshCertificate { certificate, key_pair })
}

/// Generates client certificate.
///
/// Signed by the provided `ca_certificate`.
/// Contains optional `OU` field for ACL groups.
///
/// Certificate contents:
/// * Distinguished name:
///   * `CN` - Client id from proxy bundle configuration: `config.client_id`.
///   * `O` - `"Koninklijke KPN N.V."`
///   * `OU` - Optional, organizational unit name from proxy bundle configuration:
///     `config.acl_group_name`.
///   * `L` - `"Rotterdam"`
///   * `ST` - `"Zuid-Holland"`
///   * `C` - `"NL"`
///
/// # Parameters
/// * `client_common_name` - Client common name.
/// * `organizational_unit_name` - Optional organizational unit name.
/// * `ca_certificate` - Certificate authority certificate used to sign the generated certificate.
fn generate_client_certificate<S, T>(client_common_name: S, organizational_unit_name: Option<T>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate>
where
  S: Into<DnValue>,
  T: Into<DnValue>,
{
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(vec![])?;
  params.distinguished_name = kpn_distinguished_name(client_common_name, organizational_unit_name);
  (params.not_before, params.not_after) = not_before_not_after;
  params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
  let key_pair = generate_key_pair()?;
  let csr_pem = params.serialize_request(&key_pair)?.pem()?;
  let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
  (csr_params.params.not_before, csr_params.params.not_after) = not_before_not_after;
  let certificate = csr_params.signed_by(&ca_certificate.certificate, &ca_certificate.key_pair)?;
  Ok(DshCertificate { certificate, key_pair })
}

/// Generates server certificate.
///
/// Signed by the provided `ca_certificate`.
///
/// Certificate contents:
/// * Subject alt names: List of dns entries from proxy bundle configuration (`config.dns_entries`).
/// * Distinguished name:
///   * `CN` - Common name from proxy bundle configuration: `config.common_name`.
///   * `O` - `"Koninklijke KPN N.V."`
///   * `L` - `"Rotterdam"`
///   * `ST` - `"Zuid-Holland"`
///   * `C` - `"NL"`
///
/// # Parameters
/// * `server_common_name` - Proxy certificate bundle configuration:
/// * `subject_alt_names` - Optional list of subject alt names.
/// * `ca_certificate` - Certificate authority certificate used to sign the generated certificate.
fn generate_server_certificate<T>(server_common_name: T, subject_alt_names: impl Into<Vec<String>>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate>
where
  T: Into<DnValue>,
{
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(subject_alt_names)?;
  params.distinguished_name = kpn_distinguished_name(server_common_name, None::<T>);
  (params.not_before, params.not_after) = not_before_not_after;
  params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
  let key_pair = generate_key_pair()?;
  let csr_pem = params.serialize_request(&key_pair)?.pem()?;
  let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
  (csr_params.params.not_before, csr_params.params.not_after) = not_before_not_after;
  let certificate = csr_params.signed_by(&ca_certificate.certificate, &ca_certificate.key_pair)?;
  Ok(DshCertificate { certificate, key_pair })
}

// pub fn generate_csr<T>(common_name: T, subject_alt_names: impl Into<Vec<String>>) -> DshCliResult<DshCsr>
// where
//   T: Into<DnValue>,
// {
//   let mut params: CertificateParams = CertificateParams::new(subject_alt_names)?;
//   params.is_ca = IsCa::NoCa;
//   let mut distinguished_name = DistinguishedName::new();
//   distinguished_name.push(DnType::CommonName, common_name);
//   distinguished_name.push(DnType::CountryName, "NL");
//   distinguished_name.push(DnType::StateOrProvinceName, "Zuid-Holland");
//   distinguished_name.push(DnType::LocalityName, "Rotterdam");
//   distinguished_name.push(DnType::OrganizationName, PrintableString("Koninklijke KPN N.V.".try_into()?));
//   params.distinguished_name = distinguished_name;
//   let now = time::OffsetDateTime::now_utc();
//   params.not_before = now;
//   params.not_after = now + time::Duration::days(365);
//   params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
//   params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
//   let key_pair = generate_key_pair()?;
//   let csr = params.serialize_request(&key_pair)?;
//   Ok(DshCsr { csr, key_pair })
// }

fn not_before_not_after(days: i64) -> (OffsetDateTime, OffsetDateTime) {
  let now = time::OffsetDateTime::now_utc();
  (now, now + time::Duration::days(days))
}

const KPN_ORGANIZATION_NAME: &str = "Koninklijke KPN N.V.";
const KPN_LOCALITY_NAME: &str = "Rotterdam";
const KPN_STATE_OR_PROVINCE_NAME: &str = "Zuid-Holland";
const KPN_COUNTRY_NAME: &str = "NL";

/// Create distinguished name.
///
/// * `CN` - Provided common name.
/// * `O` - `"Koninklijke KPN N.V."`
/// * `OU` - Optional, provided organizational unit name used for acl group capability.
/// * `L` - `"Rotterdam"`
/// * `ST` - `"Zuid-Holland"`
/// * `C` - `"NL"`
///
/// # Parameters
/// * `common_name` - Common name.
/// * `organizational_unit_name` - Optional organizational unit name.
fn kpn_distinguished_name<S, T>(common_name: S, organizational_unit_name: Option<T>) -> DistinguishedName
where
  S: Into<DnValue>,
  T: Into<DnValue>,
{
  let mut distinguished_name = DistinguishedName::new();
  distinguished_name.push(DnType::CommonName, common_name.into());
  distinguished_name.push(DnType::OrganizationName, KPN_ORGANIZATION_NAME);
  if let Some(organizational_unit_name) = organizational_unit_name {
    distinguished_name.push(DnType::OrganizationalUnitName, organizational_unit_name.into());
  }
  distinguished_name.push(DnType::LocalityName, KPN_LOCALITY_NAME);
  distinguished_name.push(DnType::StateOrProvinceName, KPN_STATE_OR_PROVINCE_NAME);
  distinguished_name.push(DnType::CountryName, KPN_COUNTRY_NAME);
  distinguished_name
}

#[allow(dead_code)] // TODO
fn kpn_distinguished_name2(common_name: &str, organizational_unit_name: Option<&str>) -> DistinguishedName {
  let mut distinguished_name = DistinguishedName::new();
  distinguished_name.push(DnType::CommonName, common_name);
  distinguished_name.push(DnType::OrganizationName, PrintableString(KPN_ORGANIZATION_NAME.try_into().unwrap()));
  if let Some(organizational_unit_name) = organizational_unit_name {
    distinguished_name.push(DnType::OrganizationalUnitName, organizational_unit_name);
  }
  distinguished_name.push(DnType::LocalityName, KPN_LOCALITY_NAME);
  distinguished_name.push(DnType::StateOrProvinceName, KPN_STATE_OR_PROVINCE_NAME);
  distinguished_name.push(DnType::CountryName, KPN_COUNTRY_NAME);
  distinguished_name
}

fn generate_key_pair() -> DshCliResult<KeyPair> {
  KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, RsaKeySize::_4096).map_err(DshCliError::from)
}

#[test]
#[ignore]
fn test() {
  let common_name = "COMMON_NAME";
  let dn = kpn_distinguished_name(common_name, None::<&str>);
  let dn2 = kpn_distinguished_name2(common_name, None);
  assert_eq!(dn, dn2);
}

#[test]
fn test2() -> Result<(), Box<dyn std::error::Error>> {
  fn csr(config: &proxy::ProxyCertificateBundleConfig, distinguished_name: DistinguishedName, key_pair: &KeyPair) -> rcgen::CertificateSigningRequest {
    let mut params: CertificateParams = CertificateParams::new(config.dns_entries().unwrap()).unwrap();
    params.is_ca = IsCa::NoCa;
    params.distinguished_name = distinguished_name;
    (params.not_before, params.not_after) = not_before_not_after(365);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.serialize_request(&key_pair).unwrap()
  }

  let platform = dsh_api::platform::DshPlatform::new("nplz");
  let acl_group_name = None;
  let proxy_name = "my-proxy".to_string();
  let tenant = "my-tenant".to_string();
  let vhost_zone = dsh_api::platform::VhostZone::Public;
  let ca_common_name = platform.proxy_common_name(&proxy_name, &tenant, vhost_zone.clone())?;

  let config =
    proxy::ProxyCertificateBundleConfig { acl_group_name, ca_common_name, enable_schema_store: false, number_of_dns_records: 3, platform, proxy_name, tenant, vhost_zone };

  let key_pair = generate_key_pair().unwrap();

  let common_name = "COMMON_NAME";
  let dn1 = kpn_distinguished_name2(common_name, None);
  let dn2 = kpn_distinguished_name2(common_name, None);

  let csr1 = csr(&config, dn1, &key_pair);
  let csr2 = csr(&config, dn2, &key_pair);

  assert_eq!(csr1.pem().unwrap(), csr2.pem().unwrap());
  Ok(())
}
