use crate::bundle::csr::{CsrBuilder, KPN_DN_COUNTRY_NAME, KPN_DN_LOCALITY_NAME, KPN_DN_ORGANIZATION_NAME, KPN_DN_STATE_OR_PROVINCE_NAME};
use crate::bundle::CertificateAuthority;
use crate::context::Context;
use crate::error::DshCliError;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, SubjectFormatter, Value};
use crate::{err, DshCliResult};
use async_trait::async_trait;
use itertools::Itertools;
use log::debug;
use rcgen::{CertificateSigningRequest, KeyUsagePurpose};
use rock_api::client::{CertsParameter, PkiConnector, RockApiClient};
use rock_api::error::RockApiError;
use rock_api::types::Certificate;
use serde::Serialize;
use std::collections::HashMap;

#[cfg(feature = "rock")]
pub(crate) struct RockCertificateAuthority {
  client: RockApiClient,
  pki_connector: PkiConnector,
}

impl RockCertificateAuthority {
  pub(crate) async fn create(client: RockApiClient, pki_connector: PkiConnector) -> DshCliResult<Box<dyn CertificateAuthority + Send + Sync>> {
    debug!("create rock certificate authority {}", pki_connector);
    let ca = Box::new(Self { client, pki_connector });
    ca.check_connection().await?;
    Ok(ca)
  }
}

#[cfg(feature = "rock")]
#[async_trait]
impl CertificateAuthority for RockCertificateAuthority {
  async fn attach_ca_chain(&self, certificate_pem: &str) -> DshCliResult<String> {
    Ok(self.client.attach_ca_chain(certificate_pem, self.pki_connector.clone()).await?)
  }

  async fn authorization_check(&self, tenant_domain: &str) -> DshCliResult<bool> {
    Ok(self.client.is_authorized(&format!("$.{}", tenant_domain)).await?)
  }

  async fn ca_chain(&self) -> DshCliResult<Option<String>> {
    Ok(Some(self.client.ca_chain(self.pki_connector.clone()).await?.chain))
  }

  async fn check_connection(&self) -> DshCliResult<()> {
    match self.client.health().await {
      Ok(health) => {
        if health.status == "OK" {
          debug!("connected to rock api and health status ok");
          match self.client.cert("00000000-0000-0000-0000-000000000000").await {
            Ok(_) | Err(RockApiError::BadRequest { .. }) | Err(RockApiError::NotFound { .. }) => {
              debug!("authorized for rock api");
              Ok(())
            }
            Err(RockApiError::NotAuthorized { message }) => {
              debug!("not authorized for rock api ({})", message);
              err!("not authorized for rock api, log in using \"rock-client get_auth_token\" command")
            }
            Err(rock_api_error) => Err(DshCliError::from(rock_api_error)),
          }
        } else {
          debug!("connected to rock api but health status not ok");
          Err(DshCliError::RockApi(format!(
            "connected to rock api but health status not ok (status {})",
            health.status
          )))
        }
      }
      Err(rock_api_error) => match &rock_api_error {
        RockApiError::Reqwest { .. } => {
          debug!("rock api reqwest error ({})", rock_api_error);
          err!("could not connect to rock api, check network or vpn")
        }
        _ => Err(DshCliError::from(rock_api_error)),
      },
    }
  }

  /// Create `CsrBuilder` with default parameters for KPN certificate.
  ///
  /// <span style="color:#a00000">This method is only available when the <code>rcgen</code>
  /// feature is enabled.</span>
  ///
  /// Creates a `CsrBuilder` with the proper default settings for a signing request via the KPN
  /// _RoCK API_. The following parameters are set:
  /// * `country` - `"NL"`,
  /// * `key_usages` - `DigitalSignature`, `KeyEncipherment` and
  ///   `ContentCommitment (NonRepudiation)`,
  /// * `locality` - `"Rotterdam"`,
  /// * `organization` - `"Koninklijke KPN N.V."`,
  /// * `rsa_key_size` - recommended rsa key size from `pki_connector`,
  /// * `signature_algorithm` - recommended signature algorithm from `pki_connector`,
  /// * `state` - `"Zuid-Holland"`.
  ///
  /// The other parameters have their default values (`None` or empty).
  fn default_csr_builder(&self) -> DshCliResult<CsrBuilder> {
    Ok(
      CsrBuilder::default()
        .country(KPN_DN_COUNTRY_NAME)
        .key_usages(vec![
          KeyUsagePurpose::DigitalSignature,
          KeyUsagePurpose::KeyEncipherment,
          KeyUsagePurpose::ContentCommitment,
        ])
        .locality(KPN_DN_LOCALITY_NAME)
        .organization(KPN_DN_ORGANIZATION_NAME)
        .rsa_key_size(self.pki_connector.recommended_rsa_key_size())
        .signature_algorithm(self.pki_connector.recommended_signature_algorithm())
        .state(KPN_DN_STATE_OR_PROVINCE_NAME),
    )
  }

  async fn existing_certificate(&self, vhost_domain: &str, context: Option<(&Context, u64)>) -> DshCliResult<Option<String>> {
    let query: HashMap<CertsParameter, String> = HashMap::from([(CertsParameter::Status, "AC".to_string()), (CertsParameter::Domain, vhost_domain.to_string())]);
    match self
      .client
      .certs(&query)
      .await?
      .results
      .into_iter()
      .find(|certificate| certificate.cn == vhost_domain)
    {
      Some(certificate) => {
        let id = certificate.id.to_string();
        if let Some((context, expiration_days)) = context {
          UnitFormatter::new(&id, &ROCK_CERTIFICATE_LABELS_SHOW, context).print(&(&certificate, Some(expiration_days)), None)?;
        }
        Ok(Some(id))
      }
      None => Ok(None),
    }
  }

