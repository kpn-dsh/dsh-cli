use crate::arguments::certificate_id_argument;
use crate::capability::{Capability, CommandExecutor, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::error::DshCliError;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{hashmap_to_table, vec_to_table, OutputFormat, Value};
use crate::formatters::{Label, SubjectFormatter};
use crate::global_options::{expiration_option, get_expiration_days};
use crate::issues::{Issue, IssueDescription, IssueLabel, Severity};
use crate::proxy_bundles::DshCertificate;
use crate::secret_metadata::{secret_metadata, SecretMetadata};
use crate::subject::{Requirements, Subject};
use crate::subjects::secret::{secrets_with_metadata, SecretLabel};
use crate::subjects::{secret, DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::{DshApiError, DshApiResult};
use dsh_api::secret::SecretInjection;
use dsh_api::types::CertificateStatus;
use dsh_api::types::{ActualCertificate, AllocationStatus, Certificate};
use dsh_api::Dependant;
use futures::future::{join_all, try_join_all};
use futures::join;
use itertools::{multizip, Itertools};
use lazy_static::lazy_static;
use rcgen::{DistinguishedName, DnType, DnValue, OtherNameValue, SanType};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::time::sleep;

struct CertificateSubject {}

const CERTIFICATE_SUBJECT_TARGET: &str = "certificate";

lazy_static! {
  pub(crate) static ref CERTIFICATE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(CertificateSubject {});
}

#[async_trait]
impl Subject for CertificateSubject {
  fn subject(&self) -> &'static str {
    CERTIFICATE_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH certificates.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list certificates deployed on the DSH.".to_string()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    Some("c")
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(CERTIFICATE_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(CERTIFICATE_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(CERTIFICATE_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &CERTIFICATE_CAPABILITIES
  }
}

static CERTIFICATE_DELETE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(CapabilityBuilder::new(DELETE_COMMAND, None, &CertificateDelete {}, "Delete certificate configuration").add_target_argument(certificate_id_argument().required(true)))
});
static CERTIFICATE_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &CertificateList {}, "List certificates")
      .set_long_about("Lists all available certificates.")
      .add_extra_argument(expiration_option())
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateListAllocationStatus {}, None),
        (FlagType::Configuration, &CertificateListConfiguration {}, None),
        (FlagType::Errors, &CertificateListErrors {}, None),
        (FlagType::Ids, &CertificateListIds {}, None),
        (FlagType::Issues, &CertificateListIssues {}, None),
        (FlagType::Usage, &CertificateListUsage {}, None),
      ]),
  )
});
static CERTIFICATE_SHOW_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &CertificateShow {}, "Show certificate configuration")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateShowAllocationStatus {}, None),
        (FlagType::Usage, &CertificateShowUsage {}, None),
      ])
      .add_target_argument(certificate_id_argument().required(true))
      .add_extra_argument(expiration_option()),
  )
});
static CERTIFICATE_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> =
  LazyLock::new(|| vec![CERTIFICATE_DELETE_CAPABILITY.as_ref(), CERTIFICATE_LIST_CAPABILITY.as_ref(), CERTIFICATE_SHOW_CAPABILITY.as_ref()]);

struct CertificateDelete {}

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

static CERTIFICATE_LABELS_LIST: [CertificateLabel; 4] = [CertificateLabel::Target, CertificateLabel::DistinguishedName, CertificateLabel::NotBefore, CertificateLabel::NotAfter];

struct CertificateList {}

