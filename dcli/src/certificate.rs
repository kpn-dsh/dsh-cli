use std::collections::HashMap;

use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use futures::future::try_join_all;
use futures::try_join;
use lazy_static::lazy_static;

use crate::arguments::target_argument;
use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::context::DcliContext;
use crate::flags::FlagType;
use crate::formatters::allocation_status::{print_allocation_status, print_allocation_statuses};
use crate::formatters::certificate::{CertificateLabel, CERTIFICATE_CONFIGURATION_LABELS, CERTIFICATE_LABELS_LIST, CERTIFICATE_LABELS_SHOW};
use crate::formatters::formatter::{print_vec, TableBuilder};
use crate::formatters::show_table::ShowTable;
use crate::formatters::usage::{Usage, UsageLabel, USAGE_LABELS_LIST};
use crate::formatters::used_by::{UsedByLabel, USED_BY_LABELS_LIST};
use crate::subject::Subject;
use crate::DcliResult;
use dsh_api::types::Certificate;
use dsh_api::UsedBy;

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

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, CERTIFICATE_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Show, CERTIFICATE_SHOW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref CERTIFICATE_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List certificates")
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
  pub static ref CERTIFICATE_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Show, "Show certificate configuration")
      .set_default_command_executor(&CertificateShowAll {})
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &CertificateShowAllocationStatus {}, None),
        (FlagType::Usage, &CertificateShowUsage {}, None)
      ])
      .add_target_argument(target_argument(CERTIFICATE_SUBJECT_TARGET, None))
  );
}

struct CertificateListAll {}

#[async_trait]
impl CommandExecutor for CertificateListAll {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    context.print_capability_explanation("list all certificates with their parameters");
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
      .collect::<Vec<_>>();
    let zipped = certificate_ids.into_iter().zip(certificates_statuses_unwrapped).collect::<Vec<_>>();
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
    context.print_capability_explanation("list all certificates with their allocation status");
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
    context.print_capability_explanation("list all certificates with their configuration");
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
    context.print_capability_explanation("list all certificate ids");
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
    context.print_capability_explanation("list all certificates with the applications where they are used");
    let (certificate_ids, applications, apps) = try_join!(
      context.dsh_api_client.as_ref().unwrap().get_certificate_ids(),
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
        for (application_id, application, secret_injections) in DshApiClient::applications_with_secrets_injections(&secrets, &applications) {
          let injections = secret_injections
            .iter()
            .map(|(secret, envs)| format!("{}: {}", secret, envs.iter().map(|env| env.to_string()).collect::<Vec<_>>().join(", ")))
            .collect::<Vec<_>>();
          rows.push(Usage::application(
            certificate_id.clone(),
            application_id.clone(),
            application.instances,
            injections,
          ));
          certificate_used = true;
        }
        for (app_id, _, _, application, secret_injections) in DshApiClient::apps_with_secrets_injections(&secrets, &apps) {
          let injections = secret_injections
            .iter()
            .map(|(secret, envs)| format!("{}: {}", secret, envs.iter().map(|env| env.to_string()).collect::<Vec<_>>().join(", ")))
            .collect::<Vec<_>>();
          rows.push(Usage::app(certificate_id.clone(), app_id.clone(), application.instances, injections));
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
    context.print_capability_explanation(format!("show all parameters for certificate '{}'", certificate_id));
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
    context.print_capability_explanation(format!("show the allocation status for certificate '{}'", certificate_id));
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
    context.print_capability_explanation(format!("show all applications and apps that use certificate '{}'", certificate_id));
    let (_, used_bys) = context.dsh_api_client.as_ref().unwrap().get_certificate_with_usage(certificate_id.as_str()).await?;
    let mut builder: TableBuilder<UsedByLabel, UsedBy> = TableBuilder::list(&USED_BY_LABELS_LIST, context);
    builder.rows(&used_bys);
    builder.print();
    Ok(false)
  }
}
