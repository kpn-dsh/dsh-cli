//! Builder to generate certificate signing requests and key pairs.
//!
//! <span style="color:#a00000">This module is only available when the <code>rcgen</code>
//! feature is enabled.</span>
//!
//! A `RockApiClient` contains all configuration items necessary to access the _RoCK API_, and
//! many functions and methods to make use of the functions of the API.
//!
//!
//! ## Examples
//!
//!
//! This example will create a `CertificateSigningRequest` and a `KeyPair` for a server
//! certificate with default KPN settings and a time-to-live of 90 days, which will then be
//! signed via the KPN _RoCK API_. The key pair and the generated certificate will be written
//! to the files `my-key.key` and `my-server-cert.cert` respectively.
//!
//! ```rust,no_run
//! # use crate::rock_api::types::Connectors;
//! # use crate::rock_api::client::RockApiClient;
//! # use std::error::Error;
//! # use std::fs;
//! # use rock_api::client::PkiConnector;
//! # use rock_api::csr::CsrBuilder;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! let common_name = "my-subdomain.my-tenant.my-domain.kpn.org".to_string();
//! let builder = CsrBuilder::default_kpn(common_name, None, true, false)?;
//! let (csr, key_pair) = builder.build()?;
//!
//! let client = RockApiClient::header_based_from_auth_token_file()?;
//! let server_certificate = client.generate_signed_certificate(&csr, PkiConnector::Internal).await?;
//! fs::write("my-key.key", &key_pair.serialize_pem())?;
//! fs::write("my-server-cert.cert", server_certificate.cert.as_ref().unwrap())?;
//! #  Ok(())
//! # }
//! ```

use crate::{DshCliError, DshCliResult};
use itertools::Itertools;
use rcgen::{
  CertificateParams, CertificateSigningRequest, DistinguishedName, DnType, DnValue, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, RsaKeySize, SanType,
  SignatureAlgorithm,
};

/// Distinguished organization name (`O`): `"Koninklijke KPN N.V."`.
pub const KPN_DN_ORGANIZATION_NAME: &str = "Koninklijke KPN N.V.";

/// Distinguished locality name (`L`): `"Rotterdam"`.
pub const KPN_DN_LOCALITY_NAME: &str = "Rotterdam";

/// Distinguished state or province name (`ST`): `"Zuid-Holland"`.
pub const KPN_DN_STATE_OR_PROVINCE_NAME: &str = "Zuid-Holland";

/// Distinguished country name (`C`): `"NL"`.
pub const KPN_DN_COUNTRY_NAME: &str = "NL";

#[derive(Debug, Default)]
pub struct CsrBuilder {
  common_name: Option<DnValue>,
  country: Option<DnValue>,
  extended_key_usages: Vec<ExtendedKeyUsagePurpose>,
  key_usages: Vec<KeyUsagePurpose>,
  locality: Option<DnValue>,
  organization: Option<DnValue>,
  organizational_unit: Option<DnValue>,
  rsa_key_size: Option<RsaKeySize>,
  signature_algorithm: Option<&'static SignatureAlgorithm>,
  state: Option<DnValue>,
  subject_alt_names: Vec<SanType>,
}

impl CsrBuilder {
  /// Set extended key usage `ClientAuth`.
  ///
  /// Enables extended key usage `ClientAuth`, which indicates that the generated certificate
  /// can be used as a client (mTLS) certificate.
  pub fn _client_certificate(mut self) -> Self {
    self.extended_key_usages.push(ExtendedKeyUsagePurpose::ClientAuth);
    self
  }

  pub fn common_name<T>(mut self, common_name: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.common_name = Some(common_name.into());
    self
  }

  pub fn country<T>(mut self, country: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.country = Some(country.into());
    self
  }

  pub fn _extended_key_usages<T>(mut self, purposes: Vec<T>) -> Self
  where
    T: Into<ExtendedKeyUsagePurpose>,
  {
    self.extended_key_usages = purposes.into_iter().map(|purpose| purpose.into()).collect_vec();
    self
  }

  pub fn key_usages<T>(mut self, purposes: Vec<T>) -> Self
  where
    T: Into<KeyUsagePurpose>,
  {
    self.key_usages = purposes.into_iter().map(|purpose| purpose.into()).collect_vec();
    self
  }

  pub fn locality<T>(mut self, locality: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.locality = Some(locality.into());
    self
  }

  pub fn organization<T>(mut self, organization: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.organization = Some(organization.into());
    self
  }

