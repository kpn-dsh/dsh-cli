use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::{deserialize_platform, serialize_platform};
use dsh_api::platform::{DshPlatform, VhostZone};
use rcgen::{
  Certificate as RcgenCertificate, CertificateParams, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue::PrintableString, ExtendedKeyUsagePurpose, IsCa, KeyPair,
  KeyUsagePurpose, RsaKeySize,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use time::OffsetDateTime;

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Validation {
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

/// Contains configuration for proxy certificate bundle.
///
/// * `ca_common_name` - Certificate authority common name.
/// * `enable_schema_store` - Indicates whether a schema store dns record must be
///   included.
/// * `kafka_acl_group` - Optional configuration for Kafka ACL group.
/// * `number_of_dns_records` - Number of dns records.
/// * `platform` - Platform.
/// * `proxy_name` - Proxy name.
/// * `tenant: String` - Tenant.
/// * `vhost_zone` - Public or private.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct ProxyCertificateBundleConfig {
  #[serde(rename = "acl-group-name", skip_serializing_if = "Option::is_none")]
  pub(crate) acl_group_name: Option<String>,
  #[serde(rename = "ca-common-name")]
  pub(crate) ca_common_name: String,
  #[serde(rename = "enable-schema-store")]
  pub(crate) enable_schema_store: bool,
  #[serde(rename = "number_of_dns_records")]
  pub(crate) number_of_dns_records: usize,
  #[serde(deserialize_with = "deserialize_platform", serialize_with = "serialize_platform")]
  pub(crate) platform: DshPlatform,
  #[serde(rename = "proxy-name")]
  pub(crate) proxy_name: String,
  pub(crate) tenant: String,
  #[serde(rename = "vhost-zone")]
  pub(crate) vhost_zone: VhostZone,
}

/// Contains certificate and key-pair.
///
/// * `certificate: Certificate`
/// * `key_pair: KeyPair`
pub(crate) struct DshCertificate {
  pub certificate: RcgenCertificate,
  pub key_pair: KeyPair,
}

/// Contains proxy certificate bundle and configuration.
///
/// * `config: ProxyCertificateBundleConfig`
/// * `ca_certificate: DshCertificate`
/// * `client_certificate: DshCertificate`
/// * `server_certificate: DshCertificate`
#[derive(Debug)]
pub(crate) struct ProxyCertificateBundle {
  pub config: ProxyCertificateBundleConfig,
  pub ca_certificate: DshCertificate,
  pub client_certificate: DshCertificate,
  pub server_certificate: DshCertificate,
}

pub(crate) struct LocalCertificateBundle {
  pub(crate) configuration: (ProxyCertificateBundleConfig, String),
  pub(crate) ca_key: LocalCertificate,
  pub(crate) ca_pem: LocalCertificate,
  pub(crate) client_key: LocalCertificate,
  pub(crate) client_pem: LocalCertificate,
  pub(crate) server_key: LocalCertificate,
  pub(crate) server_pem: LocalCertificate,
}

pub(crate) struct LocalCertificate {
  pub(crate) value: String,
  pub(crate) filename: String,
}

impl Debug for ProxyCertificateBundleConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("DshCertificateConfig");
    builder.field("acl_group_name", &self.acl_group_name);
    builder.field("ca_common_name", &self.ca_common_name);
    builder.field("enable_schema_store", &self.enable_schema_store);
    builder.field("number_of_dns_records", &self.number_of_dns_records);
    builder.field("platform", &self.platform.name());
    builder.field("proxy_name", &self.proxy_name);
    builder.field("tenant", &self.tenant);
    builder.field("vhost_zone", &self.vhost_zone);
    builder.finish()
  }
}

impl ProxyCertificateBundleConfig {
  pub(crate) fn client_id(&self) -> String {
    self.tenant.clone()
  }

  // Make sure that the CN has the same value as the first SAN so that we don't 'waste' a DNS
  // record on a unique name.
  pub(crate) fn common_name(&self) -> DshCliResult<String> {
    Ok(self.platform.proxy_vhost(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), 0)?)
  }

  pub(crate) fn domain_from_platform(&self) -> DshCliResult<String> {
    match self.vhost_zone {
      VhostZone::Private => match &self.platform.private_domain() {
        Some(private_domain) => Ok(private_domain.to_string()),
        None => Err(DshCliError::Configuration(format!("platform '{}' does not support private vhosts", &self.platform))),
      },
      VhostZone::Public => Ok(self.platform.public_domain().to_string()),
    }
  }

  pub(crate) fn dns_entries(&self) -> DshCliResult<Vec<String>> {
    let mut dns_entries: Vec<String> = vec![];
    if self.enable_schema_store {
      for index in 0..usize::min(self.number_of_dns_records, 9) {
        dns_entries.push(self.platform.proxy_vhost(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), index)?);
      }
      dns_entries.push(self.platform.proxy_schema_store_vhost(&self.tenant, &self.proxy_name, self.vhost_zone.clone())?);
    } else {
      for index in 0..self.number_of_dns_records {
        dns_entries.push(self.platform.proxy_vhost(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), index)?);
      }
    }
    Ok(dns_entries)
  }

  pub(crate) fn group_id(&self, index: usize) -> String {
    match &self.acl_group_name {
      Some(acl_group_name) => self.platform.proxy_consumer_group_acl(&self.tenant, &self.proxy_name, acl_group_name, index),
      None => self.platform.proxy_consumer_group(&self.tenant, &self.proxy_name, index),
    }
  }
}

