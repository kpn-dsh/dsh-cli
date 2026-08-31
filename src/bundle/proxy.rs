use crate::bundle::ca_signed::_generate_signed_certificate_bundle;
use crate::bundle::self_signed::generate_self_signed_certificate_bundle;
use crate::bundle::{create_certificate_authority, CertificateAuthorityId};
use crate::context::Context;
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
    generate_self_signed_certificate_bundle(config)
  }

  /// Create proxy certificate bundle with ca-signed certificates.
  ///
  /// Creates a proxy certificate bundle with certificates signed by the designated certificate
  /// authority.
  ///
  /// # Parameters
  /// * `config` - Proxy certificate bundle configuration.
  /// * `certificate_authority_id` - Selects the certificate authority.
  /// * `context` - Optional pair of context reference and expiration days value). If non-empty,
  ///   the generated certificates will be printed via the `UnitFormatter` mechanism.
  pub(crate) async fn _create_ca_signed(
    config: ProxyCertificateBundleConfig,
    certificate_authority_id: CertificateAuthorityId,
    context: Option<(&Context, u64)>,
  ) -> DshCliResult<Self> {
    let certificate_authority = create_certificate_authority(certificate_authority_id).await?;
    let bundle = _generate_signed_certificate_bundle(config, certificate_authority.as_ref(), context).await?;
    Ok(bundle)
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