#[async_trait]
impl CommandExecutor for CertificateList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with their parameters");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let certificate_ids = client.get_certificate_ids().await?;
    let certificate_statuses = join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate(certificate_id))).await;
    context.print_execution_time(start_instant);
    let actual_certificates: Vec<DshApiResult<(ActualCertificate, Option<u64>)>> = certificate_statuses
      .into_iter()
      .map(|certificate_status| match certificate_status {
        Ok(status) => match status.actual {
          Some(actual_status) => Ok((actual_status, Some(expiration_days))),
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

struct CertificateListAllocationStatus {}

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

struct CertificateListConfiguration {}

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

struct CertificateListErrors {}

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

struct CertificateListIds {}

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

async fn list_certificates(client: &DshApiClient, matches: &ArgMatches, context: &Context, only_errors: bool) -> Result<Result<(), DshCliError>, DshCliError> {
  let expiration_days = get_expiration_days(matches, context.settings())?;
  let start_instant = context.now();
  let certificate_ids = client.get_certificate_ids().await?;
  let (certificates_statuses, secrets): (Vec<DshApiResult<CertificateStatus>>, DshCliResult<Vec<SecretTuple>>) = join!(
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

struct CertificateListIssues {}

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

struct CertificateListUsage {}

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

pub(crate) static SECRET_CERTIFICATE_LABELS: [SecretLabel; 14] = [
  SecretLabel::CaChain,
  SecretLabel::Description,
  SecretLabel::Format,
  SecretLabel::FormatKind,
  SecretLabel::Issuer,
  SecretLabel::Kind,
  SecretLabel::Label,
  SecretLabel::NotBefore,
  SecretLabel::NotAfter,
  SecretLabel::NumberOfEntries,
  SecretLabel::Private,
  SecretLabel::SecretName,
  SecretLabel::Size,
  SecretLabel::Subject,
];

struct CertificateShow {}

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
      UnitFormatter::new(certificate_id.clone(), &CERTIFICATE_LABELS_SHOW, context).print(&(actual_certificate, Some(expiration_days)), None)?;
      let cert_chain_secret = client.get_secret(&actual_certificate.cert_chain_secret).await?;
      UnitFormatter::new(&actual_certificate.cert_chain_secret, &SECRET_CERTIFICATE_LABELS, context).print(&(secret_metadata(&cert_chain_secret), Some(expiration_days)), None)?;
      let key_secret = client.get_secret(&actual_certificate.key_secret).await?;
      UnitFormatter::new(&actual_certificate.key_secret, &SECRET_CERTIFICATE_LABELS, context).print(&(secret_metadata(&key_secret), None), None)?;
      if let Some(passphrase_secret_name) = &actual_certificate.passphrase_secret {
        let passphrase_secret = client.get_secret(&passphrase_secret_name).await?;
        UnitFormatter::new(passphrase_secret_name, &SECRET_CERTIFICATE_LABELS, context).print(&(secret_metadata(&passphrase_secret), Some(expiration_days)), None)?;
      }
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

struct CertificateShowAllocationStatus {}

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

struct CertificateShowUsage {}

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

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum CertificateLabel {
  CertChainSecret,
  DistinguishedName,
  DnsNames,
  KeySecret,
  NotAfter,
  NotBefore,
  PassphraseSecret,
  SerialNumber,
  Target,
}

impl Label for CertificateLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CertChainSecret => "cert chain secret",
      Self::DistinguishedName => "distinguished name",
      Self::DnsNames => "dns names",
      Self::KeySecret => "key secret",
      Self::NotAfter => "not after",
      Self::NotBefore => "not before",
      Self::PassphraseSecret => "pass phrase secret",
      Self::SerialNumber => "serial number",
      Self::Target => "certificate id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<CertificateLabel> for (&ActualCertificate, Option<u64>) {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    let (actual_certificate, days) = self;
    match label {
      CertificateLabel::CertChainSecret => Value::target(&actual_certificate.cert_chain_secret),
      CertificateLabel::DistinguishedName => Value::distinguished_name(&actual_certificate.distinguished_name),
      CertificateLabel::DnsNames => Value::plain(actual_certificate.dns_names.join("\n")),
      CertificateLabel::KeySecret => Value::target(&actual_certificate.key_secret),
      CertificateLabel::NotAfter => Value::datetime_expired(&actual_certificate.not_after, *days),
      CertificateLabel::NotBefore => Value::datetime_not_before(&actual_certificate.not_before),
      CertificateLabel::PassphraseSecret => match &actual_certificate.passphrase_secret {
        Some(passphrase_secret) => Value::target(passphrase_secret.clone()),
        None => Value::hide(),
      },
      CertificateLabel::SerialNumber => Value::plain(&actual_certificate.serial_number),
      CertificateLabel::Target => Value::target(target_id),
    }
  }
}

impl SubjectFormatter<CertificateLabel> for (ActualCertificate, Option<u64>) {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    let (actual_certificate, days) = self;
    (actual_certificate, *days).value(label, target_id)
  }
}

impl SubjectFormatter<CertificateLabel> for Certificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    match label {
      CertificateLabel::CertChainSecret => Value::plain(&self.cert_chain_secret),
      CertificateLabel::KeySecret => Value::plain(&self.key_secret),
      CertificateLabel::PassphraseSecret => Value::some_or_empty(self.passphrase_secret.clone()),
      CertificateLabel::Target => Value::target(target_id),
      _ => Value::todo(),
    }
  }
}

