use crate::bundle::{create_certificate_authority, get_certificate_authority, CertificateAuthorityId};
use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::global_options::get_expiration_days;
use crate::issues::{IssueDescription, IssueLabel};
use crate::secret_metadata::secrets_with_metadata;
use crate::subject::Requirements;
use crate::subjects::certificate::labels::CertificateLabel;
use crate::subjects::certificate::{has_issues, CERTIFICATE_SUBJECT_TARGET};
use crate::subjects::proxy::options::get_vhost_zone;
use crate::subjects::secret::capabilities::{print_certificate_secret, print_key_secret};
use crate::subjects::secret::SecretWithMetadata;
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::target_platform::get_target_platform;
use crate::target_tenant::get_target_tenant;
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::{DshApiError, DshApiResult};
use dsh_api::platform::VhostZone;
use dsh_api::types::ActualCertificate;
use dsh_api::types::CertificateStatus;
use dsh_api::Dependant;
use futures::future::{join_all, try_join_all};
use futures::join;
use itertools::{multizip, Itertools};
use std::time::Duration;
use tokio::time::sleep;

pub(crate) struct CertificateDelete {}

#[async_trait]
impl CommandExecutor for CertificateDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    match client.get_certificate_status(&certificate_id).await {
      Ok(allocation_status) => {
        context.print_allocation_status(&Ok(allocation_status), CERTIFICATE_SUBJECT_TARGET);
      }
      Err(_) => return err!("secret '{}' does not exist", certificate_id),
    }

    if context.dependencies_warning("certificate", client.certificate_with_dependants::<u8>(&certificate_id).await?.1, &certificate_id)
      && !context.confirmed("do you want to continue?")?
    {
      context.print_outcome(format!("cancelled, certificate '{}' not deleted", certificate_id));
      return Ok(());
    }

    let certificate_configuration = client.get_certificate_configuration(&certificate_id).await?;
    let certificate_secrets = match certificate_configuration.passphrase_secret {
      Some(passphrase_secret) => vec![certificate_configuration.cert_chain_secret, certificate_configuration.key_secret, passphrase_secret],
      None => vec![certificate_configuration.cert_chain_secret, certificate_configuration.key_secret],
    };

    let existing_certificate_secrets: Vec<String> = join_all(certificate_secrets.iter().map(|secret_name| client.get_secret(secret_name)))
      .await
      .iter()
      .map(Result::is_ok)
      .zip(certificate_secrets)
      .filter_map(|(exists, secret_name)| if exists { Some(secret_name) } else { None })
      .collect_vec();

    if context.confirmed(format!("delete certificate '{}'?", certificate_id))? {
      let delete_existing_secrets =
        !existing_certificate_secrets.is_empty() && context.confirmed(format!("delete certificate secrets '{}'?", existing_certificate_secrets.join(", ")))?;
      if context.dry_run() {
        context.print_warning("dry-run mode, certificate not deleted");
      } else {
        client.delete_certificate_configuration(&certificate_id).await?;
        context.print_outcome(format!("certificate '{}' deleted", certificate_id));
        if delete_existing_secrets {
          // Wait until the certificate is gone
          loop {
            context.print_progress_step();
            sleep(Duration::from_millis(1000)).await;
            match client.get_certificate_configuration(&certificate_id).await {
              Ok(_) => {}
              Err(_) => break,
            }
          }
          for secret_name in existing_certificate_secrets {
            client.delete_secret_configuration(&secret_name).await?;
            context.print_outcome(format!("certificate secret '{}' deleted", secret_name));
          }
        }
      }
    } else {
      context.print_outcome(format!("cancelled, certificate '{}' not deleted", certificate_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) static CERTIFICATE_LABELS_LIST: [CertificateLabel; 6] =
  [CertificateLabel::Target, CertificateLabel::Kind, CertificateLabel::CommonName, CertificateLabel::NotAfter, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret];

pub(crate) struct CertificateList {}

pub(crate) type ValidatedVhost = (String, Option<String>, bool, VhostZone);

#[async_trait]
impl CommandExecutor for CertificateList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with their parameters");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let certificate_ids = client.get_certificate_ids().await?;
    let certificate_statuses = join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate(certificate_id))).await;
    context.print_execution_time(start_instant);
    let actual_certificates: Vec<DshApiResult<(ActualCertificate, Option<u64>, Option<ValidatedVhost>)>> = certificate_statuses
      .into_iter()
      .map(|certificate_status| match certificate_status {
        Ok(status) => match status.actual {
          Some(actual_status) => {
            let validated_vhost: Option<ValidatedVhost> = actual_status
              .dns_names
              .first()
              .and_then(|first_dns| client.platform().validate_vhost_domain(first_dns).ok());
            Ok((actual_status, Some(expiration_days), validated_vhost))
          }
          None => Err(DshApiError::from("")),
        },
        Err(error) => Err(error),
      })
      .collect_vec();
    let mut formatter = ListFormatter::new(&CERTIFICATE_LABELS_LIST, context);
    formatter.push_target_ids_and_values(certificate_ids.as_slice(), actual_certificates.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct CertificateListAllocationStatus {}

#[async_trait]
impl CommandExecutor for CertificateListAllocationStatus {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with their allocation status");
    let start_instant = context.now();
    let certificate_ids = client.get_certificate_ids().await?;
    let allocation_statuses = try_join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate_status(certificate_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new_override_target_id_label(&DEFAULT_ALLOCATION_STATUS_LABELS, "certificate id", context);
    formatter.push_target_ids_and_values(certificate_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static CERTIFICATE_CONFIGURATION_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

pub(crate) struct CertificateListConfiguration {}

#[async_trait]
impl CommandExecutor for CertificateListConfiguration {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with their configuration");
    let start_instant = context.now();
    let certificate_ids = client.get_certificate_ids().await?;
    let certificates = try_join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate_configuration(certificate_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&CERTIFICATE_CONFIGURATION_LABELS, context);
    formatter.push_target_ids_and_values(certificate_ids.as_slice(), certificates.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct CertificateListErrors {}

#[async_trait]
impl CommandExecutor for CertificateListErrors {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates that have errors");
    list_certificates(client, matches, context, true).await?
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct CertificateListIds {}

#[async_trait]
impl CommandExecutor for CertificateListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificate ids");
    let start_instant = context.now();
    let certificate_ids = client.get_certificate_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("certificate id", context);
    formatter.push_target_ids(&certificate_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static CERTIFICATE_ISSUE_LABELS_LIST: [IssueLabel; 6] =
  [IssueLabel::Target, IssueLabel::IssueKind, IssueLabel::DependencyName, IssueLabel::DependencySubject, IssueLabel::DependencyValue, IssueLabel::IssueDetails];

async fn list_certificates(client: &DshApiClient, matches: &ArgMatches, context: &Context, only_errors: bool) -> DshCliResult<DshCliResult<()>> {
  let expiration_days = get_expiration_days(matches, context.settings())?;
  let start_instant = context.now();
  let certificate_ids = client.get_certificate_ids().await?;
  let (certificates_statuses, secrets): (Vec<DshApiResult<CertificateStatus>>, DshCliResult<Vec<SecretWithMetadata>>) = join!(
    join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate(certificate_id))),
    secrets_with_metadata(client),
  );
  context.print_execution_time(start_instant);
  let secrets = secrets?;
  let certificate_issues: Vec<(String, Vec<IssueDescription>)> = multizip((certificate_ids, certificates_statuses))
    .collect_vec()
    .into_iter()
    .flat_map(|(certificate_id, certificate_status)| {
      let issues: Option<Vec<IssueDescription>> = has_issues(certificate_status, &secrets, Some(expiration_days), only_errors);
      issues.map(|issues| (certificate_id, issues))
    })
    .collect_vec();
  let mut formatter = ListFormatter::new_override_target_id_label(&CERTIFICATE_ISSUE_LABELS_LIST, "certificate id", context);
  for (certificate_id, certificate_issues) in certificate_issues.into_iter() {
    for certificate_issue in certificate_issues {
      formatter.push_target_id_value_owned(certificate_id.clone(), certificate_issue);
    }
  }
  formatter.print(None)?;
  Ok(Ok(()))
}

pub(crate) struct CertificateListIssues {}

#[async_trait]
impl CommandExecutor for CertificateListIssues {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates that have potential issues");
    list_certificates(client, matches, context, false).await?
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct CertificateListRock {}

#[async_trait]
impl CommandExecutor for CertificateListRock {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant_name = get_target_tenant(matches, context.settings())?;
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let vhost_zone = get_vhost_zone(matches, context)?.unwrap_or(VhostZone::Private);
    let tenant_domain = &platform.tenant_domain(&tenant_name, vhost_zone.clone())?;
    context.print_explanation(format!("list all rock certificates for domain '{}'", tenant_domain));
    let certificate_authority_id = get_certificate_authority(matches, context.settings())?.unwrap_or(CertificateAuthorityId::RockKpnCa);
    let certificate_authority = create_certificate_authority(certificate_authority_id)?;
    certificate_authority.list(tenant_domain, context, expiration_days).await?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

pub(crate) struct CertificateListUsage {}

#[async_trait]
impl CommandExecutor for CertificateListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with the services where they are used");
    let start_instant = context.now();
    let certificates_with_dependants: Vec<(String, CertificateStatus, Vec<Dependant<String>>)> = client.certificates_with_dependants::<String>().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new_override_target_id_label(&DEPENDANT_LABELS_LIST, "certificate id", context);
    for (certificate_id, _, dependants) in certificates_with_dependants.into_iter() {
      if dependants.is_empty() {
        formatter.push_target_id_value_owned(certificate_id.clone(), None::<Dependant<_>>);
      } else {
        for dependant in dependants {
          formatter.push_target_id_value_owned(certificate_id.clone(), Some(dependant));
        }
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) static CERTIFICATE_LABELS_SHOW: [CertificateLabel; 10] = [
  CertificateLabel::Target,
  CertificateLabel::Kind,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::PassphraseSecret,
  CertificateLabel::SerialNumber,
  CertificateLabel::CertChainSecret,
  CertificateLabel::KeySecret,
];

pub(crate) struct CertificateShow {}

#[async_trait]
impl CommandExecutor for CertificateShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for certificate '{}'", certificate_id));
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let (certificate_status, allocation_status) = join!(client.get_certificate(&certificate_id), client.get_certificate_status(&certificate_id));
    context.print_allocation_status(&allocation_status, CERTIFICATE_SUBJECT_TARGET);
    let certificate_status = certificate_status?;
    if let Some(actual_certificate) = &certificate_status.actual {
      let validated_dns = actual_certificate
        .dns_names
        .first()
        .and_then(|first_dns| client.platform().validate_vhost_domain(first_dns).ok());
      context.print_explanation(format!("certificate '{}'", certificate_id));
      UnitFormatter::new(certificate_id.clone(), &CERTIFICATE_LABELS_SHOW, context).print(&(actual_certificate, Some(expiration_days), validated_dns), None)?;

      context.print_explanation(format!("certificate chain secret '{}'", actual_certificate.cert_chain_secret));
      print_certificate_secret(actual_certificate.cert_chain_secret.as_str(), expiration_days, client, context).await?;

      context.print_explanation(format!("key secret '{}'", actual_certificate.key_secret));
      print_key_secret(actual_certificate.key_secret.as_str(), client, context).await?;

      if let Some(certificate) = &certificate_status.configuration {
        if actual_certificate.cert_chain_secret != certificate.cert_chain_secret
          || actual_certificate.key_secret != certificate.key_secret
          || actual_certificate.passphrase_secret != certificate.passphrase_secret
        {
          context.print_warning("actual certificate does not equal certificate configuration");
          UnitFormatter::new("certificate configuration", &CERTIFICATE_CONFIGURATION_LABELS, context).print(certificate, None)?;
        }
      }
    } else {
      context.print_warning(format!("certificate '{}' has no configuration", certificate_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct CertificateShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for CertificateShowAllocationStatus {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the allocation status for certificate '{}'", certificate_id));
    let start_instant = context.now();
    let allocation_status = client.get_certificate_status(&certificate_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(certificate_id, &DEFAULT_ALLOCATION_STATUS_LABELS, context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct CertificateShowUsage {}

#[async_trait]
impl CommandExecutor for CertificateShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all services and apps that use certificate '{}'", certificate_id));
    let start_instant = context.now();
    let (_, dependants) = client.certificate_with_dependants::<String>(&certificate_id).await?;
    context.print_execution_time(start_instant);
    if dependants.is_empty() {
      context.print_outcome("certificate not used")
    } else {
      let mut formatter = ListFormatter::new(&DEPENDANT_LABELS, context);
      formatter.push_values(&dependants);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}
