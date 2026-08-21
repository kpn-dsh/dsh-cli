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
use crate::{err, DshCliError, DshCliResult};
use rcgen::{
  CertificateParams, CertificateSigningRequest, DistinguishedName, DnType, DnValue, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose, RsaKeySize, SanType,
  SignatureAlgorithm, PKCS_RSA_SHA384,
};

/// Distinguished organization name (`O`): `"Koninklijke KPN N.V."`.
pub const KPN_DN_ORGANIZATION_NAME: &str = "Koninklijke KPN N.V.";

/// Distinguished locality name (`L`): `"Rotterdam"`.
pub const KPN_DN_LOCALITY_NAME: &str = "Rotterdam";

/// Distinguished state or province name (`ST`): `"Zuid-Holland"`.
pub const KPN_DN_STATE_OR_PROVINCE_NAME: &str = "Zuid-Holland";

/// Distinguished country name (`C`): `"NL"`.
pub const KPN_DN_COUNTRY_NAME: &str = "NL";

#[derive(Debug)]
pub struct CsrBuilder {
  common_name: DnValue,
  country: DnValue,
  extended_key_usages: Vec<ExtendedKeyUsagePurpose>,
  key_usages: Vec<KeyUsagePurpose>,
  locality: DnValue,
  organization: DnValue,
  organizational_unit: Option<DnValue>,
  rsa_key_size: RsaKeySize,
  signature_algorithm: &'static SignatureAlgorithm,
  state: DnValue,
  subject_alt_names: Vec<SanType>,
}

impl CsrBuilder {
  const IS_CA: IsCa = IsCa::NoCa;

  /// Create `CsrBuilder` with default parameters for KPN certificate.
  ///
  /// <span style="color:#a00000">This method is only available when the <code>rcgen</code>
  /// feature is enabled.</span>
  ///
  /// Creates a `CsrBuilder` with the proper default settings for a signing request via the KPN
  /// _RoCK API_.
  ///
  /// Note that at least one of `server_certificate` or `client_certificate` must be `true`.
  ///
  /// # Parameters
  ///
  /// * `common_name` - Common name for the certificate.
  /// * `organizational_unit` - Optional organizational unit name for the certificate.
  /// * `server_certificate` - If `true` server side authentication will be supported.
  /// * `client_certificate` - If `true` client side authentication (`mTLS`) will be supported.
  ///
  /// # Default values
  ///
  /// * `country` - `"NL"`
  /// * `extended_key_usages` - Set via the parameters `server_certificate` and `client_certificate`.
  /// * `key_usages` - `DigitalSignature`, `KeyEncipherment` and `ContentCommitment (NonRepudiation)`.
  /// * `locality` - `"Rotterdam"`
  /// * `organization` - `"Koninklijke KPN N.V."`
  /// * `rsa_key_size` - `4096` bits
  /// * `signature_algorithm` - `RSA SHA-384`
  /// * `state` - `"Zuid-Holland"`
  /// * `subject_alt_names` - Empty.
  pub fn default_kpn<T>(common_name: T, organizational_unit: Option<DnValue>, server_certificate: bool, client_certificate: bool) -> DshCliResult<Self>
  where
    T: Into<DnValue>,
  {
    let extended_key_usages = match (server_certificate, client_certificate) {
      (false, false) => return err!("at least one of server_certificate or client_certificate must be true"),
      (false, true) => vec![ExtendedKeyUsagePurpose::ClientAuth],
      (true, false) => vec![ExtendedKeyUsagePurpose::ServerAuth],
      (true, true) => vec![ExtendedKeyUsagePurpose::ClientAuth, ExtendedKeyUsagePurpose::ServerAuth],
    };
    Ok(Self {
      common_name: common_name.into(),
      country: DnValue::from(KPN_DN_COUNTRY_NAME),
      extended_key_usages,
      key_usages: vec![KeyUsagePurpose::DigitalSignature, KeyUsagePurpose::KeyEncipherment, KeyUsagePurpose::ContentCommitment],
      locality: DnValue::from(KPN_DN_LOCALITY_NAME),
      organization: DnValue::from(KPN_DN_ORGANIZATION_NAME),
      organizational_unit,
      rsa_key_size: RsaKeySize::_4096,
      signature_algorithm: &PKCS_RSA_SHA384,
      state: DnValue::from(KPN_DN_STATE_OR_PROVINCE_NAME),
      subject_alt_names: vec![],
    })
  }

  pub fn _add_subject_alt_name<T>(mut self, san: T) -> Self
  where
    T: Into<SanType>,
  {
    self.subject_alt_names.push(san.into());
    self
  }

  pub fn _common_name<T>(mut self, common_name: T) -> Self
  where
    T: Into<DnValue>,
  {
    self.common_name = common_name.into();
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
    params.is_ca = Self::IS_CA;
    params.subject_alt_names = self.subject_alt_names.clone();
    params.distinguished_name = self.distinguished_name();
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
  /// Creates the distinguished name.
  pub fn distinguished_name(&self) -> DistinguishedName {
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, self.common_name.clone());
    distinguished_name.push(DnType::CountryName, self.country.clone());
    distinguished_name.push(DnType::LocalityName, self.locality.clone());
    distinguished_name.push(DnType::OrganizationName, self.organization.clone());
    if let Some(organizational_unit_name) = &self.organizational_unit {
      distinguished_name.push(DnType::OrganizationalUnitName, organizational_unit_name.clone());
    }
    distinguished_name.push(DnType::StateOrProvinceName, self.state.clone());
    distinguished_name
  }

  /// Generate public/private key pair.
  ///
  /// <span style="color:#a00000">This method is only available when the <code>rcgen</code>
  /// feature is enabled.</span>
  ///
  /// Creates a public/private key pair as required by the _RoCK API_, based on the configured
  /// signature algorithm and rsa key size.
  ///
  /// * `PkiConnector::Internal` - `PKCS_RSA_SHA384`, `4096` bits,
  /// * `PkiConnector::ExternalRsdv` - `PKCS_RSA_SHA384`, `3072` bits.
  pub fn generate_key_pair(&self) -> DshCliResult<KeyPair> {
    KeyPair::generate_rsa_for(self.signature_algorithm, self.rsa_key_size).map_err(DshCliError::from)
  }
}
