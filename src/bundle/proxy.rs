use crate::bundle::self_signing::{generate_ca_certificate, generate_client_certificate, generate_server_certificate};
use crate::bundle::CertificateAuthorityId;
use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::{deserialize_platform, serialize_platform};
use dsh_api::platform::{DshPlatform, VhostZone};
use rcgen::{Certificate, KeyPair};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

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

impl ProxyCertificateBundle {
  /// Create proxy certificate bundle with self-signed certificates.
  ///
  /// # Parameters
  /// * `config` - Proxy certificate bundle configuration.
  pub(crate) fn create_self_signed(config: ProxyCertificateBundleConfig) -> DshCliResult<Self> {
    let ca_certificate = generate_ca_certificate(&config.ca_common_name)?;
    let client_certificate = generate_client_certificate(config.client_id(), config.acl_group_name.clone(), &ca_certificate)?;
    let server_certificate = generate_server_certificate(&config.common_name()?, config.dns_entries()?, &ca_certificate)?;
    Ok(ProxyCertificateBundle { config, ca_certificate, client_certificate, server_certificate })
  }

  /// Create proxy certificate bundle with self-signed certificates.
  ///
  /// # Parameters
  /// * `config` - Proxy certificate bundle configuration.
  /// * `certificate_authority_id` - Selects the certificate authority.
  pub(crate) fn create_signed(config: ProxyCertificateBundleConfig, certificate_authority_id: CertificateAuthorityId) -> DshCliResult<Self> {
    todo!()
  }
}

/// Contains configuration for certificate bundle.
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
/// * `certificate: rcgen::Certificate`
/// * `key_pair: rcgen::KeyPair`
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

impl ProxyCertificateBundleConfig {
  pub(crate) fn client_id(&self) -> String {
    self.tenant.clone()
  }

  // Make sure that the CN has the same value as the first SAN so that we don't 'waste' a DNS
  // record on a unique name.
  pub(crate) fn common_name(&self) -> DshCliResult<String> {
    Ok(self.platform.proxy_vhost_index(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), 0)?)
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
    for index in 0..self.effective_number_of_dns_entries() {
      dns_entries.push(self.platform.proxy_vhost_index(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), index)?);
    }
    if self.enable_schema_store {
      dns_entries.push(self.platform.proxy_schema_store_vhost(&self.tenant, &self.proxy_name, self.vhost_zone.clone())?);
    }
    Ok(dns_entries)
  }

  pub(crate) fn effective_number_of_dns_entries(&self) -> usize {
    if self.enable_schema_store {
      usize::min(self.number_of_dns_records, 9)
    } else {
      self.number_of_dns_records
    }
  }

  pub(crate) fn group_id(&self, index: usize) -> String {
    match &self.acl_group_name {
      Some(acl_group_name) => self.platform.proxy_consumer_group_acl(&self.tenant, &self.proxy_name, acl_group_name, index),
      None => self.platform.proxy_consumer_group(&self.tenant, &self.proxy_name, index),
    }
  }
}

