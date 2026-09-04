use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::error::DshCliError;
use crate::{err, DshCliResult};
use rcgen::{
  Certificate, CertificateParams, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, RsaKeySize,
};
use std::fmt::{Debug, Formatter};
use time::OffsetDateTime;

/// Contains proxy certificate bundle and configuration for self-signed certificates.
///
/// * `config: ProxyCertificateBundleConfig`
/// * `ca_certificate: DshCertificate`
/// * `client_certificate: DshCertificate`
/// * `server_certificate: DshCertificate`
#[derive(Debug)]
pub(crate) struct ProxySelfSignedCertificateBundle {
  pub config: ProxyCertificateBundleConfig,
  pub ca_certificate: DshCertificate,
  pub client_certificate: DshCertificate,
  pub server_certificate: DshCertificate,
}

/// Contains certificate and key-pair.
///
/// * `certificate: rcgen::Certificate`
/// * `key_pair: rcgen::KeyPair`
pub(crate) struct DshCertificate {
  pub certificate: Certificate,
  pub key_pair: KeyPair,
}

impl ProxySelfSignedCertificateBundle {
  /// Create proxy certificate bundle with self-signed certificates.
  ///
  /// # Parameters
  /// * `config` - Proxy certificate bundle configuration.
  pub(crate) fn create_self_signed(config: ProxyCertificateBundleConfig) -> DshCliResult<Self> {
    generate_self_signed_certificate_bundle(config)
  }
}

/// Create self-signed proxy certificate bundle.
///
/// Creates a proxy certificate bundle with self-signed certificates.
///
/// # Parameters
/// * `config` - Proxy certificate bundle configuration.
pub(crate) fn generate_self_signed_certificate_bundle(config: ProxyCertificateBundleConfig) -> DshCliResult<ProxySelfSignedCertificateBundle> {
  match &config.ca_common_name {
    Some(ca_common_name) => {
      let ca_certificate = generate_ca_certificate(ca_common_name)?;
      let client_certificate = generate_client_certificate(config.client_id(), config.acl_group_name.clone(), &ca_certificate)?;
      let server_certificate = generate_server_certificate(&config.server_common_name()?, config.dns_entries()?, &ca_certificate)?;
      Ok(ProxySelfSignedCertificateBundle { config, ca_certificate, client_certificate, server_certificate })
    }
    None => err!("certificate common name not specified"),
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
pub(crate) fn generate_client_certificate<S, T>(client_common_name: S, organizational_unit_name: Option<T>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate>
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

fn not_before_not_after(days: i64) -> (OffsetDateTime, OffsetDateTime) {
  let now = time::OffsetDateTime::now_utc();
  (now, now + time::Duration::days(days))
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
pub(crate) fn generate_server_certificate<T>(server_common_name: T, subject_alt_names: impl Into<Vec<String>>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate>
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

fn generate_key_pair() -> DshCliResult<KeyPair> {
  KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, RsaKeySize::_4096).map_err(DshCliError::from)
}

impl Debug for DshCertificate {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("DshCertificate");
    builder.field("key_pair", &self.key_pair);
    builder.field("certificate", self.certificate.params());
    builder.finish()
  }
}
