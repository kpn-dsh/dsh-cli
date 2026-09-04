use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::bundle::CertificateAuthority;
use crate::context::Context;
use crate::error::DshCliError;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::secret_metadata::find_certificates;
use crate::subjects::secret::capabilities::CERTIFICATE_LABELS_SHOW;
use crate::{err, DshCliResult};
use dsh_api::platform::VhostZone;
use log::{debug, trace};
use rcgen::{CertificateSigningRequest, Ia5String, KeyPair, SanType};
use std::str::FromStr;

/// Contains proxy certificate bundle and configuration.
///
/// * `config: ProxyCertificateBundleConfig`
/// * `client_csr` - Client certificate signing request.
/// * `client_key` - Client private/public key pair.
/// * `client_pem` - Signed client certificate.
/// * `server_csr` - Client certificate signing request.
/// * `server_key` - Server private/public key pair.
/// * `server_pem` - Signed server certificate.
#[derive(Debug)]
pub(crate) struct ProxyCaCertificateBundle {
  pub config: ProxyCertificateBundleConfig,
  pub ca_chain: String,
  pub client_csr: String,
  pub client_key: String,
  pub client_pem: String,
  pub server_csr: String,
  pub server_key: String,
  pub server_pem: String,
}

impl ProxyCaCertificateBundle {
  /// Create proxy certificate bundle with ca-signed certificates.
  ///
  /// Creates a proxy certificate bundle with certificates signed by the designated certificate
  /// authority.
  ///
  /// # Parameters
  /// * `config` - Proxy certificate bundle configuration.
  /// * `certificate_authority` - The certificate authority.
  /// * `context` - Optional pair of context reference and expiration days value). If non-empty,
  ///   the generated certificates will be printed via the `UnitFormatter` mechanism.
  pub(crate) async fn create_ca_signed(
    config: ProxyCertificateBundleConfig,
    certificate_authority: &(dyn CertificateAuthority + Send + Sync),
    context: Option<(&Context, u64)>,
  ) -> DshCliResult<Self> {
    generate_signed_certificate_bundle(config, certificate_authority, context).await
  }
}

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
pub(crate) async fn generate_signed_certificate_bundle(
  config: ProxyCertificateBundleConfig,
  certificate_authority: &(dyn CertificateAuthority + Send + Sync),
  context: Option<(&Context, u64)>,
) -> DshCliResult<ProxyCaCertificateBundle> {
  // Check if RoCK supports this platform and tenant
  let tenant_domain = &config.platform.tenant_domain(&config.tenant, config.vhost_zone.clone())?;

  debug!("tenant_domain: {}", tenant_domain);
  if !certificate_authority.authorization_check(tenant_domain).await? {
    return err!(
      "authenticated user has no authorization for tenant domain '{}' at certificate authority",
      tenant_domain
    );
  }
  debug!("authenticated user is authorizated for tenant domain '{}' at certificate authority", tenant_domain);

  let vhost_domain = match config.vhost_zone {
    VhostZone::Private => config.platform.tenant_private_vhost_domain(&config.tenant, &config.proxy_name)?,
    VhostZone::Public => config.platform.public_vhost_domain(&config.proxy_name),
  };
  debug!("vhost_domain: {}", vhost_domain);

  let (server_csr, server_cert_id, server_pem, server_key) = generate_server_certificate(&config, certificate_authority, context).await?;
  debug!("server certificate '{}' signed by rock", server_cert_id);
  trace!("server certificate\n{}", server_pem);

  if let Some(cs) = find_certificates(&server_pem) {
    if let Some((context, expiration_days)) = context {
      context.print_explanation("generated server certificate".to_string());
      for metadata in cs {
        UnitFormatter::new(&metadata.subject_common_name, &CERTIFICATE_LABELS_SHOW, context).print(&(metadata, Some(expiration_days)), None)?;
      }
    }
  } else {
    return err!("received invalid server certificate from rock api");
  }

  let (client_csr, client_cert_id, client_pem, client_key) = generate_client_certificate(&config, certificate_authority, context).await?;

  debug!("client certificate '{}' signed by rock", client_cert_id);
  trace!("client certificate\n{}", client_pem);

  if let Some(cs) = find_certificates(&client_pem) {
    if let Some((context, expiration_days)) = context {
      context.print_explanation("generated client certificate".to_string());
      for metadata in cs {
        UnitFormatter::new(&metadata.subject_common_name, &CERTIFICATE_LABELS_SHOW, context).print(&(metadata, Some(expiration_days)), None)?;
      }
    }
  } else {
    return err!("received invalid client certificate from rock api");
  }

  let ca_chain = certificate_authority
    .ca_chain()
    .await?
    .ok_or_else(|| DshCliError::RockApi("certificate authority could not provide ca chain".to_string()))?;

  Ok(ProxyCaCertificateBundle {
    config,
    ca_chain,
    client_csr: client_csr.pem()?,
    client_key: client_key.serialize_pem(),
    client_pem,
    server_csr: server_csr.pem()?,
    server_key: server_key.serialize_pem(),
    server_pem,
  })
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
/// * `KeyPair` - rivate/public key pair.
async fn generate_server_certificate(
  config: &ProxyCertificateBundleConfig,
  certificate_authority: &(dyn CertificateAuthority + Send + Sync),
  context: Option<(&Context, u64)>,
) -> DshCliResult<(CertificateSigningRequest, String, String, KeyPair)> {
  let csr_builder = certificate_authority
    .default_csr_builder()?
    .common_name(config.server_common_name()?)
    .server_certificate()
    .subject_alt_names(
      config
        .dns_entries()?
        .iter()
        .map(|dns_entry| Ia5String::from_str(dns_entry).map(SanType::DnsName))
        .collect::<Result<Vec<_>, _>>()?,
    );
  debug!("server certificate signing request builder {:#?}", csr_builder);
  let (csr, key_pair) = csr_builder.build()?;
  debug!("generate signed server certificate");
  let (certificate_id, mut pem) = certificate_authority.sign_certificate(&csr, context).await?;
  if config.attach_ca_chain {
    pem = certificate_authority.attach_ca_chain(&pem).await?;
  }
  Ok((csr, certificate_id, pem, key_pair))
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
/// * `KeyPair` - rivate/public key pair.
async fn generate_client_certificate(
  config: &ProxyCertificateBundleConfig,
  certificate_authority: &(dyn CertificateAuthority + Send + Sync),
  context: Option<(&Context, u64)>,
) -> DshCliResult<(CertificateSigningRequest, String, String, KeyPair)> {
  let mut csr_builder = certificate_authority
    .default_csr_builder()?
    .common_name(config.client_common_name()?)
    .client_certificate();
  if let Some(acl_group_name) = &config.acl_group_name {
    csr_builder = csr_builder.organizational_unit(acl_group_name);
  }
  debug!("client certificate signing request builder {:#?}", csr_builder);
  let (csr, key_pair) = csr_builder.build()?;
  debug!("generate signed client certificate");
  let (certificate_id, mut pem) = certificate_authority.sign_certificate(&csr, context).await?;
  if config.attach_ca_chain {
    pem = certificate_authority.attach_ca_chain(&pem).await?;
  }
  Ok((csr, certificate_id, pem, key_pair))
}
