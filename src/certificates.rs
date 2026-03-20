use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::DshPlatform;
use rcgen::{
  Certificate, CertificateParams, CertificateSigningRequest, CertificateSigningRequestParams, DistinguishedName, DnType, DnValue::PrintableString, ExtendedKeyUsagePurpose, IsCa,
  KeyPair, KeyUsagePurpose, RsaKeySize,
};
use whoami::username;

pub struct DshCertificate {
  pub cert: Certificate,
  pub key: KeyPair,
}

pub struct CertBundle {
  pub server: DshCertificate,
  pub client: DshCertificate,
  pub certificate_authority: DshCertificate,
}

impl CertBundle {
  pub fn from_config(config: &DshCertificateConfig) -> DshCliResult<Self> {
    let certificate_authority = generate_ca_cert()?;
    let server = generate_server_cert(config, &certificate_authority)?;
    let client = generate_client_cert(&config.tenant_name, &certificate_authority)?;
    Ok(CertBundle { server, client, certificate_authority })
  }
}

pub struct ClientBundle {
  pub client: DshCertificate,
  pub certificate_authority: DshCertificate,
}

impl ClientBundle {
  pub fn new(common_name: &str) -> DshCliResult<Self> {
    let certificate_authority = generate_ca_cert()?;
    let client = generate_client_cert(common_name, &certificate_authority)?;
    Ok(ClientBundle { client, certificate_authority })
  }
}

// #[derive(Debug)]
pub struct CsrBundle {
  pub csr: CertificateSigningRequest,
  pub key: KeyPair,
}

impl CsrBundle {
  pub fn from_config(config: &DshCertificateConfig) -> DshCliResult<Self> {
    generate_csr(config)
  }
}

pub fn generate_client_cert(client_id: &str, certificate_authority: &DshCertificate) -> DshCliResult<DshCertificate> {
  let mut params: CertificateParams = CertificateParams::new(vec![client_id.to_string()])?;

  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, client_id);
  dn.push(DnType::CountryName, "NL");
  dn.push(DnType::StateOrProvinceName, "Zuid-Holland");
  dn.push(DnType::LocalityName, "Rotterdam");
  dn.push(DnType::OrganizationName, PrintableString("Koninklijke KPN N.V.".try_into()?));
  params.distinguished_name = dn;

  let now = time::OffsetDateTime::now_utc();
  params.not_before = now;
  params.not_after = now + time::Duration::days(365);

  // key usage attributes
  params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];

  // CSR creation and signing
  let key = generate_private_key()?;
  let csr_pem = params.serialize_request(&key)?.pem()?;

  // BUG: WHY DO WE HAVE TO SPECIFY THIS YET AGAIN, submit an issue to Rcgen
  let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
  csr_params.params.not_before = now;
  csr_params.params.not_after = now + time::Duration::days(365);

  let cert = csr_params.signed_by(&certificate_authority.cert, &certificate_authority.key)?;

  Ok(DshCertificate { cert, key })
}

pub fn generate_server_cert(config: &DshCertificateConfig, certificate_authority: &DshCertificate) -> DshCliResult<DshCertificate> {
  let mut certificate_params: CertificateParams = CertificateParams::new(config.dns_array())?;
  let mut distinguished_name = DistinguishedName::new();
  distinguished_name.push(DnType::CommonName, config.cn());
  distinguished_name.push(DnType::CountryName, "NL");
  distinguished_name.push(DnType::StateOrProvinceName, "Zuid-Holland");
  distinguished_name.push(DnType::LocalityName, "Rotterdam");
  distinguished_name.push(DnType::OrganizationName, PrintableString("Koninklijke KPN N.V.".try_into()?));
  certificate_params.distinguished_name = distinguished_name;

  let now = time::OffsetDateTime::now_utc();
  certificate_params.not_before = now;
  certificate_params.not_after = now + time::Duration::days(365);

  // key usage attributes
  certificate_params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
  certificate_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

  // CSR creation and signing
  let key = generate_private_key()?;
  let csr_pem = certificate_params.serialize_request(&key)?.pem()?;

  // BUG: WHY DO WE HAVE TO SPECIFY THIS YET AGAIN
  let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
  csr_params.params.not_before = now;
  csr_params.params.not_after = now + time::Duration::days(365);

  let cert = csr_params.signed_by(&certificate_authority.cert, &certificate_authority.key)?;
  // let cert = CertificateSigningRequestParams::signed_by(csr_params, &ca.cert, &ca.key)?;

  Ok(DshCertificate { cert, key })
}

pub fn generate_ca_cert() -> DshCliResult<DshCertificate> {
  // for CA we attach the machines user id as CN
  let user = username()?;

  let mut params: CertificateParams = CertificateParams::new(vec![user.clone()])?;
  params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, user);
  dn.push(DnType::CountryName, "NL");
  dn.push(DnType::StateOrProvinceName, "Zuid-Holland");
  dn.push(DnType::LocalityName, "Rotterdam");
  dn.push(DnType::OrganizationName, PrintableString("Koninklijke KPN N.V.".try_into()?));
  params.distinguished_name = dn;
  let now = time::OffsetDateTime::now_utc();
  params.not_before = now;
  params.not_after = now + time::Duration::days(365);

  params.use_authority_key_identifier_extension = true;

  // key usages
  params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign, KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth, ExtendedKeyUsagePurpose::ClientAuth];

  let key = KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, RsaKeySize::_4096)?;
  let cert = params.self_signed(&key)?;

  Ok(DshCertificate { cert, key })
}