impl SubjectFormatter<CertificateLabel> for DshCertificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    match label {
      CertificateLabel::CertChainSecret => Value::unreachable(),
      CertificateLabel::DistinguishedName => Value::plain(hashmap_to_table(&hashmap_from_distinguished_name(&self.certificate.params().distinguished_name))),
      CertificateLabel::DnsNames => Value::plain(self.certificate.params().subject_alt_names.iter().map(san_to_string).collect_vec().join("\n")),
      CertificateLabel::KeySecret => Value::unreachable(),
      CertificateLabel::NotAfter => Value::plain(self.certificate.params().not_after),
      CertificateLabel::NotBefore => Value::plain(self.certificate.params().not_before),
      CertificateLabel::PassphraseSecret => Value::unreachable(),
      CertificateLabel::SerialNumber => Value::some_or_hide(self.certificate.params().serial_number.as_ref().map(|serial_number| serial_number.to_string())),
      CertificateLabel::Target => Value::target(target_id),
    }
  }
}

fn san_to_string(san_type: &SanType) -> String {
  match san_type {
    SanType::Rfc822Name(rfc822) => format!("rfc822: {}", rfc822),
    SanType::DnsName(dns_name) => format!("dns: {}", dns_name),
    SanType::URI(uri) => format!("uri: {}", uri),
    SanType::IpAddress(ip_addr) => format!("ip address: {}", ip_addr),
    SanType::OtherName((_, OtherNameValue::Utf8String(utf8_string))) => format!("utf8: {}", utf8_string),
    _ => "".to_string(),
  }
}

fn hashmap_from_distinguished_name(distinguished_name: &DistinguishedName) -> HashMap<String, String> {
  distinguished_name
    .iter()
    .map(|(dn_type, dn_value)| (dn_type_name(dn_type).to_string(), dn_value_string(dn_value)))
    .collect::<HashMap<_, _>>()
}

fn dn_value_string(dn_value: &DnValue) -> String {
  match dn_value {
    DnValue::Ia5String(ia5_string) => ia5_string.to_string(),
    DnValue::PrintableString(printable_string) => printable_string.to_string(),
    DnValue::TeletexString(teletex_string) => teletex_string.to_string(),
    DnValue::Utf8String(utf8_string) => utf8_string.to_string(),
    _ => "".to_string(),
  }
}

fn dn_type_name(dn_type: &DnType) -> &'static str {
  match dn_type {
    DnType::CountryName => "C",
    DnType::LocalityName => "L",
    DnType::StateOrProvinceName => "ST",
    DnType::OrganizationName => "O",
    DnType::OrganizationalUnitName => "OU",
    DnType::CommonName => "CN",
    DnType::CustomDnType(_) => "",
    _ => "",
  }
}

/// Check if a certificate has issues.
///
/// # Parameters
/// * `certificate_status`
/// * `secrets` - List of [`SecretTuple`]s describing all secrets. Each tuple consists of:
///   * `String` - Secret name.
///   * `Option<String>` - Secret id when it is a system secret.
///   * `SecretMetadata` - Secret metadata.
///   * `Option<AllocationStatus>` - Secret allocation status.
///   * `Vec<Dependant<SecretInjection>>` - List of apps, applications and proxies that depend
///     on the secret.
/// * `days` - Number of days until expiration.
/// * `only_errors` - If `true` only issues with severity level `Severity::Error` will be returned.
///
/// # Returns
/// * `Some(Vec<IssueDescription>)` - List of tuples describing the issues found
///   (at least one).
/// * `None` - No issues where found.
fn has_issues(certificate_status: DshApiResult<CertificateStatus>, secrets: &[SecretTuple], days: Option<u64>, only_errors: bool) -> Option<Vec<IssueDescription>> {
  let certificate_status = match certificate_status {
    Ok(certificate_status) => certificate_status,
    Err(_) => return Some(vec![(None, Issue::Unexpected { message: "could not get certificate status".to_string() })]),
  };

  let mut issues: Vec<IssueDescription> = vec![];

  if !certificate_status.status.provisioned {
    issues.push((None, Issue::NotProvisioned))
  }
  for notification in &certificate_status.status.notifications {
    if notification.remove {
      issues.push((None, Issue::RemovalNotification { notification: notification.clone() }));
    } else {
      issues.push((None, Issue::CreationUpdateNotification { notification: notification.clone() }));
    }
  }

  if let Some(actual_certificate) = certificate_status.actual {
    let mut cert_chain_secret_issues = secret_issues(&actual_certificate.cert_chain_secret, days, only_errors, "cert chain", secrets);
    if !cert_chain_secret_issues.iter().any(|(_, issue)| matches!(issue, Issue::Expired { .. })) {
      if let Some(issue) = Issue::datetime_expired(&actual_certificate.not_after, days) {
        if !only_errors || issue.severity() == Severity::Error {
          issues.push((None, issue));
        }
      }
    }

    if !cert_chain_secret_issues.iter().any(|(_, issue)| matches!(issue, Issue::Before { .. })) {
      if let Some(issue) = Issue::datetime_before(&actual_certificate.not_before) {
        if !only_errors || issue.severity() == Severity::Error {
          issues.push((None, issue));
        }
      }
    }
    if secret_is_certificate(&actual_certificate.cert_chain_secret, secrets).is_some_and(|a| !a) {
      cert_chain_secret_issues.push((
        Some(("cert chain", actual_certificate.cert_chain_secret.to_string(), "secret")),
        Issue::Misconfiguration { explanation: "".to_string() },
      ))
    }
    issues.append(&mut cert_chain_secret_issues);

    issues.append(&mut secret_issues(&actual_certificate.key_secret, days, only_errors, "key", secrets));

    if let Some(passphrase_secret) = actual_certificate.passphrase_secret {
      issues.append(&mut secret_issues(&passphrase_secret, days, only_errors, "passphrase", secrets));
    }
  }

  if only_errors {
    issues.retain(|(_, issue)| issue.severity() == Severity::Error);
  }
  if issues.is_empty() {
    None
  } else {
    Some(issues)
  }
}