  pub fn organizational_unit<T>(mut self, organizational_unit: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.organizational_unit = Some(organizational_unit.into());
    self
  }

  pub fn rsa_key_size<T>(mut self, rsa_key_size: T) -> Self
  where
    T: Into<RsaKeySize>,
  {
    self.rsa_key_size = Some(rsa_key_size.into());
    self
  }

  /// Set extended key usage `ServerAuth`.
  ///
  /// Enables extended key usage `ServerAuth`, which indicates that the generated certificate
  /// can be used as a server certificate.
  pub fn server_certificate(mut self) -> Self {
    self.extended_key_usages.push(ExtendedKeyUsagePurpose::ServerAuth);
    self
  }

  pub fn signature_algorithm(mut self, algorithm: &'static SignatureAlgorithm) -> Self {
    self.signature_algorithm = Some(algorithm);
    self
  }

  pub fn state<T>(mut self, state: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.state = Some(state.into());
    self
  }

  pub fn subject_alt_names<T>(mut self, sans: Vec<T>) -> Self
  where
    T: Into<SanType>,
  {
    self.subject_alt_names = sans.into_iter().map(|san| san.into()).collect_vec();
    self
  }

  pub fn _add_subject_alt_name<T>(mut self, san: T) -> Self
  where
    T: Into<SanType>,
  {
    self.subject_alt_names.push(san.into());
    self
  }

  /// Build a csr from the current configuration.
  ///
  /// <span style="color:#a00000">This method is only available when the <code>rcgen</code>
  /// feature is enabled.</span>
  ///
  /// # Returns
  /// * `Ok()` - Tuple containing
  ///   * `CertificateSigningRequest` - The generated certificate signing request.
  ///   * `KeyPair` - The private/public key pair.
  pub fn build(&self) -> DshCliResult<(CertificateSigningRequest, KeyPair)> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::NoCa;
    params.subject_alt_names = self.subject_alt_names.clone();
    params.distinguished_name = self.distinguished_name()?;
    params.key_usages = self.key_usages.clone();
    params.extended_key_usages = self.extended_key_usages.clone();
    let key_pair = self.generate_key_pair()?;
    let csr = params.serialize_request(&key_pair)?;
    Ok((csr, key_pair))
  }

  /// Create distinguished name for KPN certificates.
  ///
  /// <span style="color:#a00000">This method is only available when the <code>rcgen</code>
  /// feature is enabled.</span>
  ///
  /// Creates the distinguished name. The following parameters are mandatory and must be set before
  /// calling this method: `common_name`, `country`, `locality`, `organization` and `state`. The
  /// `organizational_unit_name` is optional.
  ///
  /// # Returns
  /// * `Ok(DistinguishedName)` - Successfully created.
  /// * `Err(_)` - A mandatory parameter is missing.
  fn distinguished_name(&self) -> DshCliResult<DistinguishedName> {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(
      DnType::CommonName,
      self.common_name.clone().ok_or_else(|| DshCliError::from("common name not set"))?,
    );
    distinguished_name.push(DnType::CountryName, self.country.clone().ok_or_else(|| DshCliError::from("country not set"))?);
    distinguished_name.push(DnType::LocalityName, self.locality.clone().ok_or_else(|| DshCliError::from("locality not set"))?);
    distinguished_name.push(
      DnType::OrganizationName,
      self.organization.clone().ok_or_else(|| DshCliError::from("organization not set"))?,
    );
    if let Some(organizational_unit_name) = &self.organizational_unit {
      distinguished_name.push(DnType::OrganizationalUnitName, organizational_unit_name.clone());
    }
    distinguished_name.push(DnType::StateOrProvinceName, self.state.clone().ok_or_else(|| DshCliError::from("state not set"))?);
    Ok(distinguished_name)
  }

  /// Generate public/private key pair.
  ///
  /// <span style="color:#a00000">This method is only available when the <code>rcgen</code>
  /// feature is enabled.</span>
  ///
  /// Creates a public/private key pair, based on the mandatory parameters `signature_algorithm`
  /// and `rsa_key_size`.
  ///
  /// # Returns
  /// * `Ok(KeyPair)` - Successfully created.
  /// * `Err(_)` - A mandatory parameter is missing.
  fn generate_key_pair(&self) -> DshCliResult<KeyPair> {
    KeyPair::generate_rsa_for(
      self.signature_algorithm.ok_or_else(|| DshCliError::from("signature algorithm not set"))?,
      self.rsa_key_size.ok_or_else(|| DshCliError::from("rsa key size not set"))?,
    )
    .map_err(DshCliError::from)
  }
}
