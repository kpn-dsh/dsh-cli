use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::DshPlatform;
use rcgen::{
  Certificate, CertificateParams, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue, DnValue::PrintableString, ExtendedKeyUsagePurpose, IsCa, KeyPair,
  KeyUsagePurpose, OtherNameValue, RsaKeySize, SanType,
};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use time::OffsetDateTime;
use whoami::username;

const KPN_ORGANIZATION_NAME: &str = "Koninklijke KPN N.V.";
const KPN_LOCALITY_NAME: &str = "Rotterdam";
const KPN_STATE_OR_PROVINCE_NAME: &str = "Zuid-Holland";
const KPN_COUNTRY_NAME: &str = "NL";

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

#[derive(Debug)]
pub(crate) struct ProxyCertificateBundle {
  pub ca_certificate: DshCertificate,
  pub client_certificate: DshCertificate,
  pub server_certificate: DshCertificate,
}

impl TryFrom<&ProxyCertificateBundleConfig> for ProxyCertificateBundle {
  type Error = DshCliError;

  fn try_from(config: &ProxyCertificateBundleConfig) -> DshCliResult<Self> {
    let ca_certificate = generate_ca_certificate()?;
    let client_certificate = generate_client_certificate(&config.tenant, &ca_certificate)?;
    let server_certificate = generate_server_certificate(config, &ca_certificate)?;
    Ok(ProxyCertificateBundle { ca_certificate, client_certificate, server_certificate })
  }
}

// pub(crate) struct ClientBundle {
//   pub ca_certificate: DshCertificate,
//   pub client_certificate: DshCertificate,
// }

// impl ClientBundle {
//   pub(crate) fn new(common_name: &str) -> DshCliResult<Self> {
//     let ca_certificate = generate_ca_certificate()?;
//     let client_certificate = generate_client_certificate(common_name, &ca_certificate)?;
//     Ok(ClientBundle { ca_certificate, client_certificate })
//   }
// }

// pub(crate) struct CsrBundle {
//   pub csr: CertificateSigningRequest,
//   pub key_pair: KeyPair,
// }

// impl TryFrom<&ProxyCertificateBundleConfig> for CsrBundle {
//   type Error = DshCliError;
//
//   fn try_from(config: &ProxyCertificateBundleConfig) -> DshCliResult<Self> {
//     generate_csr(config)
//   }
// }

/// Generate self-signed certificate authority certificate
///
/// The username is used as the common name.
fn generate_ca_certificate() -> DshCliResult<DshCertificate> {
  let user = username()?;
  let distinguished_name = distinguished_name(&user);
  let mut params: CertificateParams = CertificateParams::new(vec![user])?;
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

fn generate_client_certificate(client_id: &str, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate> {
  let not_before_not_after = not_before_not_after(365);
  let mut params: CertificateParams = CertificateParams::new(vec![client_id.to_string()])?;
  params.distinguished_name = distinguished_name(client_id);
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
  let mut params: CertificateParams = CertificateParams::new(config.dns_array())?;
  params.distinguished_name = distinguished_name(&config.cn());
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

// // FIX: critical EKU attributes
// pub(crate) fn generate_csr(config: &ProxyCertificateBundleConfig) -> DshCliResult<CsrBundle> {
//   let mut params: CertificateParams = CertificateParams::new(config.dns_array())?;
//   params.is_ca = IsCa::NoCa;
//   params.distinguished_name = distinguished_name(&config.cn());
//   (params.not_before, params.not_after) = not_before_not_after(365);
//   params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
//   params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
//   let key_pair = generate_key_pair()?;
//   let csr = params.serialize_request(&key_pair)?;
//
//   Ok(CsrBundle { csr, key_pair })
// }

fn not_before_not_after(days: i64) -> (OffsetDateTime, OffsetDateTime) {
  let now = time::OffsetDateTime::now_utc();
  (now, now + time::Duration::days(days))
}

fn distinguished_name(common_name: &str) -> DistinguishedName {
  let mut distinguished_name = DistinguishedName::new();
  distinguished_name.push(DnType::CommonName, common_name);
  distinguished_name.push(DnType::OrganizationName, KPN_ORGANIZATION_NAME);
  distinguished_name.push(DnType::LocalityName, KPN_LOCALITY_NAME);
  distinguished_name.push(DnType::StateOrProvinceName, KPN_STATE_OR_PROVINCE_NAME);
  distinguished_name.push(DnType::CountryName, KPN_COUNTRY_NAME);
  distinguished_name
}

#[allow(dead_code)] // TODO
fn distinguished_name2(common_name: &str) -> DistinguishedName {
  let mut distinguished_name = DistinguishedName::new();
  distinguished_name.push(DnType::CommonName, common_name);
  distinguished_name.push(DnType::OrganizationName, PrintableString(KPN_ORGANIZATION_NAME.try_into().unwrap()));
  distinguished_name.push(DnType::LocalityName, KPN_LOCALITY_NAME);
  distinguished_name.push(DnType::StateOrProvinceName, KPN_STATE_OR_PROVINCE_NAME);
  distinguished_name.push(DnType::CountryName, KPN_COUNTRY_NAME);
  distinguished_name
}

fn generate_key_pair() -> DshCliResult<KeyPair> {
  KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, RsaKeySize::_4096).map_err(DshCliError::from)
}

#[derive(Clone)]
pub(crate) struct ProxyCertificateBundleConfig {
  pub platform: DshPlatform,
  pub tenant: String,
  pub broker_prefix: String,
  pub number_of_brokers: u32,
  pub public_vhost: bool,
  pub include_schema_store_dns_record: bool,
}

impl Debug for ProxyCertificateBundleConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("DshCertificateConfig");
    builder.field("platform", &self.platform.name());
    builder.field("tenant", &self.tenant);
    builder.field("broker_prefix", &self.broker_prefix);
    builder.field("number_of_brokers", &self.number_of_brokers);
    builder.field("public_vhost", &self.public_vhost);
    builder.field("include_schema_store_dns_record", &self.include_schema_store_dns_record);
    builder.finish()
  }
}

