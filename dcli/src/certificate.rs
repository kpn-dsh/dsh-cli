use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use dsh_api::types::{ActualCertificate, AppCatalogApp, Application, Certificate, CertificateStatus};

use crate::app::apps_with_secret_injections;
use crate::application::applications_with_secret_injections;
use crate::capability::{Capability, CapabilityType, CommandExecutor, DeclarativeCapability};
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::certificate::{CertificateLabel, CERTIFICATE_CONFIGURATION_LABELS, CERTIFICATE_LABELS_LIST, CERTIFICATE_LABELS_SHOW};
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::show_table::ShowTable;
use crate::formatters::usage::{Usage, UsageLabel, USAGE_LABELS_LIST, USAGE_LABELS_SHOW};
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

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

  fn subject_first_upper(&self) -> &'static str {
    "Certificate"
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

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, CERTIFICATE_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, CERTIFICATE_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref CERTIFICATE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::List,
    command_about: "List certificates".to_string(),
    command_long_about: Some("Lists all available certificates.".to_string()),
    command_executors: vec![
      (FlagType::All, &CertificateListAll {}, None),
      (FlagType::AllocationStatus, &CertificateListAllocationStatus {}, None),
      (FlagType::Configuration, &CertificateListConfiguration {}, None),
      (FlagType::Ids, &CertificateListIds {}, None),
      (FlagType::Usage, &CertificateListUsage {}, None),
    ],
    default_command_executor: Some(&CertificateListAll {}),
    run_all_executors: true,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
  pub static ref CERTIFICATE_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(DeclarativeCapability {
    capability_type: CapabilityType::Show,
    command_about: "Show certificate configuration".to_string(),
    command_long_about: None,
    command_executors: vec![
      (FlagType::All, &CertificateShowAll {}, None),
      (FlagType::AllocationStatus, &CertificateShowAllocationStatus {}, None),
      (FlagType::Usage, &CertificateShowUsage {}, None)
    ],
    default_command_executor: Some(&CertificateShowAll {}),
    run_all_executors: false,
    extra_arguments: vec![],
    filter_flags: vec![],
    modifier_flags: vec![],
  });
}

struct CertificateListAll {}

#[async_trait]
impl CommandExecutor for CertificateListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all certificates with their parameters");
    }
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().get_certificate_ids().await?;
    let certificate_statuses = futures::future::join_all(
      certificate_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_certificate(id.as_str())),
    )
    .await;
    let certificates_statuses_unwrapped = certificate_statuses
      .iter()
      .map(|certificate_status| certificate_status.as_ref().unwrap().to_owned().actual.unwrap())
      .collect::<Vec<ActualCertificate>>();
    let zipped = certificate_ids
      .into_iter()
      .zip(certificates_statuses_unwrapped)
      .collect::<Vec<(String, ActualCertificate)>>();
    let mut builder = TableBuilder::list(&CERTIFICATE_LABELS_LIST, context);
    builder.values(&zipped);
    builder.print();
    Ok(false)
  }
}

struct CertificateListAllocationStatus {}

#[async_trait]
impl CommandExecutor for CertificateListAllocationStatus {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all certificates with their allocation status");
    }
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().get_certificate_ids().await?;
    let allocation_statuses = try_join_all(
      certificate_ids
        .iter()
        .map(|certificate_id| context.dsh_api_client.as_ref().unwrap().get_certificate_allocation_status(certificate_id)),
    )
    .await?;
    print_allocation_statuses(certificate_ids, allocation_statuses, context);
    Ok(false)
  }
}

struct CertificateListConfiguration {}

#[async_trait]
impl CommandExecutor for CertificateListConfiguration {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all certificates with their configuration");
    }
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().get_certificate_ids().await?;
    let certificates = try_join_all(
      certificate_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_certificate_configuration(id.as_str())),
    )
    .await?;
    let zipped: Vec<(String, Certificate)> = certificate_ids.into_iter().zip(certificates).collect::<Vec<(String, Certificate)>>();
    let mut builder: TableBuilder<CertificateLabel, Certificate> = TableBuilder::list(&CERTIFICATE_CONFIGURATION_LABELS, context);
    builder.values(&zipped);
    builder.print();
    Ok(false)
  }
}

struct CertificateListIds {}

#[async_trait]
impl CommandExecutor for CertificateListIds {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all certificate ids");
    }
    print_vec(
      "certificate ids".to_string(),
      context.dsh_api_client.as_ref().unwrap().get_certificate_ids().await?,
      context,
    );
    Ok(false)
  }
}

struct CertificateListUsage {}

