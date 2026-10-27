use crate::bundle::CertificateAuthorityId;
use crate::error::DshCliError;
use crate::DshCliResult;
use dsh_api::platform::{deserialize_platform, serialize_platform};
use dsh_api::platform::{DshPlatform, VhostZone};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProxyCertificateBundleConfig {
  #[serde(rename = "acl-group-name", skip_serializing_if = "Option::is_none")]
  pub(crate) acl_group_name: Option<String>,
  #[serde(default, rename = "attach-ca-chain")]
  pub(crate) attach_ca_chain: bool,
  #[serde(rename = "bootstrap-servers", skip_serializing_if = "Option::is_none")]
  pub(crate) bootstrap_servers: Option<String>,
  #[serde(rename = "ca-common-name", skip_serializing_if = "Option::is_none")]
  pub(crate) ca_common_name: Option<String>,
  #[serde(rename = "ca-id", skip_serializing_if = "Option::is_none")]
  pub(crate) certificate_authority_id: Option<CertificateAuthorityId>,
  #[serde(rename = "enable-schema-store")]
  pub(crate) enable_schema_store: bool,
  #[serde(rename = "group-id", skip_serializing_if = "Option::is_none")]
  pub(crate) group_id: Option<String>,
  #[serde(rename = "number_of_dns_records")]
  pub(crate) number_of_dns_records: usize,
  #[serde(deserialize_with = "deserialize_platform", serialize_with = "serialize_platform")]
  pub(crate) platform: DshPlatform,
  #[serde(rename = "proxy-name")]
  pub(crate) proxy_name: String,
  #[serde(rename = "schema-store-endpoint", skip_serializing_if = "Option::is_none")]
  pub(crate) schema_store_endpoint: Option<String>,
  pub(crate) tenant: String,
  #[serde(rename = "vhost-zone")]
  pub(crate) vhost_zone: VhostZone,
}

impl ProxyCertificateBundleConfig {
  pub(crate) fn client_id(&self) -> String {
    self.tenant.clone()
  }

  // Make sure that the CN has the same value as the first SAN so that we don't 'waste' a DNS
  // record on a unique name.
  pub(crate) fn server_common_name(&self) -> DshCliResult<String> {
    Ok(self.platform.proxy_vhost_index(&self.tenant, &self.proxy_name, self.vhost_zone.clone(), 0)?)
  }

  pub(crate) fn client_common_name(&self) -> DshCliResult<String> {
    Ok(self.platform.tenant_domain(&self.tenant, self.vhost_zone.clone())?)
  }

  pub(crate) fn domain_from_platform(&self) -> DshCliResult<String> {
    match self.vhost_zone {
      VhostZone::Private => match &self.platform.private_domain() {
        Some(private_domain) => Ok(private_domain.to_string()),
        None => Err(DshCliError::Configuration(format!("platform '{}' does not support private vhosts", self.platform))),
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
}
