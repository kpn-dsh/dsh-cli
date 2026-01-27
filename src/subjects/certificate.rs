use crate::arguments::certificate_id_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{vec_to_table, OutputFormat, Value};
use crate::formatters::{Label, SubjectFormatter};
use crate::secret_entry::secret_entries_from;
use crate::subject::{Requirements, Subject};
use crate::subjects::secret::SECRET_LABELS_LIST;
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::types::CertificateStatus;
use dsh_api::types::{ActualCertificate, Certificate};
use dsh_api::{Dependant, DshApiError, DshApiResult};
use futures::future::{join_all, try_join_all};
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use serde::Serialize;

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
      LIST_COMMAND => Some(CERTIFICATE_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(CERTIFICATE_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &CERTIFICATE_CAPABILITIES
  }
}

lazy_static! {
  static ref CERTIFICATE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &CertificateList {}, "List certificates")
      .set_long_about("Lists all available certificates.")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateListAllocationStatus {}, None),
        (FlagType::Configuration, &CertificateListConfiguration {}, None),
        (FlagType::Ids, &CertificateListIds {}, None),
        (FlagType::Usage, &CertificateListUsage {}, None),
      ])
  );
  static ref CERTIFICATE_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &CertificateShow {}, "Show certificate configuration")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateShowAllocationStatus {}, None),
        (FlagType::Usage, &CertificateShowUsage {}, None)
      ])
      .add_target_argument(certificate_id_argument().required(true))
  );
  static ref CERTIFICATE_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![CERTIFICATE_LIST_CAPABILITY.as_ref(), CERTIFICATE_SHOW_CAPABILITY.as_ref()];
}

struct CertificateList {}

#[async_trait]
impl CommandExecutor for CertificateList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with their parameters");
    let start_instant = context.now();
    let certificate_ids = client.get_certificate_ids().await?;
    let certificate_statuses = join_all(certificate_ids.iter().map(|certificate_id| client.get_certificate(certificate_id))).await;
    context.print_execution_time(start_instant);
    let actual_certificates: Vec<DshApiResult<ActualCertificate>> = certificate_statuses
      .into_iter()
      .map(|certificate_status| match certificate_status {
        Ok(status) => match status.actual {
          Some(actual_status) => Ok(actual_status),
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
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
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

struct CertificateListUsage {}

#[async_trait]
impl CommandExecutor for CertificateListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all certificates with the services where they are used");
    let start_instant = context.now();
    let certificates_with_dependants: Vec<(String, CertificateStatus, Vec<Dependant<String>>)> = client.certificates_with_dependants::<String>().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, context);
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

struct CertificateShow {}

#[async_trait]
impl CommandExecutor for CertificateShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for certificate '{}'", certificate_id));
    let start_instant = context.now();
    let (certificate_status, allocation_status) = join!(client.get_certificate(&certificate_id), client.get_certificate_status(&certificate_id));
    context.print_allocation_status(&allocation_status, CERTIFICATE_SUBJECT_TARGET);
    let certificate_status = certificate_status?;
    if let Some(actual_certificate) = &certificate_status.actual {
      UnitFormatter::new(certificate_id.clone(), &CERTIFICATE_LABELS_SHOW, context).print(actual_certificate, None)?;
      let certificate_secret_ids = if let Some(passphrase_secret) = &actual_certificate.passphrase_secret {
        vec![actual_certificate.cert_chain_secret.clone(), actual_certificate.key_secret.clone(), passphrase_secret.clone()]
      } else {
        vec![actual_certificate.cert_chain_secret.clone(), actual_certificate.key_secret.clone()]
      };
      let certificate_secrets = try_join_all(certificate_secret_ids.iter().map(|secret_id| client.get_secret(secret_id))).await?;
      context.print_execution_time(start_instant);
      let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, context);
      for (secret_id, secret) in certificate_secret_ids.iter().zip(certificate_secrets) {
        for secret_entry in secret_entries_from(&secret, false) {
          formatter.push_target_id_value_owned(secret_id.clone(), secret_entry);
        }
      }
      formatter.print(None)?;
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
      context.print_execution_time(start_instant);
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

  fn as_str_for_list(&self) -> &str {
    match self {
      Self::CertChainSecret => "cert secret",
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

impl SubjectFormatter<CertificateLabel> for ActualCertificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    match label {
      CertificateLabel::CertChainSecret => Value::plain(&self.cert_chain_secret),
      CertificateLabel::DistinguishedName => Value::plain(format_distinguished_name(&self.distinguished_name)),
      CertificateLabel::DnsNames => Value::plain(self.dns_names.join("\n")),
      CertificateLabel::KeySecret => Value::plain(&self.key_secret),
      CertificateLabel::NotAfter => Value::date_time_expired(&self.not_after),
      CertificateLabel::NotBefore => Value::date_time_not_yet_passed(&self.not_before),
      CertificateLabel::PassphraseSecret => Value::option(self.passphrase_secret.clone()),
      CertificateLabel::SerialNumber => Value::plain(&self.serial_number),
      CertificateLabel::Target => Value::target(target_id),
    }
  }
}

impl SubjectFormatter<CertificateLabel> for Certificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    match label {
      CertificateLabel::CertChainSecret => Value::plain(&self.cert_chain_secret),
      CertificateLabel::KeySecret => Value::plain(&self.key_secret),
      CertificateLabel::PassphraseSecret => Value::option(self.passphrase_secret.clone()),
      CertificateLabel::Target => Value::target(target_id),
      _ => unreachable!(),
    }
  }
}

static CERTIFICATE_CONFIGURATION_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

static CERTIFICATE_LABELS_LIST: [CertificateLabel; 4] = [CertificateLabel::Target, CertificateLabel::DistinguishedName, CertificateLabel::NotBefore, CertificateLabel::NotAfter];

pub(crate) static CERTIFICATE_LABELS_SHOW: [CertificateLabel; 9] = [
  CertificateLabel::Target,
  CertificateLabel::CertChainSecret,
  CertificateLabel::KeySecret,
  CertificateLabel::PassphraseSecret,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::SerialNumber,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
];

fn format_distinguished_name(distinguished_name: &str) -> String {
  vec_to_table(
    &distinguished_name
      .split(",")
      .map(|relative_distinguished_name| {
        let attribute_value = relative_distinguished_name.split("=").map(|s| s.trim().to_string()).collect_vec();
        (
          attribute_value.first().cloned().unwrap_or_default().to_string(),
          vec![attribute_value.get(1).cloned().unwrap_or_default().to_string()],
        )
      })
      .collect_vec(),
  )
}
