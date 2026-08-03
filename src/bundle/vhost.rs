// use crate::error::DshCliError;
// use crate::DshCliResult;
// use dsh_api::platform::{deserialize_platform, serialize_platform};
// use dsh_api::platform::{DshPlatform, VhostZone};
// use serde::{Deserialize, Serialize};
// use std::fmt::{Debug, Formatter};

// /// Contains configuration for certificate bundle.
// ///
// /// * `ca_common_name` - Certificate authority common name.
// /// * `platform` - Platform.
// /// * `tenant: String` - Tenant.
// /// * `vhost_zone` - Public or private.
// #[derive(Clone, Deserialize, Serialize)]
// pub(crate) struct VhostCertificateBundleConfig {
//   #[serde(rename = "ca-common-name")]
//   pub(crate) ca_common_name: String,
//   #[serde(deserialize_with = "deserialize_platform", serialize_with = "serialize_platform")]
//   pub(crate) platform: DshPlatform,
//   pub(crate) tenant: String,
//   #[serde(rename = "vhost")]
//   pub(crate) vhost: String,
//   #[serde(rename = "vhost-zone")]
//   pub(crate) vhost_zone: VhostZone,
// }

// /// Contains vhost certificate bundle and configuration.
// ///
// /// * `config: VhostCertificateBundleConfig`
// /// * `csr: DshCertificate`
// #[derive(Debug)]
// pub(crate) struct VhostCertificateBundle {
//   pub config: VhostCertificateBundleConfig,
//   pub csr: DshCsr,
// }

// pub(crate) struct LocalVhostCertificateBundle {
//   pub(crate) configuration: (VhostCertificateBundleConfig, String),
//   pub(crate) csr_key: LocalVhostCertificate,
//   pub(crate) csr_pem: LocalVhostCertificate,
// }

// pub(crate) struct LocalVhostCertificate {
//   pub(crate) value: String,
//   pub(crate) filename: String,
// }

// impl VhostCertificateBundleConfig {
//   pub(crate) fn domain_from_platform(&self) -> DshCliResult<String> {
//     match self.vhost_zone {
//       VhostZone::Private => match &self.platform.private_domain() {
//         Some(private_domain) => Ok(private_domain.to_string()),
//         None => Err(DshCliError::Configuration(format!("platform '{}' does not support private vhosts", &self.platform))),
//       },
//       VhostZone::Public => Ok(self.platform.public_domain().to_string()),
//     }
//   }
// }

// impl Debug for VhostCertificateBundleConfig {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     let mut builder = f.debug_struct("VhostCertificateBundleConfig");
//     builder.field("ca_common_name", &self.ca_common_name);
//     builder.field("platform", &self.platform.name());
//     builder.field("tenant", &self.tenant);
//     builder.field("vhost", &self.vhost);
//     builder.field("vhost_zone", &self.vhost_zone);
//     builder.finish()
//   }
// }

// impl TryFrom<VhostCertificateBundleConfig> for VhostCertificateBundle {
//   type Error = DshCliError;
//
//   fn try_from(config: VhostCertificateBundleConfig) -> DshCliResult<Self> {
//     let csr = generate_csr(&config.ca_common_name, vec![config.vhost.clone()])?;
//     // let ca_certificate = generate_ca_certificate(&config.ca_common_name)?;
//     // let client_certificate = generate_client_certificate(config.client_id(), config.acl_group_name.clone(), &ca_certificate)?;
//     // let server_certificate = generate_server_certificate(&config.common_name()?, config.dns_entries()?, &ca_certificate)?;
//     Ok(VhostCertificateBundle { config, csr })
//   }
// }