// FIX: critical EKU attributes
pub fn generate_csr(config: &DshCertificateConfig) -> DshCliResult<CsrBundle> {
  let mut params: CertificateParams = CertificateParams::new(config.dns_array())?;
  params.is_ca = IsCa::NoCa;

  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, config.cn());
  dn.push(DnType::CountryName, "NL");
  dn.push(DnType::StateOrProvinceName, "Zuid-Holland");
  dn.push(DnType::LocalityName, "Rotterdam");
  dn.push(DnType::OrganizationName, PrintableString("Koninklijke KPN N.V.".try_into()?));
  params.distinguished_name = dn;

  let now = time::OffsetDateTime::now_utc();
  params.not_before = now;
  params.not_after = now + time::Duration::days(365);

  // key usage attributes
  params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
  params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

  // CSR creation and signing
  let key = generate_private_key()?;
  let csr = params.serialize_request(&key)?;

  Ok(CsrBundle { csr, key })
}

// used by both Cert and CSR command
fn generate_private_key() -> DshCliResult<KeyPair> {
  KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, RsaKeySize::_4096).map_err(DshCliError::from)
}

#[derive(Debug, Clone)]
pub struct DshCertificateConfig {
  pub platform: DshPlatform,
  pub tenant_name: String,
  pub broker_prefix: String,
  pub number_of_brokers: u32,
  pub private_domain: bool,
  pub include_schema_store_dns_record: bool,
}

impl DshCertificateConfig {
  pub fn dns_array(&self) -> Vec<String> {
    let mut dns_array: Vec<String> = vec![];
    // when signing a CSR with the KPN CA we are limited by a maximum of 10 DNS records, here
    // we ensure that if schema store is enabled we leave enough room to comply
    let broker_amount = if self.include_schema_store_dns_record && self.number_of_brokers >= 10 { 9 } else { self.number_of_brokers };

    for i in 0..broker_amount {
      dns_array.push(self.format_dns(i));
    }

    if self.include_schema_store_dns_record {
      dns_array.push(format!(
        "{}-schema-store.kafka.{}.{}",
        self.broker_prefix,
        self.tenant_name,
        if self.private_domain { self.platform.private_domain().unwrap_or_default() } else { self.platform.public_domain() }
      ));
    }

    dns_array
  }

  pub fn format_dns(&self, index: u32) -> String {
    format!(
      "{}-{}.kafka.{}.{}",
      self.broker_prefix,
      index,
      self.tenant_name,
      if self.private_domain { self.platform.private_domain().unwrap_or_default() } else { self.platform.public_domain() }
    )
  }

  // we make sure that the CN has the same value as the first SAN so that we don't 'waste' a DNS
  // record on a unique name
  pub fn cn(&self) -> String {
    self.format_dns(0)
  }
}

/// `cert`-specific flags
#[derive(Clone, Debug)]
pub struct CertConfig {
  /// All the common flags
  pub common: DshCertificateConfig,

  /// Deploy Kafka Proxy with API key.
  /// Will fallback to `DSH_REST_KEY` If no input is provided
  pub deploy: Option<Option<String>>,

  /// Overwrite files in output directory
  pub overwrite: bool,

  /// Don't persist any output files
  pub no_write: bool,
}

/// `csr`-specific flags
#[derive(Debug)]
pub struct CsrConfig {
  /// All the common flags
  pub common: DshCertificateConfig,

  /// Overwrite files in output directory
  pub overwrite: bool,
}

/// `deploy`-specific flags
#[derive(Debug, Clone)]
pub struct DeployConfig {
  /// Path to TLS certificate (PEM)
  pub cert: String,

  /// Path to private key (PEM)
  pub key: String,

  /// Path to certificate chain / truststore (PEM)
  pub chain: String,

  /// Tenant REST API key.
  /// Will fallback to `DSH_REST_KEY` If no input is provided
  pub api_key: Option<Option<String>>,
}

/// `validate`-specific flags
#[derive(Debug, Clone)]
pub struct ValidateConfig {
  pub tenant_name: String,
  pub platform: DshPlatform,

  /// Tenant REST API key.
  pub api_key: Option<Option<String>>,
}

/// `replace`-specific flags
#[derive(Debug, Clone)]
pub struct ReplaceConfig {
  /// Path to TLS certificate (PEM)
  pub cert: String,

  /// Path to private key (PEM)
  pub key: String,

  /// Tenant REST API key.
  /// Will fallback to `$DSH_REST_KEY` If no input is provided
  pub api_key: Option<Option<String>>,
}