#[async_trait]
impl CommandExecutor for CertificateListUsage {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all certificates with the applications where they are used");
    }
    let certificate_ids = context.dsh_api_client.as_ref().unwrap().get_certificate_ids().await?;
    let (applications, apps) = try_join!(
      context.dsh_api_client.as_ref().unwrap().get_applications(),
      context.dsh_api_client.as_ref().unwrap().get_app_configurations()
    )?;
    let certificates = futures::future::join_all(
      certificate_ids
        .iter()
        .map(|id| context.dsh_api_client.as_ref().unwrap().get_certificate_configuration(id.as_str())),
    )
    .await;
    let mut rows: Vec<Usage> = vec![];
    for (certificate_id, certificate) in certificate_ids.iter().zip(certificates) {
      let mut certificate_used = false;
      if let Ok(certificate_configuration) = certificate {
        let secrets = match certificate_configuration.passphrase_secret {
          Some(passphrase_secret) => vec![certificate_configuration.cert_chain_secret, certificate_configuration.key_secret, passphrase_secret],
          None => vec![certificate_configuration.cert_chain_secret, certificate_configuration.key_secret],
        };
        for (application_id, instances, secret_injections) in applications_with_secret_injections(&secrets, &applications) {
          let injections = secret_injections
            .iter()
            .map(|(secret, envs)| format!("{}: {}", secret, envs.join(", ")))
            .collect::<Vec<String>>();
          rows.push(Usage::application(certificate_id.clone(), application_id.clone(), instances, injections));
          certificate_used = true;
        }
        for (app_id, instances, secret_injections) in apps_with_secret_injections(&secrets, &apps) {
          let injections = secret_injections
            .iter()
            .map(|(secret, envs)| format!("{}: {}", secret, envs.join(", ")))
            .collect::<Vec<String>>();
          rows.push(Usage::app(certificate_id.clone(), app_id.clone(), instances, injections));
          certificate_used = true;
        }
      }
      if !certificate_used {
        rows.push(Usage::empty(certificate_id.clone()))
      }
    }
    let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::list(&USAGE_LABELS_LIST, context);
    builder.rows(&rows);
    builder.print();
    Ok(false)
  }
}

struct CertificateShowAll {}

#[async_trait]
impl CommandExecutor for CertificateShowAll {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all parameters for certificate '{}'", certificate_id);
    }
    let certificate = context.dsh_api_client.as_ref().unwrap().get_certificate(certificate_id.as_str()).await?;
    if let Some(actual_certificate) = certificate.actual {
      let table = ShowTable::new(&certificate_id, &actual_certificate, &CERTIFICATE_LABELS_SHOW, context);
      table.print();
    }
    Ok(false)
  }
}

struct CertificateShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for CertificateShowAllocationStatus {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show the allocation status for certificate '{}'", certificate_id);
    }
    let allocation_status = context
      .dsh_api_client
      .as_ref()
      .unwrap()
      .get_certificate_allocation_status(certificate_id.as_str())
      .await?;
    print_allocation_status(certificate_id, allocation_status, context);
    Ok(false)
  }
}

struct CertificateShowUsage {}

#[async_trait]
impl CommandExecutor for CertificateShowUsage {
  async fn execute(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    let certificate_id = target.unwrap_or_else(|| unreachable!());
    if context.show_capability_explanation() {
      println!("show all applications and apps that use certificate '{}'", certificate_id);
    }
    let (certificate_status, applications, apps): (CertificateStatus, HashMap<String, Application>, HashMap<String, AppCatalogApp>) = try_join!(
      context.dsh_api_client.as_ref().unwrap().get_certificate(certificate_id.as_str()),
      context.dsh_api_client.as_ref().unwrap().get_applications(),
      context.dsh_api_client.as_ref().unwrap().get_app_configurations()
    )?;
    let mut rows: Vec<Usage> = vec![];
    if let Some(configuration) = certificate_status.configuration {
      let secrets = match configuration.passphrase_secret {
        Some(passphrase_secret) => vec![configuration.cert_chain_secret, configuration.key_secret, passphrase_secret],
        None => vec![configuration.cert_chain_secret, configuration.key_secret],
      };
      for (application_id, instances, secret_injections) in applications_with_secret_injections(&secrets, &applications) {
        for (_, injections) in secret_injections {
          rows.push(Usage::application(certificate_id.clone(), application_id.to_string(), instances, injections));
        }
      }
      for (app_id, instances, secret_injections) in apps_with_secret_injections(&secrets, &apps) {
        for (_, injections) in secret_injections {
          rows.push(Usage::app(certificate_id.clone(), app_id.clone(), instances, injections));
        }
      }
    }
    let mut builder: TableBuilder<UsageLabel, Usage> = TableBuilder::show(&USAGE_LABELS_SHOW, context);
    builder.rows(&rows);
    builder.print();
    Ok(false)
  }
}