impl ProxyCertificateBundleConfig {
  pub(crate) fn dns_array(&self) -> Vec<String> {
    let mut dns_array: Vec<String> = vec![];
    let broker_amount = if self.include_schema_store_dns_record && self.number_of_brokers >= 10 { 9 } else { self.number_of_brokers };
    for i in 0..broker_amount {
      dns_array.push(self.format_dns(i));
    }
    if self.include_schema_store_dns_record {
      dns_array.push(format!(
        "{}-schema-store.kafka.{}.{}",
        self.broker_prefix,
        self.tenant,
        if self.public_vhost { self.platform.public_domain() } else { self.platform.private_domain().unwrap_or_default() }
      ));
    }
    dns_array
  }

  pub(crate) fn format_dns(&self, index: u32) -> String {
    format!(
      "{}-{}.kafka.{}.{}",
      self.broker_prefix,
      index,
      self.tenant,
      if self.public_vhost { self.platform.public_domain() } else { self.platform.private_domain().unwrap_or_default() }
    )
  }

  // Make sure that the CN has the same value as the first SAN so that we don't 'waste' a DNS
  // record on a unique name.
  pub(crate) fn cn(&self) -> String {
    self.format_dns(0)
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
  let dn = distinguished_name(common_name);
  let dn2 = distinguished_name2(common_name);
  assert_eq!(dn, dn2);
}

#[test]
fn test2() {
  fn csr(config: &ProxyCertificateBundleConfig, dn: DistinguishedName, key_pair: &KeyPair) -> rcgen::CertificateSigningRequest {
    let mut params: CertificateParams = CertificateParams::new(config.dns_array()).unwrap();
    params.is_ca = IsCa::NoCa;
    params.distinguished_name = dn;
    // params.distinguished_name = distinguished_name(&config.cn());
    (params.not_before, params.not_after) = not_before_not_after(365);
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.serialize_request(&key_pair).unwrap()
  }

  let config = ProxyCertificateBundleConfig {
    platform: DshPlatform::new("nplz"),
    tenant: "test-tenant".to_string(),
    broker_prefix: "broooker".to_string(),
    number_of_brokers: 3,
    public_vhost: false,
    include_schema_store_dns_record: false,
  };

  let key_pair = generate_key_pair().unwrap();

  let common_name = "COMMON_NAME";
  let dn1 = distinguished_name2(common_name);
  let dn2 = distinguished_name2(common_name);

  let csr1 = csr(&config, dn1, &key_pair);
  let csr2 = csr(&config, dn2, &key_pair);

  assert_eq!(csr1.pem().unwrap(), csr2.pem().unwrap());
}
