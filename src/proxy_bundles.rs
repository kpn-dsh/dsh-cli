use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::{DshPlatform, VhostZone};
use rcgen::{
  Certificate, CertificateParams, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue, DnValue::PrintableString, ExtendedKeyUsagePurpose, IsCa, KeyPair,
  KeyUsagePurpose, OtherNameValue, RsaKeySize, SanType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use time::OffsetDateTime;

use dsh_api::platform::{deserialize_platform, serialize_platform};

/// Contains certificate and key-pair.
///
/// * `certificate: Certificate`
/// * `key_pair: KeyPair`
pub(crate) struct DshCertificate {
  pub certificate: Certificate,
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

impl TryFrom<ProxyCertificateBundleConfig> for ProxyCertificateBundle {
  type Error = DshCliError;

  fn try_from(config: ProxyCertificateBundleConfig) -> DshCliResult<Self> {
    domain_from_platform(&config.vhost_zone, &config.platform)?;
    let ca_certificate = generate_ca_certificate(config.ca_common_name.clone())?;
    let client_certificate = generate_client_certificate(&config.tenant, config.acl_group_id.clone(), &ca_certificate)?;
    let server_certificate = generate_server_certificate(&config, &ca_certificate)?;
    Ok(ProxyCertificateBundle { config, ca_certificate, client_certificate, server_certificate })
  }
}

pub(crate) struct LocalCertificate {
  pub(crate) value: String,
  pub(crate) filename: String,
}

pub(crate) struct LocalCertificateBundle {
  pub(crate) configuration: (ProxyCertificateBundleConfig, String),
  pub(crate) _ca_key: LocalCertificate,
  pub(crate) ca_pem: LocalCertificate,
  pub(crate) client_key: LocalCertificate,
  pub(crate) _client_pem: LocalCertificate,
  pub(crate) _server_key: LocalCertificate,
  pub(crate) server_pem: LocalCertificate,
}

/// Generates self-signed certificate authority certificate.
///
/// # Parameters
/// * `username` - Used as the common name.
fn generate_ca_certificate(username: String) -> DshCliResult<DshCertificate> {
  let distinguished_name = kpn_distinguished_name(&username, None);
  let mut params: CertificateParams = CertificateParams::new(vec![username])?;
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

fn generate_client_certificate(client_id: &str, organizational_unit_name: Option<String>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate> {
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(vec![client_id.to_string()])?;
  params.distinguished_name = kpn_distinguished_name(client_id, organizational_unit_name);
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

fn generate_server_certificate(config: &ProxyCertificateBundleConfig, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate> {
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(config.dns_array()?)?;
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

/// Contains configuration for proxy certificate bundle.
///
/// * `acl_group_id` - Optional ACL group identifier.
/// * `broker_prefix` - Prefix for dns names.
/// * `ca_common_name` - Certificate authority common name.
/// * `include_schema_store_dns_record` - Indicates whether a schema store dns record must be
///   included.
/// * `number_of_dns_records` - Number of dns records.
/// * `platform` - Platform.
/// * `tenant: String` - Tenant.
/// * `vhost_zone` - Public or private..
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct ProxyCertificateBundleConfig {
  #[serde(rename = "acl-group-id")]
  pub acl_group_id: Option<String>,
  #[serde(rename = "broker-prefix")]
  pub broker_prefix: String,
  #[serde(rename = "ca-common-name")]
  pub ca_common_name: String,
  #[serde(rename = "include-schema-store-dns-record")]
  pub include_schema_store_dns_record: bool,
  #[serde(rename = "number_of_dns_records")]
  pub number_of_dns_records: usize,
  #[serde(deserialize_with = "deserialize_platform", serialize_with = "serialize_platform")]
  pub platform: DshPlatform,
  pub tenant: String,
  #[serde(rename = "vhost-zone")]
  pub vhost_zone: VhostZone,
}

impl Debug for ProxyCertificateBundleConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("DshCertificateConfig");
    builder.field("broker_prefix", &self.broker_prefix);
    builder.field("ca_common_name", &self.ca_common_name);
    builder.field("include_schema_store_dns_record", &self.include_schema_store_dns_record);
    builder.field("number_of_dns_records", &self.number_of_dns_records);
    builder.field("platform", &self.platform.name());
    builder.field("tenant", &self.tenant);
    builder.field("vhost_zone", &self.vhost_zone);
    builder.finish()
  }
}

impl ProxyCertificateBundleConfig {
  pub(crate) fn dns_array(&self) -> DshCliResult<Vec<String>> {
    let mut dns_array: Vec<String> = vec![];
    if self.include_schema_store_dns_record {
      for index in 0..usize::min(self.number_of_dns_records, 9) {
        dns_array.push(
          self
            .platform
            .proxy_broker_vhost(&self.broker_prefix, &self.tenant, self.vhost_zone.clone(), index)?,
        );
      }
      dns_array.push(self.platform.proxy_schema_store_vhost(&self.broker_prefix, &self.tenant, self.vhost_zone.clone())?);
    } else {
      for index in 0..self.number_of_dns_records {
        dns_array.push(
          self
            .platform
            .proxy_broker_vhost(&self.broker_prefix, &self.tenant, self.vhost_zone.clone(), index)?,
        );
      }
    }
    Ok(dns_array)
  }

  // Make sure that the CN has the same value as the first SAN so that we don't 'waste' a DNS
  // record on a unique name.
  fn common_name(&self) -> DshCliResult<String> {
    Ok(self.platform.proxy_broker_vhost(&self.broker_prefix, &self.tenant, self.vhost_zone.clone(), 0)?)
  }
}

fn domain_from_platform<'a>(vhost_zone: &VhostZone, platform: &'a DshPlatform) -> DshCliResult<&'a str> {
  match vhost_zone {
    VhostZone::Private => match platform.private_domain() {
      Some(private_domain) => Ok(private_domain),
      None => Err(DshCliError::Configuration(format!("platform '{}' does not support private vhosts", platform))),
    },
    VhostZone::Public => Ok(platform.public_domain()),
  }
}

pub(crate) fn san_to_string(san_type: &SanType) -> String {
  match san_type {
    SanType::Rfc822Name(rfc822) => format!("rfc822: {}", rfc822),
    SanType::DnsName(dns_name) => format!("dns: {}", dns_name),
    SanType::URI(uri) => format!("uri: {}", uri),
    SanType::IpAddress(ip_addr) => format!("ip address: {}", ip_addr),
    SanType::OtherName((_, OtherNameValue::Utf8String(utf8_string))) => format!("utf8: {}", utf8_string),
    _ => "".to_string(),
  }
}

pub(crate) fn hashmap_from_distinguished_name(distinguished_name: &DistinguishedName) -> HashMap<String, String> {
  distinguished_name
    .iter()
    .map(|(dn_type, dn_value)| (dn_type_name(dn_type).to_string(), dn_value_string(dn_value)))
    .collect::<HashMap<_, _>>()
}

fn dn_type_name(dn_type: &DnType) -> &'static str {
  match dn_type {
    DnType::CountryName => "C",
    DnType::LocalityName => "L",
    DnType::StateOrProvinceName => "ST",
    DnType::OrganizationName => "O",
    DnType::OrganizationalUnitName => "OU",
    DnType::CommonName => "CN",
    DnType::CustomDnType(_) => "",
    _ => "",
  }
}

fn dn_value_string(dn_value: &DnValue) -> String {
  match dn_value {
    DnValue::Ia5String(ia5_string) => ia5_string.to_string(),
    DnValue::PrintableString(printable_string) => printable_string.to_string(),
    DnValue::TeletexString(teletex_string) => teletex_string.to_string(),
    DnValue::Utf8String(utf8_string) => utf8_string.to_string(),
    _ => "".to_string(),
  }
}

#[test]
fn test() {
  let common_name = "COMMON_NAME";
  let dn = kpn_distinguished_name(common_name, None);
  let dn2 = kpn_distinguished_name2(common_name, None);
  assert_eq!(dn, dn2);
}

#[test]
fn test2() {
  fn csr(config: &ProxyCertificateBundleConfig, distinguished_name: DistinguishedName, key_pair: &KeyPair) -> rcgen::CertificateSigningRequest {
    let mut params: CertificateParams = CertificateParams::new(config.dns_array().unwrap()).unwrap();
    params.is_ca = IsCa::NoCa;
    params.distinguished_name = distinguished_name;
    // params.distinguished_name = distinguished_name(&config.cn());
    (params.not_before, params.not_after) = not_before_not_after(365);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.serialize_request(&key_pair).unwrap()
  }

  let config = ProxyCertificateBundleConfig {
    acl_group_id: None,
    broker_prefix: "broooker".to_string(),
    ca_common_name: "username".to_string(),
    include_schema_store_dns_record: false,
    number_of_dns_records: 3,
    platform: DshPlatform::new("nplz"),
    tenant: "test-tenant".to_string(),
    vhost_zone: VhostZone::Public,
  };

  let key_pair = generate_key_pair().unwrap();

  let common_name = "COMMON_NAME";
  let dn1 = kpn_distinguished_name2(common_name, None);
  let dn2 = kpn_distinguished_name2(common_name, None);

  let csr1 = csr(&config, dn1, &key_pair);
  let csr2 = csr(&config, dn2, &key_pair);

  assert_eq!(csr1.pem().unwrap(), csr2.pem().unwrap());
}
