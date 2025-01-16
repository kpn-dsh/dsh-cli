use crate::arguments::target_argument;
use crate::capability::{Capability, CommandExecutor, LIST_COMMAND, LIST_COMMAND_PAIR, SHOW_COMMAND, SHOW_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::subject::Subject;
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, USED_BY_LABELS, USED_BY_LABELS_LIST};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::types::CertificateStatus;
use dsh_api::types::{ActualCertificate, Certificate};
use dsh_api::UsedBy;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use serde::Serialize;
use std::time::Instant;

pub(crate) struct CertificateSubject {}

const CERTIFICATE_SUBJECT_TARGET: &str = "certificate";

lazy_static! {
  pub static ref CERTIFICATE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(CertificateSubject {});
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

  fn requires_dsh_api_client(&self) -> bool {
    true
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
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List certificates")
      .set_long_about("Lists all available certificates.")
      .set_default_command_executor(&CertificateListAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateListAllocationStatus {}, None),
        (FlagType::Configuration, &CertificateListConfiguration {}, None),
        (FlagType::Ids, &CertificateListIds {}, None),
        (FlagType::Usage, &CertificateListUsage {}, None),
      ])
      .set_run_all_executors(true)
  );
  static ref CERTIFICATE_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND_PAIR, "Show certificate configuration")
      .set_default_command_executor(&CertificateShowAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateShowAllocationStatus {}, None),
        (FlagType::Usage, &CertificateShowUsage {}, None)
      ])
      .add_target_argument(target_argument(CERTIFICATE_SUBJECT_TARGET, None))
  );
  static ref CERTIFICATE_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![CERTIFICATE_LIST_CAPABILITY.as_ref(), CERTIFICATE_SHOW_CAPABILITY.as_ref()];
}

struct CertificateListAll {}

#[async_trait]
impl CommandExecutor for CertificateListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all certificates with their parameters");
    let start_instant = Instant::now();
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().list_certificate_ids().await?;
    let certificate_statuses = futures::future::join_all(
      certificate_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_certificate(id.as_str())),
    )
    .await;
    context.print_execution_time(start_instant);
    let certificates_statuses_unwrapped = certificate_statuses
      .iter()
      .map(|certificate_status| certificate_status.as_ref().unwrap().to_owned().actual.unwrap())
      .collect::<Vec<_>>();
    let mut formatter = ListFormatter::new(&CERTIFICATE_LABELS_LIST, None, context);
    formatter.push_target_ids_and_values(certificate_ids.as_slice(), certificates_statuses_unwrapped.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct CertificateListAllocationStatus {}

#[async_trait]
impl CommandExecutor for CertificateListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all certificates with their allocation status");
    let start_instant = Instant::now();
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().list_certificate_ids().await?;
    let allocation_statuses = try_join_all(
      certificate_ids
        .iter()
        .map(|certificate_id| context.dsh_api_client.as_ref().unwrap().get_certificate_allocation_status(certificate_id)),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("certificate id"), context);
    formatter.push_target_ids_and_values(certificate_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct CertificateListConfiguration {}

#[async_trait]
impl CommandExecutor for CertificateListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all certificates with their configuration");
    let start_instant = Instant::now();
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().list_certificate_ids().await?;
    let certificates = try_join_all(
      certificate_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_certificate_configuration(id.as_str())),
    )
    .await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&CERTIFICATE_CONFIGURATION_LABELS, None, context);
    formatter.push_target_ids_and_values(certificate_ids.as_slice(), certificates.as_slice());
    formatter.print()?;
    Ok(())
  }
}

struct CertificateListIds {}

#[async_trait]
impl CommandExecutor for CertificateListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all certificate ids");
    let start_instant = Instant::now();
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().list_certificate_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("certificate id", context);
    formatter.push_target_ids(&certificate_ids);
    formatter.print()?;
    Ok(())
  }
}

struct CertificateListUsage {}

#[async_trait]
impl CommandExecutor for CertificateListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all certificates with the applications where they are used");
    let start_instant = Instant::now();
    let certificates_with_usage: Vec<(String, CertificateStatus, Vec<UsedBy>)> = context.dsh_api_client.as_ref().unwrap().list_certificates_with_usage().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&USED_BY_LABELS_LIST, Some("certificate id"), context);
    for (certificate_id, _certificate, used_bys) in &certificates_with_usage {
      let mut first = true;
      for used_by in used_bys {
        if first {
          formatter.push_target_id_value(certificate_id.clone(), used_by);
        } else {
          formatter.push_target_id_value("".to_string(), used_by);
        }
        first = false;
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no certificate found in apps or applications");
    } else {
      formatter.print()?;
    }
    Ok(())
  }
}

