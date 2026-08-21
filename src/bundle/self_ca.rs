use crate::bundle::{generate_key_pair, kpn_distinguished_name, not_before_not_after, CertificateAuthority};
use crate::context::Context;
use crate::{err, DshCliResult};
use async_trait::async_trait;
use rcgen::{Certificate, CertificateParams, CertificateSigningRequest, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose};

pub(crate) struct _SelfSigningCertificateAuthority {
  _ca_certificate: Certificate,
  _key_pair: KeyPair,
}

impl _SelfSigningCertificateAuthority {
  pub(crate) fn _create(ca_common_name: &str) -> DshCliResult<Box<dyn CertificateAuthority + Send + Sync>> {
    let distinguished_name = kpn_distinguished_name(ca_common_name, None::<&str>);
    let mut params: CertificateParams = CertificateParams::new(vec![ca_common_name.into()])?;
    params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    params.distinguished_name = distinguished_name;
    (params.not_before, params.not_after) = not_before_not_after(365);
    params.use_authority_key_identifier_extension = true;
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign, KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth, ExtendedKeyUsagePurpose::ClientAuth];
    let key_pair = generate_key_pair()?;
    let ca_certificate = params.self_signed(&key_pair)?;
    Ok(Box::new(Self { _ca_certificate: ca_certificate, _key_pair: key_pair }))
  }
}

#[async_trait]
impl CertificateAuthority for _SelfSigningCertificateAuthority {
  async fn attach_ca_chain(&self, certificate_pem: &str) -> DshCliResult<String> {
    Ok(certificate_pem.to_string())
  }

  async fn authorization_check(&self, _tenant_domain: &str) -> DshCliResult<bool> {
    Ok(true)
  }

  async fn ca_chain(&self) -> DshCliResult<Option<String>> {
    Ok(None)
  }

  async fn check_connection(&self) -> DshCliResult<()> {
    Ok(())
  }

  async fn existing_certificate(&self, _vhost_domain: &str, _context: Option<(&Context, u64)>) -> DshCliResult<Option<String>> {
    Ok(None)
  }

  async fn list(&self, _domain: &str, _context: &Context, _expiration_days: u64) -> DshCliResult<()> {
    Ok(())
  }

  async fn signed_certificate(&self, _csr: &CertificateSigningRequest, _context: Option<(&Context, u64)>) -> DshCliResult<(String, String)> {
    // let not_before_not_after = not_before_not_after(365);
    // let mut params: CertificateParams = CertificateParams::new(subject_alt_names)?;
    // params.distinguished_name = kpn_distinguished_name(server_common_name, None::<T>);
    // (params.not_before, params.not_after) = not_before_not_after;
    // params.key_usages = vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::KeyAgreement];
    // params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    // let key_pair = generate_key_pair()?;
    // let csr_pem = params.serialize_request(&key_pair)?.pem()?;
    // let mut csr_params = CertificateSigningRequestParams::from_pem(&csr_pem)?;
    // (csr_params.params.not_before, csr_params.params.not_after) = not_before_not_after;
    // let certificate = csr_params.signed_by(&self.ca_certificate, &self.key_pair)?;
    // Ok(DshCertificate { certificate, key_pair })
    err!("not yet implemented")
  }
}
