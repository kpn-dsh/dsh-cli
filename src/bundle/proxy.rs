use crate::bundle::{generate_ca_certificate, generate_client_certificate, generate_server_certificate, DshCertificate};
use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::{deserialize_platform, serialize_platform};
use dsh_api::platform::{DshPlatform, VhostZone};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

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

pub(crate) struct LocalProxyCertificateBundle {
  pub(crate) configuration: (ProxyCertificateBundleConfig, String),
  pub(crate) ca_key: LocalProxyCertificate,
  pub(crate) ca_pem: LocalProxyCertificate,
  pub(crate) client_key: LocalProxyCertificate,
  pub(crate) client_pem: LocalProxyCertificate,
  pub(crate) server_key: LocalProxyCertificate,
  pub(crate) server_pem: LocalProxyCertificate,
}

pub(crate) struct LocalProxyCertificate {
  pub(crate) value: String,
  pub(crate) filename: String,
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

impl TryFrom<ProxyCertificateBundleConfig> for ProxyCertificateBundle {
  type Error = DshCliError;

  fn try_from(config: ProxyCertificateBundleConfig) -> DshCliResult<Self> {
    let ca_certificate = generate_ca_certificate(&config.ca_common_name)?;
    let client_certificate = generate_client_certificate(config.client_id(), config.acl_group_name.clone(), &ca_certificate)?;
    let server_certificate = generate_server_certificate(&config.common_name()?, config.dns_entries()?, &ca_certificate)?;
    Ok(ProxyCertificateBundle { config, ca_certificate, client_certificate, server_certificate })
  }
}