  async fn list(&self, domain: &str, context: &Context, expiration_days: u64) -> DshCliResult<()> {
    let query_active: HashMap<CertsParameter, String> = HashMap::from([(CertsParameter::Status, "AC".to_string()), (CertsParameter::Domain, domain.to_string())]);
    let mut certificates = self.client.certs(&query_active).await?.results;
    let query_revoked: HashMap<CertsParameter, String> = HashMap::from([(CertsParameter::Status, "RE".to_string()), (CertsParameter::Domain, domain.to_string())]);
    let mut revoked_certificates = self.client.certs(&query_revoked).await?.results;
    certificates.append(&mut revoked_certificates);
    certificates.sort_by(|cert_a, cert_b| cert_a.cn.cmp(&cert_b.cn));
    let mut formatter = ListFormatter::new(&ROCK_CERTIFICATE_LABELS_LIST, context);
    for certificate in &certificates {
      formatter.push_target_id_value_owned(certificate.cn.clone(), (certificate, Some(expiration_days)));
    }
    formatter.print(None)?;

    Ok(())
  }

  async fn sign_certificate(&self, csr: &CertificateSigningRequest, context: Option<(&Context, u64)>) -> DshCliResult<(String, String)> {
    let signed_certificate = self.client.get_signed_certificate(csr, self.pki_connector.clone()).await?;
    // Due to a bug in the RoCK API (see KNOWN_ISSUES.md in rock_api crate) we need to load
    // the certificate again to obtain the proper not_before and not_after values.
    // TODO Check whether this is solved in latest version of the RoCK API?
    if let Some((context, _)) = context {
      context.print_explanation("reload certificate");
    }
    let reloaded_certificate = self.client.cert(signed_certificate.id).await?;
    if let Some((context, expiration_days)) = context {
      UnitFormatter::new(reloaded_certificate.id, &ROCK_CERTIFICATE_LABELS_SHOW, context).print(&(&reloaded_certificate, Some(expiration_days)), None)?;
    }
    Ok((
      reloaded_certificate.id.to_string(),
      reloaded_certificate.cert.ok_or_else(|| DshCliError::from("missing certificate"))?,
    ))
  }
}

#[cfg(feature = "rock")]
const ROCK_CERTIFICATE_LABELS_LIST: [RockCertificateLabel; 5] =
  [RockCertificateLabel::CommonName, RockCertificateLabel::AdministrativeGroup, RockCertificateLabel::NotAfter, RockCertificateLabel::Status, RockCertificateLabel::Id];

#[cfg(feature = "rock")]
const ROCK_CERTIFICATE_LABELS_SHOW: [RockCertificateLabel; 9] = [
  RockCertificateLabel::AdministrativeGroup,
  RockCertificateLabel::AltNames,
  RockCertificateLabel::CommonName,
  RockCertificateLabel::ConnectorName,
  RockCertificateLabel::Id,
  RockCertificateLabel::ManagedByGroup,
  RockCertificateLabel::NotAfter,
  RockCertificateLabel::NotBefore,
  RockCertificateLabel::Status,
];

#[cfg(feature = "rock")]
#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum RockCertificateLabel {
  AdministrativeGroup,
  AltNames,
  CommonName,
  ConnectorName,
  Id,
  ManagedByGroup,
  NotAfter,
  NotBefore,
  Status,
}

#[cfg(feature = "rock")]
impl Label for RockCertificateLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AdministrativeGroup => "administrative group",
      Self::AltNames => "alt names",
      Self::CommonName => "common name",
      Self::ConnectorName => "connector name",
      Self::Id => "id",
      Self::ManagedByGroup => "managed by group",
      Self::NotAfter => "not after",
      Self::NotBefore => "not before",
      Self::Status => "status",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, RockCertificateLabel::Id)
  }
}

#[cfg(feature = "rock")]
impl SubjectFormatter<RockCertificateLabel> for (&Certificate, Option<u64>) {
  fn value(&self, label: &RockCertificateLabel, _target_id: &str) -> Value {
    use chrono::{DateTime, Utc};
    use time::UtcDateTime;
    fn time_to_chrono(time: &UtcDateTime) -> DateTime<Utc> {
      DateTime::from_timestamp_secs(time.unix_timestamp()).unwrap_or_default()
    }
    let (certificate, days): (&Certificate, Option<u64>) = *self;
    match label {
      RockCertificateLabel::AdministrativeGroup => Value::plain(certificate.ad_group.clone()),
      RockCertificateLabel::AltNames => match &certificate.alt_names {
        Some(alt_names) => Value::non_empty_or_hide(&alt_names.iter().map(|alt_name| alt_name.cn.to_string()).collect_vec()),
        None => Value::hide(),
      },
      RockCertificateLabel::CommonName => Value::plain(certificate.cn.clone()),
      RockCertificateLabel::ConnectorName => Value::plain(certificate.connector_name.clone()),
      RockCertificateLabel::Id => Value::plain(certificate.id),
      RockCertificateLabel::ManagedByGroup => Value::plain(certificate.managed_by_group.clone()),
      RockCertificateLabel::NotAfter => Value::datetime_expired(&time_to_chrono(&certificate.not_after), days),
      RockCertificateLabel::NotBefore => Value::datetime(&time_to_chrono(&certificate.not_before)),
      RockCertificateLabel::Status => {
        if certificate.status != "AC" {
          Value::warn(certificate.status.clone())
        } else {
          Value::plain(certificate.status.clone())
        }
      }
    }
  }
}
