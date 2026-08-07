use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::Requirements;
use crate::subjects::certificate::get_relative_distinguished_name;
#[cfg(feature = "rock")]
use crate::subjects::vhost::labels::RockCertificateLabel;
use crate::subjects::vhost::labels::{VhostListLabel, VhostValue};
use crate::subjects::{DependantLabel, DEPENDANT_LABELS_LIST};
use crate::{include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{ActualCertificate, CertificateStatus};
use dsh_api::vhost::{VhostInjection, VhostString};
use dsh_api::{Dependant, DependantApp};
use futures::future::{join_all, try_join};
use itertools::Itertools;
#[cfg(feature = "rock")]
use rock_api::rcgen::INTERMEDIATE_KPN_PRIVATE_NV_TB_G1_CRT;
use std::str::FromStr;

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

static VHOST_LIST_LABELS: [VhostListLabel; 7] =
  [VhostListLabel::Vhost, VhostListLabel::Zone, VhostListLabel::Kind, VhostListLabel::ServiceId, VhostListLabel::Instances, VhostListLabel::Url, VhostListLabel::Cert];

pub(crate) struct VhostList {}

#[async_trait]
impl CommandExecutor for VhostList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list configured vhosts");
    let start_instant = context.now();
    let applications = client.get_application_configuration_map().await?;

    let certs: Vec<(String, CertificateStatus)> = client.certificates().await?;

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
              Ok(vhost) => {
                let url = client.platform().url_from_vhost_string(&vhost, Some(client.tenant_name())).ok();
                let cert = url.as_ref().map(|url| find_cert(url, &certs)).unwrap_or_default().map(|(cert_id, _)| cert_id);
                Some(VhostValue {
                  vhost: vhost.vhost_name,
                  zone: vhost.zone,
                  tenant: vhost.tenant_name,
                  kafka_flag: vhost.kafka,
                  service_id: application_id.to_string(),
                  instances: application.instances,
                  port: port.to_string(),
                  port_mapping: port_mapping.clone(),
                  url,
                  cert,
                })
              }
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

fn find_cert(target_common_name: &str, certs: &[(String, CertificateStatus)]) -> Option<(String, ActualCertificate)> {
  certs.iter().find_map(|(certificate_id, certificate_status)| match &certificate_status.actual {
    Some(actual_certificate) => match get_relative_distinguished_name(actual_certificate.distinguished_name.as_str(), "CN") {
      Some(common_name) => {
        if target_common_name == common_name {
          Some((certificate_id.clone(), actual_certificate.clone()))
        } else {
          None
        }
      }
      None => None,
    },
    None => None,
  })
}

static DEPENDANT_LABELS_LIST_APPS: [DependantLabel; 2] = [DependantLabel::DependantId, DependantLabel::Dependencies];

pub(crate) struct VhostListApps {}

#[async_trait]
impl CommandExecutor for VhostListApps {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are used as resources in apps will be listed here");
    context.print_explanation("list vhosts with apps that use them as resources");
    let start_instant = context.now();
    let vhosts_with_app_usage: Vec<(String, Vec<DependantApp>)> = client.vhosts_with_dependant_apps().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST_APPS, context);
    for (vhost, dependant_apps) in &vhosts_with_app_usage {
      for dependant in dependant_apps {
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

static VHOST_SHOW_LABELS: [VhostListLabel; 14] = [
  VhostListLabel::Vhost,
  VhostListLabel::Zone,
  VhostListLabel::Kind,
  VhostListLabel::ServiceId,
  VhostListLabel::Port,
  VhostListLabel::Instances,
  VhostListLabel::Auth,
  VhostListLabel::Tenant,
  VhostListLabel::Mode,
  VhostListLabel::Paths,
  VhostListLabel::Tls,
  VhostListLabel::Whitelist,
  VhostListLabel::Url,
  VhostListLabel::Cert,
];

pub(crate) struct VhostShow {}

#[async_trait]
impl CommandExecutor for VhostShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let vhost_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for vhost '{}'", vhost_id));
    let start_instant = context.now();
    let (applications, certificate_ids) = try_join(client.get_application_configuration_map(), client.get_certificate_ids()).await?;
    // let applications = client.get_application_configuration_map().await?;
    // let certificate_ids = client.get_certificate_ids().await?;
    let _certificate_statuses = join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate(certificate_id))).await;

    //     let certs = certificate_ids.zip() certificate_statuses.iter().filter_map(|certificate_status| {
    //       if let Ok(certificate_status) = certificate_status {
    //         let actual_certificate = certificate_status.actual.unwrap();
    //        if let Some(common_name) = get_relative_distinguished_name(&actual_certificate.distinguished_name, "CN") {
    // if common_name ==
    //        }
    //         actual_certificate.dns_names;
    //         let configuration = b.configuration;
    //         None
    //       } else {
    //         None
    //       }
    //     }).collect_vec();

    let certs: Vec<(String, CertificateStatus)> = client.certificates().await?;

    context.print_execution_time(start_instant);
    let mut vhosts = applications
      .iter()
      .flat_map(|(application_id, application)| {
        application
          .exposed_ports
          .iter()
          .filter_map(|(port, port_mapping)| match port_mapping.vhost {
            Some(ref vhost_string) => match VhostString::from_str(vhost_string) {
              Ok(vhost) => {
                if vhost.vhost_name == vhost_id {
                  let url = client.platform().url_from_vhost_string(&vhost, Some(client.tenant_name())).ok();
                  let cert = url.as_ref().map(|url| find_cert(url, &certs)).unwrap_or_default().map(|(cert_id, _)| cert_id);
                  Some(VhostValue {
                    vhost: vhost.vhost_name,
                    zone: vhost.zone,
                    tenant: vhost.tenant_name,
                    kafka_flag: vhost.kafka,
                    service_id: application_id.to_string(),
                    instances: application.instances,
                    port: port.to_string(),
                    port_mapping: port_mapping.clone(),
                    url,
                    cert,
                  })
                } else {
                  None
                }
              }
              Err(_) => None,
            },
            None => None,
          })
          .collect_vec()
      })
      .collect_vec();
    if vhosts.is_empty() {
      context.print_outcome(format!("vhost '{}' not configured", vhost_id));
    } else {
      vhosts.sort_by(|a, b| (&a.vhost, &a.service_id).cmp(&(&b.vhost, &b.service_id)));
      for vhost in vhosts {
        UnitFormatter::new(vhost.vhost.clone(), &VHOST_SHOW_LABELS, context).print(&vhost, None)?;
      }
    }
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
