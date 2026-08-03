use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::subject::Requirements;
#[cfg(feature = "rock")]
use crate::subjects::vhost::labels::RockCertificateLabel;
use crate::subjects::vhost::{VhostListValue, VHOST_LIST_LABELS};
use crate::subjects::DEPENDANT_LABELS_LIST;
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::vhost::{VhostInjection, VhostString};
use dsh_api::Dependant;
use itertools::Itertools;
use std::str::FromStr;

// static VHOST_BUNDLE_LABELS_CREATE: [VhostBundleLabel; 5] =
//   [VhostBundleLabel::Platform, VhostBundleLabel::Tenant, VhostBundleLabel::BundleName, VhostBundleLabel::CaCommonName, VhostBundleLabel::VhostZone];

#[cfg(feature = "rock")]
pub(crate) struct VhostAddCertificate {}

#[cfg(feature = "rock")]
#[async_trait]
impl CommandExecutor for VhostAddCertificate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    use crate::error::DshCliError;
    use crate::formatters::unit_formatter::UnitFormatter;
    use crate::global_options::get_expiration_days;
    use crate::subjects::certificate::capabilities::CERTIFICATE_LABELS_SHOW;
    use crate::target_platform::get_target_platform;
    use crate::target_tenant::get_target_tenant;
    use crate::{cli_error, err};
    use dsh_api::platform::VhostZone;
    use dsh_api::types::{Certificate, CertificateStatus, Secret};
    use futures::join;
    use regex::Regex;
    use rock_api::client::CertsParameter;
    use rock_api::client::RockApiClient;
    use std::collections::HashMap;
    use std::sync::LazyLock;

    // TODO    vhost_subdomain need not be the same as certificate id

    static VHOST_SUBDOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9\-]*[a-z0-9]$").unwrap());

    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let vhost_subdomain = target.unwrap_or_else(|| unreachable!());

    if !VHOST_SUBDOMAIN_REGEX.is_match(&vhost_subdomain) {
      return err!("illegal vhost subdomain '{}'", vhost_subdomain);
    }

    let vhost_domain = platform.tenant_private_vhost_domain(&tenant, &vhost_subdomain)?;

    let rock_client = match RockApiClient::header_based_from_auth_token_file() {
      Ok(client) => client,
      Err(error) => {
        context.print_error(error.to_string());
        return err!("failed to connect to rock api");
      }
    };

    // Check if RoCK supports this platform and tenant
    let tenant_domain_string = format!("$.{}", platform.tenant_domain(&tenant, VhostZone::Private)?);
    let authorized_domain_list = rock_client.domain_list().await?;
    if authorized_domain_list
      .domains
      .iter()
      .find(|authorized_domain| **authorized_domain == tenant_domain_string)
      .is_none()
    {
      return err!("authenticated user has no authorization for domain '{}' at rock api", tenant_domain_string);
    }

    context.print_explanation(format!("add certificate for vhost domain '{}'", vhost_domain));

    // Check if the DSH already has a certificate for this vhost subdomain.

    let expiration_days = get_expiration_days(matches, context.settings())?;

    let certificates: Vec<(String, CertificateStatus)> = client.certificates().await?;
    let conflicting_dsh_certificates: Vec<&(String, CertificateStatus)> = certificates
      .iter()
      .filter(|(_, certificate_status)| {
        certificate_status
          .actual
          .as_ref()
          .is_some_and(|actual_certificate| actual_certificate.distinguished_name.contains(&vhost_domain) || actual_certificate.dns_names.contains(&vhost_domain))
      })
      .collect_vec();
    if !conflicting_dsh_certificates.is_empty() {
      for (certificate_id, certificate_status) in conflicting_dsh_certificates {
        context.print_warning(format!("dsh certificate '{}' already protects vhost domain", certificate_id));
        UnitFormatter::new(certificate_id.clone(), &CERTIFICATE_LABELS_SHOW, context).print(&(certificate_status.actual.as_ref().unwrap(), Some(expiration_days), None), None)?;
      }
      return err!("delete conflicting dsh certificate(s) first");
    }

    // Check if RoCK already has a certificate for this vhost domain
    let query: HashMap<CertsParameter, String> = HashMap::from([(CertsParameter::Status, "AC".to_string())]);
    let active_rock_certificates = rock_client.certs(&query).await?;
    if let Some(active_certificate) = active_rock_certificates.results.into_iter().find(|certificate| certificate.cn == vhost_domain) {
      context.print_warning(format!("found existing certificate for vhost domain '{}' in rock database", vhost_domain));
      UnitFormatter::new(active_certificate.id, &ROCK_CERTIFICATE_LABELS, context).print(&(&active_certificate, Some(expiration_days)), None)?;
      if !context.confirmed("overwrite existing rock certificate")? {
        context.print_warning("cancelled");
        return Ok(());
      }
    }

    // TODO Generate csr and certificate separately, for improved logging
    context.print_explanation("generate new certificate");
    let (signed_certificate, key_pair) = rock_client.generate_signed_server_certificate(vhost_domain.clone(), None, vec![]).await?;

    // TODO Due to a bug in the RoCK API (see KNOWN_ISSUES.md in rock_api crate) we need to load
    // the certificate again from the seb service.
    context.print_explanation("reload certificate");
    let signed_certificate = rock_client.cert(signed_certificate.id).await?;
    UnitFormatter::new(signed_certificate.id, &ROCK_CERTIFICATE_LABELS, context).print(&(&signed_certificate, Some(expiration_days)), None)?;

    if !context.confirmed(format!("deploy vhost certificate '{}' for domain '{}'?", signed_certificate.id, vhost_domain))? {
      return err!("cancelled, vhost certificate '{}' not deployed", signed_certificate.id);
    }

    context.print_explanation("deploy new vhost certificate");

    let certificate_name = format!("{}-cert", vhost_subdomain);
    let key_secret_name = format!("{}-cert-key", vhost_subdomain);
    let certificate_secret_name = format!("{}-server-cert", vhost_subdomain);

    let (certificate_result, key_secret_result, certificate_secret_result) = join!(
      client.get_certificate(&certificate_name),
      client.get_secret(&key_secret_name),
      client.get_secret(&certificate_secret_name),
    );
    if certificate_result.is_ok() {
      context.print_error(format!("certificate '{}' already exists", certificate_name));
    }
    if key_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", key_secret_name));
    }
    if certificate_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", certificate_secret_name));
    }
    if certificate_result.is_ok() || key_secret_result.is_ok() || certificate_secret_result.is_ok() {
      return err!("cancelled, some resources already exist");
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, secret '{}' not deployed", key_secret_name));
    } else {
      let key_pair_secret = key_pair.serialize_pem();
      let secret = Secret::new(&key_secret_name, &key_pair_secret);
      client.post_secret(&secret).await?;
      context.print_outcome(format!("secret '{}' deployed", &key_secret_name));
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, secret '{}' not deployed", certificate_secret_name));
    } else {
      let certificate_pem = signed_certificate.cert.ok_or_else(|| DshCliError::from("missing certificate"))?;
      // TODO Add intermediate kpn private certificate
      let certificate_secret = format!("{}\n{}", certificate_pem.trim(), INTERMEDIATE_KPN_PRIVATE_NV_TB_G1_CRT);
      let secret = Secret::new(&certificate_secret_name, &certificate_secret);
      client.post_secret(&secret).await?;
      context.print_outcome(format!("secret '{}' deployed", &certificate_secret_name));
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, certificate '{}' not deployed", certificate_name));
    } else {
      let certificate_body = Certificate { cert_chain_secret: certificate_secret_name, key_secret: key_secret_name, passphrase_secret: None };
      client
        .put_certificate_configuration(&certificate_name, &certificate_body)
        .await
        .map_err(|error| cli_error!("error deploying certificate configuration '{}' ({})", certificate_name, error))?;
      context.print_outcome(format!("certificate '{}' deployed", &certificate_name));
    }

    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct VhostList {}