struct CertificateShowAll {}

#[async_trait]
impl CommandExecutor for CertificateShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all parameters for certificate '{}'", certificate_id));
    let start_instant = Instant::now();
    let certificate = context.dsh_api_client.as_ref().unwrap().get_certificate(certificate_id.as_str()).await?;
    if let Some(actual_certificate) = certificate.actual {
      context.print_execution_time(start_instant);
      let formatter = UnitFormatter::new(certificate_id, &CERTIFICATE_LABELS_SHOW, None, &actual_certificate, context);
      formatter.print()?;
    }
    Ok(())
  }
}

struct CertificateShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for CertificateShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the allocation status for certificate '{}'", certificate_id));
    let start_instant = Instant::now();
    let allocation_status = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .get_certificate_allocation_status(certificate_id.as_str())
      .await?;
    context.print_execution_time(start_instant);
    let formatter = UnitFormatter::new(
      certificate_id,
      &DEFAULT_ALLOCATION_STATUS_LABELS,
      Some("certificate id"),
      &allocation_status,
      context,
    );
    formatter.print()?;
    Ok(())
  }
}

struct CertificateShowUsage {}

#[async_trait]
impl CommandExecutor for CertificateShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show all applications and apps that use certificate '{}'", certificate_id));
    let start_instant = Instant::now();
    let (_, usages) = context.dsh_api_client.as_ref().unwrap().get_certificate_with_usage(certificate_id.as_str()).await?;
    context.print_execution_time(start_instant);
    if usages.is_empty() {
      context.print_outcome("certificate not used")
    } else {
      let mut formatter = ListFormatter::new(&USED_BY_LABELS, None, context);
      formatter.push_values(&usages);
      formatter.print()?;
    }

    // let mut builder: TableBuilder<UsedByLabel, UsedBy> = TableBuilder::list(&USED_BY_LABELS_LIST, context);
    // builder.rows(&used_bys);
    // builder.print();
    Ok(())
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub enum CertificateLabel {
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
  fn value(&self, label: &CertificateLabel, target_id: &str) -> String {
    match label {
      CertificateLabel::CertChainSecret => self.cert_chain_secret.to_string(),
      CertificateLabel::DistinguishedName => self.distinguished_name.clone().split(",").collect::<Vec<_>>().join("\n"),
      CertificateLabel::DnsNames => self.dns_names.join("\n"),
      CertificateLabel::KeySecret => self.key_secret.to_string(),
      CertificateLabel::NotAfter => self.not_after.to_string(),
      CertificateLabel::NotBefore => self.not_before.to_string(),
      CertificateLabel::PassphraseSecret => self.passphrase_secret.clone().map(|ref s| s.to_string()).unwrap_or_default(),
      CertificateLabel::SerialNumber => self.serial_number.to_string(),
      CertificateLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<CertificateLabel> {
    Some(CertificateLabel::Target)
  }
}

impl SubjectFormatter<CertificateLabel> for Certificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> String {
    match label {
      CertificateLabel::CertChainSecret => self.cert_chain_secret.clone(),
      CertificateLabel::KeySecret => self.key_secret.clone(),
      CertificateLabel::PassphraseSecret => self.clone().passphrase_secret.unwrap_or_default(),
      CertificateLabel::Target => target_id.to_string(),
      _ => unreachable!(),
    }
  }

  fn target_label(&self) -> Option<CertificateLabel> {
    Some(CertificateLabel::Target)
  }
}

pub static CERTIFICATE_CONFIGURATION_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

pub static CERTIFICATE_LABELS_LIST: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::DistinguishedName, CertificateLabel::NotBefore, CertificateLabel::NotAfter];

pub static CERTIFICATE_LABELS_SHOW: [CertificateLabel; 9] = [
  CertificateLabel::Target,
  CertificateLabel::CertChainSecret,
  CertificateLabel::KeySecret,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::PassphraseSecret,
  CertificateLabel::SerialNumber,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
];