impl TryFrom<ProxyCertificateBundleConfig> for ProxyCertificateBundle {
  type Error = DshCliError;

  fn try_from(config: ProxyCertificateBundleConfig) -> DshCliResult<Self> {
    let ca_certificate = generate_ca_certificate(&config)?;
    let client_certificate = generate_client_certificate(&config, &ca_certificate)?;
    let server_certificate = generate_server_certificate(&config, &ca_certificate)?;
    Ok(ProxyCertificateBundle { config, ca_certificate, client_certificate, server_certificate })
  }
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
/// * Subject alt names: CA common name from proxy bundle configuration (`config.ca_common_name`).
/// * Distinguished name:
///   * `CN` - CA common name from proxy bundle configuration (`config.ca_common_name`).
///   * `O` - `"Koninklijke KPN N.V."`
///   * `L` - `"Rotterdam"`
///   * `ST` - `"Zuid-Holland"`
///   * `C` - `"NL"`
///
/// # Parameters
/// * `config` - Proxy certificate bundle configuration:
///   * `common_name`
fn generate_ca_certificate(config: &ProxyCertificateBundleConfig) -> DshCliResult<DshCertificate> {
  let distinguished_name = kpn_distinguished_name(&config.ca_common_name, None);
  let mut params: CertificateParams = CertificateParams::new(vec![config.ca_common_name.clone()])?;
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
/// * `config` - Proxy certificate bundle configuration:
///   * `config.client_id`
///   * `config.organizational_unit_name`
/// * `ca_certificate` - Certificate authority certificate used to sign the generated certificate.
fn generate_client_certificate(config: &ProxyCertificateBundleConfig, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate> {
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(vec![])?;
  params.distinguished_name = kpn_distinguished_name(&config.client_id(), config.acl_group_name.clone());
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
/// * `config` - Proxy certificate bundle configuration:
///   * `config.dns_entries`
///   * `common_name`
/// * `ca_certificate` - Certificate authority certificate used to sign the generated certificate.
fn generate_server_certificate(config: &ProxyCertificateBundleConfig, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate> {
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(config.dns_entries()?)?;
  params.distinguished_name = kpn_distinguished_name(&config.common_name()?, None);
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
/// * `common_name`
/// * `organizational_unit_name`
fn kpn_distinguished_name(common_name: &str, organizational_unit_name: Option<String>) -> DistinguishedName {
  let mut distinguished_name = DistinguishedName::new();

  distinguished_name.push(DnType::CommonName, common_name);
  distinguished_name.push(DnType::OrganizationName, KPN_ORGANIZATION_NAME);
  if let Some(organizational_unit_name) = organizational_unit_name {
    distinguished_name.push(DnType::OrganizationalUnitName, organizational_unit_name);
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
  let dn = kpn_distinguished_name(common_name, None);
  let dn2 = kpn_distinguished_name2(common_name, None);
  assert_eq!(dn, dn2);
}

#[test]
fn test2() -> Result<(), Box<dyn std::error::Error>> {
  fn csr(config: &ProxyCertificateBundleConfig, distinguished_name: DistinguishedName, key_pair: &KeyPair) -> rcgen::CertificateSigningRequest {
    let mut params: CertificateParams = CertificateParams::new(config.dns_entries().unwrap()).unwrap();
    params.is_ca = IsCa::NoCa;
    params.distinguished_name = distinguished_name;
    (params.not_before, params.not_after) = not_before_not_after(365);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.serialize_request(&key_pair).unwrap()
  }

  let platform = DshPlatform::new("nplz");
  let acl_group_name = None;
  let proxy_name = "my-proxy".to_string();
  let tenant = "my-tenant".to_string();
  let vhost_zone = VhostZone::Public;
  let ca_common_name = platform.proxy_common_name(&proxy_name, &tenant, vhost_zone.clone())?;

  let config = ProxyCertificateBundleConfig { acl_group_name, ca_common_name, enable_schema_store: false, number_of_dns_records: 3, platform, proxy_name, tenant, vhost_zone };

  let key_pair = generate_key_pair().unwrap();

  let common_name = "COMMON_NAME";
  let dn1 = kpn_distinguished_name2(common_name, None);
  let dn2 = kpn_distinguished_name2(common_name, None);

  let csr1 = csr(&config, dn1, &key_pair);
  let csr2 = csr(&config, dn2, &key_pair);

  assert_eq!(csr1.pem().unwrap(), csr2.pem().unwrap());
  Ok(())
}