#[async_trait]
impl CommandExecutor for VhostList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list configured vhosts");
    let start_instant = context.now();
    let applications = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);
    let (include_started, include_stopped) = include_started_stopped(matches);
    let mut vhost_list_values = applications
      .iter()
      .filter(|(_, application)| (application.instances > 0 && include_started) || (application.instances == 0 && include_stopped))
      .flat_map(|(application_id, application)| {
        application
          .exposed_ports
          .iter()
          .filter_map(|(port, port_mapping)| match port_mapping.vhost {
            Some(ref vhost_string) => match VhostString::from_str(vhost_string) {
              Ok(vhost) => Some(VhostListValue {
                vhost: vhost.vhost_name,
                zone: vhost.zone,
                tenant: vhost.tenant_name,
                kafka_flag: vhost.kafka,
                service_id: application_id.to_string(),
                instances: application.instances,
                port: port.to_string(),
                port_mapping: port_mapping.clone(),
              }),
              Err(_) => None,
            },
            None => None,
          })
          .collect_vec()
      })
      .collect_vec();
    if vhost_list_values.is_empty() {
      context.print_outcome("no vhosts configured");
      Ok(())
    } else {
      vhost_list_values.sort_by(|a, b| (&a.vhost, &a.service_id).cmp(&(&b.vhost, &b.service_id)));
      let mut formatter = ListFormatter::new(&VHOST_LIST_LABELS, context);
      formatter.push_values(&vhost_list_values);
      formatter.print(None)
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct VhostListUsage {}

#[async_trait]
impl CommandExecutor for VhostListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list vhosts with services and apps that use them");
    let start_instant = context.now();
    let vhosts_with_usage: Vec<(String, Vec<Dependant<VhostInjection>>)> = client.vhosts_with_dependants().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, context);
    for (vhost, dependants) in &vhosts_with_usage {
      for dependant in dependants {
        formatter.push_target_id_value(vhost.clone(), dependant);
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[cfg(feature = "rock")]
pub(crate) struct VhostUpdateCertificate {}

#[cfg(feature = "rock")]
#[async_trait]
impl CommandExecutor for VhostUpdateCertificate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    use crate::error::DshCliError;
    use crate::formatters::unit_formatter::UnitFormatter;
    use crate::global_options::get_expiration_days;
    use crate::subjects::certificate::capabilities::CERTIFICATE_LABELS_SHOW;
    use crate::target_platform::get_target_platform;
    use crate::target_tenant::get_target_tenant;
    use crate::{cli_error, err};
    use dsh_api::platform::VhostZone;
    use dsh_api::types::{Certificate, CertificateStatus, Secret};
    use futures::join;
    use regex::Regex;
    use rock_api::client::CertsParameter;
    use rock_api::client::RockApiClient;
    use std::collections::HashMap;
    use std::sync::LazyLock;

    static VHOST_SUBDOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9\-]*[a-z0-9]$").unwrap());

    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let vhost_subdomain = target.unwrap_or_else(|| unreachable!());

    if !VHOST_SUBDOMAIN_REGEX.is_match(&vhost_subdomain) {
      return err!("illegal vhost subdomain '{}'", vhost_subdomain);
    }

    let vhost_domain = platform.tenant_private_vhost_domain(&tenant, &vhost_subdomain)?;

    let rock_client = match RockApiClient::header_based_from_auth_token_file() {
      Ok(client) => client,
      Err(error) => {
        context.print_error(error.to_string());
        return err!("failed to connect to rock api");
      }
    };

    // Check if RoCK supports this platform and tenant
    let tenant_domain_string = format!("$.{}", platform.tenant_domain(&tenant, VhostZone::Private)?);
    let authorized_domain_list = rock_client.domain_list().await?;
    if authorized_domain_list
      .domains
      .iter()
      .find(|authorized_domain| **authorized_domain == tenant_domain_string)
      .is_none()
    {
      return err!("authenticated user has no authorization for domain '{}' at rock api", tenant_domain_string);
    }

    context.print_explanation(format!("update certificate for vhost domain '{}'", vhost_domain));

    // Check if the DSH has a certificate for this vhost subdomain.

    let expiration_days = get_expiration_days(matches, context.settings())?;
    let certificates: Vec<(String, CertificateStatus)> = client.certificates().await?;
    let conflicting_dsh_certificates: Vec<&(String, CertificateStatus)> = certificates
      .iter()
      .filter(|(_, certificate_status)| {
        certificate_status
          .actual
          .as_ref()
          .is_some_and(|actual_certificate| actual_certificate.distinguished_name.contains(&vhost_domain) || actual_certificate.dns_names.contains(&vhost_domain))
      })
      .collect_vec();
    if !conflicting_dsh_certificates.is_empty() {
      for (certificate_id, certificate_status) in conflicting_dsh_certificates {
        context.print_warning(format!("dsh certificate '{}' already protects vhost domain", certificate_id));
        UnitFormatter::new(certificate_id.clone(), &CERTIFICATE_LABELS_SHOW, context).print(&(certificate_status.actual.as_ref().unwrap(), Some(expiration_days), None), None)?;
      }
      return err!("delete conflicting dsh certificate(s) first");
    }

    // Check if RoCK already has a certificate for this vhost domain
    let query: HashMap<CertsParameter, String> = HashMap::from([(CertsParameter::Status, "AC".to_string())]);
    let active_rock_certificates = rock_client.certs(&query).await?;
    if let Some(active_certificate) = active_rock_certificates.results.into_iter().find(|certificate| certificate.cn == vhost_domain) {
      context.print_warning(format!("found existing certificate for vhost domain '{}' in rock database", vhost_domain));
      UnitFormatter::new(active_certificate.id, &ROCK_CERTIFICATE_LABELS, context).print(&(&active_certificate, Some(expiration_days)), None)?;
      if !context.confirmed("overwrite existing rock certificate")? {
        context.print_warning("cancelled");
        return Ok(());
      }
    }

    // TODO Generate csr and certificate separately, for improved logging
    context.print_explanation("generate new certificate");
    let (signed_certificate, key_pair) = rock_client.generate_signed_server_certificate(vhost_domain.clone(), None, vec![]).await?;

    // TODO Due to a bug in the RoCK API (see KNOWN_ISSUES.md in rock_api crate) we need to load
    // the certificate again from the seb service.
    context.print_explanation("reload certificate");
    let signed_certificate = rock_client.cert(signed_certificate.id).await?;
    UnitFormatter::new(signed_certificate.id, &ROCK_CERTIFICATE_LABELS, context).print(&(&signed_certificate, Some(expiration_days)), None)?;

    if !context.confirmed(format!("deploy vhost certificate '{}' for domain '{}'?", signed_certificate.id, vhost_domain))? {
      return err!("cancelled, vhost certificate '{}' not deployed", signed_certificate.id);
    }

    context.print_explanation("deploy new vhost certificate");

    let certificate_name = format!("{}-cert", vhost_subdomain);
    let key_secret_name = format!("{}-cert-key", vhost_subdomain);
    let certificate_secret_name = format!("{}-server-cert", vhost_subdomain);

    let (certificate_result, key_secret_result, certificate_secret_result) = join!(
      client.get_certificate(&certificate_name),
      client.get_secret(&key_secret_name),
      client.get_secret(&certificate_secret_name),
    );
    if certificate_result.is_ok() {
      context.print_error(format!("certificate '{}' already exists", certificate_name));
    }
    if key_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", key_secret_name));
    }
    if certificate_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", certificate_secret_name));
    }
    if certificate_result.is_ok() || key_secret_result.is_ok() || certificate_secret_result.is_ok() {
      return err!("cancelled, some resources already exist");
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, secret '{}' not deployed", key_secret_name));
    } else {
      let key_pair_secret = key_pair.serialize_pem();
      let secret = Secret::new(&key_secret_name, &key_pair_secret);
      client.post_secret(&secret).await?;
      context.print_outcome(format!("secret '{}' deployed", &key_secret_name));
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, secret '{}' not deployed", certificate_secret_name));
    } else {
      let certificate_pem = signed_certificate.cert.ok_or_else(|| DshCliError::from("missing certificate"))?;
      // TODO Add intermediate kpn private certificate
      let certificate_secret = format!("{}\n{}", certificate_pem.trim(), INTERMEDIATE_KPN_PRIVATE_NV_TB_G1_CRT);
      let secret = Secret::new(&certificate_secret_name, &certificate_secret);
      client.post_secret(&secret).await?;
      context.print_outcome(format!("secret '{}' deployed", &certificate_secret_name));
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, certificate '{}' not deployed", certificate_name));
    } else {
      let certificate_body = Certificate { cert_chain_secret: certificate_secret_name, key_secret: key_secret_name, passphrase_secret: None };
      client
        .put_certificate_configuration(&certificate_name, &certificate_body)
        .await
        .map_err(|error| cli_error!("error deploying certificate configuration '{}' ({})", certificate_name, error))?;
      context.print_outcome(format!("certificate '{}' deployed", &certificate_name));
    }

    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[cfg(feature = "rock")]
const ROCK_CERTIFICATE_LABELS: [RockCertificateLabel; 9] = [
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

const INTERMEDIATE_KPN_PRIVATE_NV_TB_G1_CRT: &str = r#"-----BEGIN CERTIFICATE-----
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
9KK2q2I+jyufvDBq870RDLdBQQxI7ZgZwtA=
-----END CERTIFICATE-----"#;

// let _url = "https://artifacts.kpn.org/artifactory/kpn-pki/intermediate.kpn.private.nv.tb-g1.crt";