impl Debug for ProxyCertificateBundleConfig {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let mut builder = f.debug_struct("ProxyCertificateBundleConfig");
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

// /// Generates self-signed certificate authority certificate.
// ///
// /// Certificate contents:
// /// * Subject alt names: CA common name from parameter `ca_common_name.
// /// * Distinguished name:
// ///   * `CN` - CA common name from parameter `ca_common_name
// ///   * `O` - `"Koninklijke KPN N.V."`
// ///   * `L` - `"Rotterdam"`
// ///   * `ST` - `"Zuid-Holland"`
// ///   * `C` - `"NL"`
// ///
// /// # Parameters
// /// * `ca_common_name` - Name to use for the self-signed certificate.
// fn generate_ca_certificate<T>(ca_common_name: T) -> DshCliResult<DshCertificate>
// where
//   T: Into<String> + Copy,
// {
//   let distinguished_name = kpn_distinguished_name(ca_common_name.into(), None::<T>);
//   let mut params: CertificateParams = CertificateParams::new(vec![ca_common_name.into()])?;
//   params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
//   params.distinguished_name = distinguished_name;
//   (params.not_before, params.not_after) = not_before_not_after(365);
//   params.use_authority_key_identifier_extension = true;
//   params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign, KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment];
//   params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth, ExtendedKeyUsagePurpose::ClientAuth];
//   let key_pair = generate_key_pair()?;
//   let certificate = params.self_signed(&key_pair)?;
//   Ok(DshCertificate { certificate, key_pair })
// }

// /// Generates client certificate.
// ///
// /// Signed by the provided `ca_certificate`.
// /// Contains optional `OU` field for ACL groups.
// ///
// /// Certificate contents:
// /// * Distinguished name:
// ///   * `CN` - Client id from proxy bundle configuration: `config.client_id`.
// ///   * `O` - `"Koninklijke KPN N.V."`
// ///   * `OU` - Optional, organizational unit name from proxy bundle configuration:
// ///     `config.acl_group_name`.
// ///   * `L` - `"Rotterdam"`
// ///   * `ST` - `"Zuid-Holland"`
// ///   * `C` - `"NL"`
// ///
// /// # Parameters
// /// * `client_common_name` - Client common name.
// /// * `organizational_unit_name` - Optional organizational unit name.
// /// * `ca_certificate` - Certificate authority certificate used to sign the generated certificate.
// fn generate_client_certificate<S, T>(client_common_name: S, organizational_unit_name: Option<T>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate>
// where
//   S: Into<DnValue>,
//   T: Into<DnValue>,
// {
//   let not_before_not_after = not_before_not_after(365);
//   let mut params: CertificateParams = CertificateParams::new(vec![])?;
//   params.distinguished_name = kpn_distinguished_name(client_common_name, organizational_unit_name);
//   (params.not_before, params.not_after) = not_before_not_after;
//   params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
//   params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
//   let key_pair = generate_key_pair()?;
//   let csr_pem = params.serialize_request(&key_pair)?.pem()?;
//   let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
//   (csr_params.params.not_before, csr_params.params.not_after) = not_before_not_after;
//   let certificate = csr_params.signed_by(&ca_certificate.certificate, &ca_certificate.key_pair)?;
//   Ok(DshCertificate { certificate, key_pair })
// }

// fn not_before_not_after(days: i64) -> (OffsetDateTime, OffsetDateTime) {
//   let now = time::OffsetDateTime::now_utc();
//   (now, now + time::Duration::days(days))
// }

// /// Generates server certificate.
// ///
// /// Signed by the provided `ca_certificate`.
// ///
// /// Certificate contents:
// /// * Subject alt names: List of dns entries from proxy bundle configuration (`config.dns_entries`).
// /// * Distinguished name:
// ///   * `CN` - Common name from proxy bundle configuration: `config.common_name`.
// ///   * `O` - `"Koninklijke KPN N.V."`
// ///   * `L` - `"Rotterdam"`
// ///   * `ST` - `"Zuid-Holland"`
// ///   * `C` - `"NL"`
// ///
// /// # Parameters
// /// * `server_common_name` - Proxy certificate bundle configuration:
// /// * `subject_alt_names` - Optional list of subject alt names.
// /// * `ca_certificate` - Certificate authority certificate used to sign the generated certificate.
// fn generate_server_certificate<T>(server_common_name: T, subject_alt_names: impl Into<Vec<String>>, ca_certificate: &DshCertificate) -> DshCliResult<DshCertificate>
// where
//   T: Into<DnValue>,
// {
//   let not_before_not_after = not_before_not_after(365);
//   let mut params: CertificateParams = CertificateParams::new(subject_alt_names)?;
//   params.distinguished_name = kpn_distinguished_name(server_common_name, None::<T>);
//   (params.not_before, params.not_after) = not_before_not_after;
//   params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
//   params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
//   let key_pair = generate_key_pair()?;
//   let csr_pem = params.serialize_request(&key_pair)?.pem()?;
//   let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
//   (csr_params.params.not_before, csr_params.params.not_after) = not_before_not_after;
//   let certificate = csr_params.signed_by(&ca_certificate.certificate, &ca_certificate.key_pair)?;
//   Ok(DshCertificate { certificate, key_pair })
// }

// const KPN_ORGANIZATION_NAME: &str = "Koninklijke KPN N.V.";
// const KPN_LOCALITY_NAME: &str = "Rotterdam";
// const KPN_STATE_OR_PROVINCE_NAME: &str = "Zuid-Holland";
// const KPN_COUNTRY_NAME: &str = "NL";
//
// /// Create distinguished name.
// ///
// /// * `CN` - Provided common name.
// /// * `O` - `"Koninklijke KPN N.V."`
// /// * `OU` - Optional, provided organizational unit name used for acl group capability.
// /// * `L` - `"Rotterdam"`
// /// * `ST` - `"Zuid-Holland"`
// /// * `C` - `"NL"`
// ///
// /// # Parameters
// /// * `common_name` - Common name.
// /// * `organizational_unit_name` - Optional organizational unit name.
// fn kpn_distinguished_name<S, T>(common_name: S, organizational_unit_name: Option<T>) -> DistinguishedName
// where
//   S: Into<DnValue>,
//   T: Into<DnValue>,
// {
//   let mut distinguished_name = DistinguishedName::new();
//   distinguished_name.push(DnType::CommonName, common_name.into());
//   distinguished_name.push(DnType::OrganizationName, KPN_ORGANIZATION_NAME);
//   if let Some(organizational_unit_name) = organizational_unit_name {
//     distinguished_name.push(DnType::OrganizationalUnitName, organizational_unit_name.into());
//   }
//   distinguished_name.push(DnType::LocalityName, KPN_LOCALITY_NAME);
//   distinguished_name.push(DnType::StateOrProvinceName, KPN_STATE_OR_PROVINCE_NAME);
//   distinguished_name.push(DnType::CountryName, KPN_COUNTRY_NAME);
//   distinguished_name
// }

// fn generate_key_pair() -> DshCliResult<KeyPair> {
//   KeyPair::generate_rsa_for(&rcgen::PKCS_RSA_SHA256, RsaKeySize::_4096).map_err(DshCliError::from)
// }
