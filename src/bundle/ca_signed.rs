use crate::bundle::proxy::{ProxyCertificateBundle, ProxyCertificateBundleConfig};
use crate::bundle::CertificateAuthority;
use crate::context::Context;
use crate::DshCliResult;
use log::debug;
use rcgen::{Ia5String, SanType};
use std::str::FromStr;

/// Create proxy certificate bundle with ca-signed certificates.
///
/// Creates a proxy certificate bundle with certificates signed by the provided certificate
/// authority.
///
/// # Parameters
/// * `config` - Proxy certificate bundle configuration.
/// * `certificate_authority` - Certificate authority.
/// * `context` - Optional pair of context reference and expiration days value). If non-empty,
///   the generated certificates will be printed via the `UnitFormatter` mechanism.
pub(crate) async fn _generate_signed_certificate_bundle(
  config: ProxyCertificateBundleConfig,
  certificate_authority: &(dyn CertificateAuthority + Send + Sync),
  context: Option<(&Context, u64)>,
) -> DshCliResult<ProxyCertificateBundle> {
  let _ca_chain = certificate_authority.ca_chain().await?;

  let _server = generate_server_certificate(&config, certificate_authority, context).await?;

  let _client = generate_client_certificate(&config, certificate_authority, context).await?;

  // TODO Attach ca chain
  todo!()
}

/// Create signed server certificate.
///
/// Creates a server certificate signed by the provided certificate authority.
///
/// # Parameters
/// * `config` - Proxy certificate bundle configuration, provides:
///   * `common_name` - Server common name.
///   * `dns_entries` - Subject alternative names.
/// * `certificate_authority` - Certificate authority.
/// * `context` - Optional pair of context reference and expiration days value. If non-empty,
///   the generated certificate will be printed via the `UnitFormatter` mechanism.
///
/// # Returns
/// Tuple containing
/// * `String` - Certificate id.
/// * `String` - Certificate in pem format.
async fn generate_server_certificate(
  config: &ProxyCertificateBundleConfig,
  certificate_authority: &(dyn CertificateAuthority + Send + Sync),
  context: Option<(&Context, u64)>,
) -> DshCliResult<(String, String)> {
  let csr_builder = certificate_authority
    .default_csr_builder()?
    .common_name(config.common_name()?)
    .server_certificate()
    .subject_alt_names(
      config
        .dns_entries()?
        .iter()
        .map(|dns_entry| Ia5String::from_str(dns_entry).map(SanType::DnsName))
        .collect::<Result<Vec<_>, _>>()?,
    );
  let (csr, _key_pair) = &csr_builder.build()?;
  debug!("generate signed server certificate");
  certificate_authority.sign_certificate(csr, context).await
}

/// Create signed client certificate.
///
/// Creates a client certificate signed by the provided certificate authority.
///
/// # Parameters
/// * `config` - Proxy certificate bundle configuration, provides:
///   * `client_id` - Client common name.
///   * `acl_group_name` - Optional value used for organization unit name (required when acl
///     groups are needed).
/// * `certificate_authority` - Certificate authority.
/// * `context` - Optional pair of context reference and expiration days value. If non-empty,
///   the generated certificate will be printed via the `UnitFormatter` mechanism.
///
/// # Returns
/// Tuple containing
/// * `String` - Certificate id.
/// * `String` - Certificate in pem format.
async fn generate_client_certificate(
  config: &ProxyCertificateBundleConfig,
  certificate_authority: &(dyn CertificateAuthority + Send + Sync),
  context: Option<(&Context, u64)>,
) -> DshCliResult<(String, String)> {
  let mut csr_builder = certificate_authority.default_csr_builder()?.common_name(config.client_id())._client_certificate();
  if let Some(acl_group_name) = &config.acl_group_name {
    csr_builder = csr_builder.organizational_unit(acl_group_name);
  }
  let (csr, _client_key_pair) = &csr_builder.build()?;
  debug!("generate signed client certificate");
  certificate_authority.sign_certificate(csr, context).await
}
