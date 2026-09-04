use crate::bundle::{create_certificate_authority, get_certificate_authority_interactive};
use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::Requirements;
use crate::subjects::certificate::get_relative_distinguished_name;
use crate::subjects::proxy::options::{get_vhost_zone_interactive, ATTACH_CA_CHAIN_OPTION};
use crate::subjects::vhost::labels::{VhostListLabel, VhostValue};
use crate::subjects::{DependantLabel, DEPENDANT_LABELS_LIST};
use crate::target_platform::get_target_platform;
use crate::target_tenant::get_target_tenant;
use crate::{err, include_started_stopped, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::{ActualCertificate, Application, CertificateStatus};
use dsh_api::vhost::{VhostInjection, VhostString};
use dsh_api::{Dependant, DependantApp};
use futures::future::try_join;
use futures::try_join;
use itertools::Itertools;
use std::collections::HashMap;
use std::str::FromStr;

#[cfg(feature = "rock")]
pub(crate) struct VhostAddCertificate {}

#[cfg(feature = "rock")]
#[async_trait]
impl CommandExecutor for VhostAddCertificate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
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
    use std::sync::LazyLock;

    // TODO    vhost_subdomain need not be the same as certificate id

    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let vhost_name = target.unwrap_or_else(|| unreachable!());
    let expiration_days = get_expiration_days(matches, context.settings())?;

    static VHOST_SUBDOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9\-]*[a-z0-9]$").unwrap());
    if !VHOST_SUBDOMAIN_REGEX.is_match(&vhost_name) {
      return err!("illegal vhost subdomain '{}'", vhost_name);
    }

    let (applications, certificates) = try_join!(client.get_application_configuration_map(), client.certificates())?;

    let vhost_applications: Vec<(String, &Application, VhostString)> = applications_using_vhost(&vhost_name, &applications);
    let default_vhost_zone: Option<VhostZone> = if vhost_applications.len() > 1 {
      context.print_warning(format!(
        "vhost '{}' used in multiple services ({})",
        vhost_name,
        vhost_applications.iter().map(|(application_id, _, _)| application_id).join(", ")
      ));
      None
    } else {
      match vhost_applications.first() {
        Some((service_id, _, vhost_string)) => {
          if vhost_string.kafka {
            return err!("vhost '{}' used in kafka proxy service '{}'", vhost_name, service_id);
          } else {
            context.print_outcome(format!("vhost '{}' used in service '{}'", vhost_name, service_id));
            vhost_string.zone.clone()
          }
        }
        None => {
          context.print_warning(format!("vhost '{}' not used in any service", vhost_name));
          if context.confirmed("continue")? {
            None
          } else {
            return Ok(());
          }
        }
      }
    };
    let vhost_zone = get_vhost_zone_interactive(matches, context, default_vhost_zone.unwrap_or(VhostZone::Private))?;

    let certificate_authority_id = get_certificate_authority_interactive(matches, context)?;
    let certificate_authority = create_certificate_authority(certificate_authority_id).await?;

    // Check if RoCK supports this platform and tenant
    let tenant_domain = &platform.tenant_domain(&tenant, vhost_zone.clone())?;
    if !certificate_authority.authorization_check(tenant_domain).await? {
      return err!("authenticated user has no authorization for tenant domain '{}' at rock api", tenant_domain);
    }

    let vhost_domain = match vhost_zone {
      VhostZone::Private => platform.tenant_private_vhost_domain(&tenant, &vhost_name)?,
      VhostZone::Public => platform.public_vhost_domain(&vhost_name),
    };

    context.print_explanation(format!("add certificate for vhost domain '{}'", vhost_domain));

    // Check if the DSH already has a certificate for this vhost subdomain.
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

    if let Some(existing_certificate_id) = certificate_authority.existing_certificate(&vhost_domain, Some((context, expiration_days))).await? {
      context.print_warning(format!("found existing certificate for vhost domain '{}' in rock database", vhost_domain));
      // TODO What to do here?
      if !context.confirmed(format!("overwrite existing rock certificate '{}'", existing_certificate_id))? {
        context.print_warning("cancelled");
        return Ok(());
      }
    }

    context.print_explanation("generate server certificate signing request");

    let builder = certificate_authority.default_csr_builder()?.common_name(&vhost_domain).server_certificate();
    let (csr, key_pair) = builder.build()?;

    context.print_explanation("generate signed certificate");
    let (signed_certificate_id, signed_certificate_pem) = certificate_authority.sign_certificate(&csr, Some((context, expiration_days))).await?;

    if !context.confirmed(format!("deploy vhost certificate '{}' for domain '{}'?", signed_certificate_id, vhost_domain))? {
      return err!("cancelled, vhost certificate '{}' not deployed", signed_certificate_id);
    }

    let certificate_name = format!("{}-cert", vhost_name);
    let key_secret_name = format!("{}-cert-key", vhost_name);
    let certificate_secret_name = format!("{}-server-cert", vhost_name);

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
      context.print_warning(format!("dry-run mode, key secret '{}' not deployed", key_secret_name));
    } else {
      let key_pair_secret = key_pair.serialize_pem();
      let secret = Secret::new(&key_secret_name, &key_pair_secret);
      client.post_secret(&secret).await?;
      context.print_outcome(format!("key secret '{}' deployed", &key_secret_name));
    }

    if context.dry_run() {
      context.print_warning(format!("dry-run mode, certificate secret '{}' not deployed", certificate_secret_name));
    } else {
      let attach_ca_chain = matches.get_one::<bool>(ATTACH_CA_CHAIN_OPTION).cloned().unwrap_or(true);
      let certificate_secret = if attach_ca_chain {
        context.print_outcome("attach intermediate and root certificates");
        certificate_authority.attach_ca_chain(&signed_certificate_pem).await?
      } else {
        signed_certificate_pem
      };
      let secret = Secret::new(&certificate_secret_name, &certificate_secret);
      client.post_secret(&secret).await?;
      context.print_outcome(format!("certificate secret '{}' deployed", &certificate_secret_name));
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
  [VhostListLabel::Vhost, VhostListLabel::Zone, VhostListLabel::Kind, VhostListLabel::ServiceId, VhostListLabel::Instances, VhostListLabel::Domain, VhostListLabel::Cert];

pub(crate) struct VhostList {}

#[async_trait]
impl CommandExecutor for VhostList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_warning("only vhosts that are actually used in service configurations will be listed here");
    context.print_explanation("list configured vhosts");

    let start_instant = context.now();
    let (applications, certs) = try_join!(client.get_application_configuration_map(), client.certificates())?;
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
                let domain = client.platform().domain_from_vhost_string(&vhost, Some(client.tenant_name())).ok();
                let cert = domain
                  .as_ref()
                  .map(|domain| find_cert(domain, &certs))
                  .unwrap_or_default()
                  .map(|(cert_id, _)| cert_id);
                Some(VhostValue {
                  cert,
                  domain,
                  instances: application.instances,
                  kafka_flag: vhost.kafka,
                  port: port.to_string(),
                  port_mapping: port_mapping.clone(),
                  service_id: application_id.to_string(),
                  tenant: vhost.tenant_name,
                  vhost: vhost.vhost_name,
                  zone: vhost.zone,
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

pub(crate) struct VhostOpen {}

#[async_trait]
impl CommandExecutor for VhostOpen {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let vhost_name = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let applications = client.get_application_configuration_map().await?;
    context.print_execution_time(start_instant);

    let vhost_applications: Vec<(String, &Application, VhostString)> = applications_using_vhost(&vhost_name, &applications);

    if vhost_applications.len() > 1 {
      return err!(
        "vhost '{}' configured in multiple services ({})",
        vhost_name,
        vhost_applications.iter().map(|(application_id, _, _)| application_id).join(", ")
      );
    } else {
      match vhost_applications.first() {
        Some((service_id, application, vhost_string)) => {
          if vhost_string.kafka {
            context.print_warning(format!("vhost '{}' is configured in kafka proxy service '{}'", vhost_name, service_id))
          } else if application.instances > 0 {
            let url = platform.url_from_vhost_string(vhost_string, Some(&tenant_name))?;
            context.open_url(
              &url,
              format!("'{}' for tenant '{}@{}' and service '{}'", vhost_name, platform, tenant_name, service_id),
            );
          } else {
            context.print_warning(format!(
              "vhost '{}' is configured in service '{}', which is currently not running",
              vhost_name, service_id
            ))
          }
        }
        None => context.print_outcome(format!("vhost '{}' not configured in any service", vhost_name)),
      }
    }
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
  VhostListLabel::Domain,
  VhostListLabel::Cert,
];

pub(crate) struct VhostShow {}

#[async_trait]
impl CommandExecutor for VhostShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let vhost_name = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for vhost '{}'", vhost_name));
    let start_instant = context.now();
    let (applications, certificates) = try_join(client.get_application_configuration_map(), client.certificates()).await?;

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
                if vhost.vhost_name == vhost_name {
                  let domain = client.platform().domain_from_vhost_string(&vhost, Some(client.tenant_name())).ok();
                  let cert = domain
                    .as_ref()
                    .map(|domain| find_cert(domain, &certificates))
                    .unwrap_or_default()
                    .map(|(cert_id, _)| cert_id);
                  Some(VhostValue {
                    cert,
                    domain,
                    instances: application.instances,
                    kafka_flag: vhost.kafka,
                    port: port.to_string(),
                    port_mapping: port_mapping.clone(),
                    service_id: application_id.to_string(),
                    tenant: vhost.tenant_name,
                    vhost: vhost.vhost_name,
                    zone: vhost.zone,
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
      context.print_outcome(format!("vhost '{}' not configured", vhost_name));
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
  async fn execute_with_client(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, _client: &DshApiClient, _context: &Context) -> DshCliResult<()> {
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

/// Get services that have a configuration for a given vhost.
///
/// # Parameters
/// * `vhost_name` - Name of the vhost to scan for.
/// * `applications` - Map containing all application configurations.
///
/// # Returns
/// Vector of tuples, where each tuple describes:
/// * `String` - Service_id of the services that has a configuration for the vhost.
/// * `&Application` - Configuration of the service.
/// * `VhostString` - Vhost configuration for the service, contains VhostZone and
///   the kafka flag.
fn applications_using_vhost<'a>(vhost_name: &str, applications: &'a HashMap<String, Application>) -> Vec<(String, &'a Application, VhostString)> {
  applications
    .iter()
    .flat_map(|(application_id, application)| {
      application
        .exposed_ports
        .values()
        .filter_map(|port_mapping| match port_mapping.vhost {
          Some(ref port_mapping_vhost) => match VhostString::from_str(port_mapping_vhost) {
            Ok(vhost_string) => {
              if vhost_string.vhost_name == *vhost_name {
                Some((application_id.clone(), application, vhost_string))
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
    .collect_vec()
}

/// Find certificate by common name.
///
/// # Parameters
/// * `target_common_name` - Common name to look for.
/// * `certs` - Vector of (certificate id, certificate configuration) pairs.
///
/// # Returns
/// Vector of tuples of matching certificates, where each tuple consists of:
/// * `String` - Certificate id.
/// * `ActualCertificate` - Certificate configuration.
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
