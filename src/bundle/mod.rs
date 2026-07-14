pub(crate) mod proxy;
pub(crate) mod vhost;

use crate::error::DshCliError;
use crate::DshCliResult;
use rcgen::{
  Certificate as RcgenCertificate, CertificateParams, CertificateSigningRequest, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue, DnValue::PrintableString,
  ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, RsaKeySize,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use time::OffsetDateTime;

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

/// Contains certificate signing request and key-pair.
///
/// * `csr` - Certificate signing request.
/// * `key_pair` - Public and private key pair
pub(crate) struct _DshCsr {
  pub csr: CertificateSigningRequest,
  pub key_pair: KeyPair,
}

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

pub fn _generate_csr<T>(common_name: T, subject_alt_names: impl Into<Vec<String>>) -> DshCliResult<_DshCsr>
where
  T: Into<DnValue>,
{
  let mut params: CertificateParams = CertificateParams::new(subject_alt_names)?;
  params.is_ca = IsCa::NoCa;

  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, common_name);
  dn.push(DnType::CountryName, "NL");
  dn.push(DnType::StateOrProvinceName, "Zuid-Holland");
  dn.push(DnType::LocalityName, "Rotterdam");
  dn.push(DnType::OrganizationName, PrintableString("Koninklijke KPN N.V.".try_into()?));
  params.distinguished_name = dn;

  let now = time::OffsetDateTime::now_utc();
  params.not_before = now;
  params.not_after = now + time::Duration::days(365);

  params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

  let key_pair = generate_key_pair()?;
  let csr = params.serialize_request(&key_pair)?;

  Ok(_DshCsr { csr, key_pair })
}

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