/// Tuple describing a secret.
///
/// * `String` - Secret name.
/// * `Option<String>` - Secret id when it is a system secret.
/// * `SecretMetadata` - Secret metadata.
/// * `Option<AllocationStatus>` - Secret allocation status.
/// * `Vec<Dependant<SecretInjection>>` - List of apps, applications and proxies that depend
///   on the secret.
type SecretTuple = (String, Option<String>, SecretMetadata, Option<AllocationStatus>, Vec<Dependant<SecretInjection>>);

fn secret_issues<'a>(secret: &str, days: Option<u64>, only_errors: bool, secret_attribute: &'static str, secrets: &[SecretTuple]) -> Vec<IssueDescription<'a>> {
  let mut issues: Vec<IssueDescription<'_>> = vec![];
  match secrets.iter().find(|(secret_id, _, _, _, _)| secret_id == secret) {
    Some(secret_tuple) => {
      if let Some(cert_chain_secret_issues) = secret::has_issues(secret_tuple, days, only_errors) {
        for issue in cert_chain_secret_issues {
          issues.push((Some((secret_attribute, secret.to_string(), "secret")), issue));
        }
      }
    }
    None => {
      issues.push((Some((secret_attribute, secret.to_string(), "secret")), Issue::NotFound {}));
    }
  }
  issues
}

fn secret_is_certificate(secret: &str, secrets: &[SecretTuple]) -> Option<bool> {
  secrets
    .iter()
    .find(|(secret_id, _, _, _, _)| secret_id == secret)
    .map(|(_, _, secret_metadata, _, _)| matches!(secret_metadata, SecretMetadata::Certificate { .. }))
}

static CERTIFICATE_CONFIGURATION_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

pub(crate) static CERTIFICATE_LABELS_SHOW: [CertificateLabel; 9] = [
  CertificateLabel::Target,
  CertificateLabel::CertChainSecret,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
  CertificateLabel::KeySecret,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::PassphraseSecret,
  CertificateLabel::SerialNumber,
];

pub(crate) fn format_distinguished_name(distinguished_name: &str) -> String {
  const ATTRIBUTES: [&str; 8] = ["CN", "O", "OU", "L", "S", "SP", "ST", "C"];
  let map = distinguished_name_to_map(distinguished_name);
  let mut attribute_value_pairs = vec![];
  for attribute in ATTRIBUTES {
    if let Some(value) = map.get(attribute) {
      attribute_value_pairs.push((attribute.to_string(), vec![value.to_string()]))
    }
  }
  for (attribute, value) in map {
    if !ATTRIBUTES.contains(&attribute.as_str()) {
      attribute_value_pairs.push((attribute, vec![value]))
    }
  }
  vec_to_table(&attribute_value_pairs)
}

pub(crate) fn distinguished_name_to_map(distinguished_name: &str) -> HashMap<String, String> {
  distinguished_name
    .split(",")
    .map(|relative_distinguished_name| {
      let attribute_value = relative_distinguished_name.split("=").map(|s| s.trim().to_string()).collect_vec();
      (
        attribute_value.first().cloned().unwrap_or_default().to_string(),
        attribute_value.get(1).cloned().unwrap_or_default().to_string(),
      )
    })
    .collect::<HashMap<_, _>>()
}
