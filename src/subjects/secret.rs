use crate::arguments::secret_id_argument;
use crate::capability::{
  Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS, UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, OutputFormat, SubjectFormatter};
use crate::modifier_flags::ModifierFlagType;
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::{error, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::secret::{is_system_id, secret_id_to_secret_name, SecretInjection};
use dsh_api::types::Secret;
use dsh_api::{Dependant, DshApiError};
use futures::future::{join_all, try_join_all};
use futures::try_join;
use itertools::Itertools;
use lazy_static::lazy_static;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use rsa::RsaPublicKey;
use serde::Serialize;
use serde_json::Value;
use x509_parser::asn1_rs::FromDer;
use x509_parser::pem::{parse_x509_pem, Pem};
use x509_parser::prelude::X509Certificate;
use x509_parser::x509::X509Name;

pub struct SecretSubject {}

const SECRET_SUBJECT_TARGET: &str = "secret";

lazy_static! {
  pub(crate) static ref SECRET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SecretSubject {});
}

#[async_trait]
impl Subject for SecretSubject {
  fn subject(&self) -> &'static str {
    SECRET_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH secrets.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list secrets used by the services and apps on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(SECRET_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(SECRET_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(SECRET_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(SECRET_SHOW_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(SECRET_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &SECRET_CAPABILITIES
  }
}

lazy_static! {
  static ref SECRET_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &SecretCreate {}, "Create new secret")
      .set_long_about("Create a new secret.")
      .add_target_argument(secret_id_argument().required(true))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  );
  static ref SECRET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &SecretDelete {}, "Delete secret")
      .set_long_about("Delete a secret.")
      .add_target_argument(secret_id_argument().required(true))
  );
  static ref SECRET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &SecretList {}, "List secrets")
      .set_long_about("Lists all secrets used by the services and apps on the DSH.")
      .add_command_executors(vec![
        (FlagType::AllocationStatus, &SecretListAllocationStatus {}, None),
        (FlagType::Certificates, &SecretListCertificates {}, None),
        (FlagType::Ids, &SecretListIds {}, None),
        (FlagType::Keys, &SecretListKeys {}, None),
        (FlagType::System, &SecretListSystem {}, None),
        (FlagType::Usage, &SecretListUsage {}, None)
      ])
  );
  static ref SECRET_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &SecretShow {}, "Show secret details")
      .add_command_executor(
        FlagType::AllocationStatus,
        &SecretShowAllocationStatus {},
        Some("Show secret allocation status".to_string())
      )
      .add_command_executor(FlagType::Usage, &SecretShowUsage {}, Some("Show where the secret is used".to_string()))
      .add_command_executor(FlagType::Value, &SecretShowValue {}, Some("Show the secret value".to_string()))
      .add_target_argument(secret_id_argument().required(true))
  );
  static ref SECRET_UPDATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &SecretUpdate {}, "Update secret")
      .set_long_about("Update a secret.")
      .add_target_argument(secret_id_argument().required(true))
      .add_modifier_flag(ModifierFlagType::MultiLine, None),
  );
  static ref SECRET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![SECRET_CREATE_CAPABILITY.as_ref(), SECRET_DELETE_CAPABILITY.as_ref(), SECRET_LIST_CAPABILITY.as_ref(), SECRET_SHOW_CAPABILITY.as_ref(), SECRET_UPDATE_CAPABILITY.as_ref()];
}

struct SecretCreate {}

#[async_trait]
impl CommandExecutor for SecretCreate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if client.get_secret(&secret_id).await.is_ok() {
      return Err(error!("secret '{}' already exists", secret_id));
    }
    if context.stdin_is_terminal() {
      if matches.get_flag(ModifierFlagType::MultiLine.id()) {
        context.print_explanation(format!("create new multi-line secret '{}'", secret_id));
        let secret = context.read_multi_line("enter multi-line secret (terminate input with ctrl-d after last line)")?;
        let secret = Secret { name: secret_id.clone(), value: secret };
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not created");
        } else {
          client.post_secret(&secret).await?;
          context.print_outcome(format!("secret '{}' created", secret_id));
        }
      } else {
        context.print_explanation(format!("create new single line secret '{}'", secret_id));
        let secret = context.read_single_line_password("enter secret")?;
        let secret = Secret { name: secret_id.clone(), value: secret };
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not created");
        } else {
          client.post_secret(&secret).await?;
          context.print_outcome(format!("secret '{}' created", secret_id));
        }
      }
    } else {
      let secret = context.read_multi_line("")?;
      let secret = Secret { name: secret_id.clone(), value: secret };
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not created");
      } else {
        client.post_secret(&secret).await?;
        context.print_outcome(format!("secret '{}' created", secret_id));
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretDelete {}

#[async_trait]
impl CommandExecutor for SecretDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if client.get_secret_configuration(&secret_id).await.is_err() {
      return Err(error!("secret '{}' does not exist", secret_id));
    }
    if context.confirmed(format!("delete secret '{}'?", secret_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not deleted");
      } else {
        client.delete_secret_configuration(&secret_id).await?;
        context.print_outcome(format!("secret '{}' deleted", secret_id));
      }
    } else {
      context.print_outcome(format!("cancelled, secret '{}' not deleted", secret_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretList {}

#[async_trait]
impl CommandExecutor for SecretList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets");
    let start_instant = context.now();
    let secret_names = client
      .get_secret_ids()
      .await?
      .iter()
      .map(|secret_id| if is_system_id(secret_id) { (secret_id_to_secret_name(secret_id).into_owned(), true) } else { (secret_id.to_string(), false) })
      .collect_vec();
    let secret_value_results = join_all(secret_names.iter().map(|(secret_name, _)| client.get_secret(secret_name))).await;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, Some("secret id"), context);
    for ((secret_name, system_id), secret_value_result) in secret_names.into_iter().zip(secret_value_results) {
      match secret_value_result {
        Ok(secret_value) => {
          if secret_value.trim().is_empty() {
            formatter.push_target_id_value_owned(secret_name.clone(), SecretEntry::new("empty".to_string(), "".to_string(), None, None));
          } else if system_id {
            formatter.push_target_id_value_owned(secret_name.clone(), SecretEntry::new("system".to_string(), "plain".to_string(), None, None));
          } else if let Some(certificate_entries) = get_certificates_from_pem(&secret_value) {
            for certificate_entry in certificate_entries {
              formatter.push_target_id_value_owned(
                secret_name.clone(),
                SecretEntry::new(
                  "cert".to_string(),
                  "pem".to_string(),
                  Some(certificate_entry.subject),
                  Some(certificate_entry.not_after),
                ),
              );
            }
          } else if let Some(keys) = get_keys_from_pem(&secret_value) {
            for (rsa_private_key, label, kind) in keys {
              formatter.push_target_id_value_owned(
                secret_name.clone(),
                SecretEntry::new(
                  "pki".to_string(),
                  format!("pem.{}", kind),
                  Some(format!("{}, {} bit", label, rsa_private_key.size())),
                  None,
                ),
              );
            }
          } else if let Some(description) = get_json(&secret_value) {
            formatter.push_target_id_value_owned(secret_name, SecretEntry::new("settings".to_string(), "json".to_string(), Some(description), None));
          } else if let Some(description) = get_toml(&secret_value) {
            formatter.push_target_id_value_owned(secret_name, SecretEntry::new("settings".to_string(), "toml".to_string(), Some(description), None));
          } else if let Some(description) = get_yaml(&secret_value) {
            formatter.push_target_id_value_owned(secret_name, SecretEntry::new("settings".to_string(), "yaml".to_string(), Some(description), None));
          } else if let Some(description) = get_multiline(&secret_value) {
            formatter.push_target_id_value_owned(
              secret_name,
              SecretEntry::new("settings".to_string(), "multi-line".to_string(), Some(description), None),
            );
          } else {
            formatter.push_target_id_value_owned(secret_name, SecretEntry::new("regular".to_string(), "plain".to_string(), None, None));
          }
        }
        Err(error) => {
          let (kind, message) = match error {
            DshApiError::NotFound(_) => ("not-found", Some("secret not found, possibly pending".to_string())),
            DshApiError::BadRequest(_) => ("bad-request", None),
            DshApiError::Configuration(_) => ("configuration", None),
            DshApiError::NotAuthorized(_) => ("not-authorized", None),
            DshApiError::Parameter(_) => ("parameter", None),
            DshApiError::Unexpected(_, _) => ("unexpected", Some("possibly a network failure".to_string())),
            DshApiError::Unprocessable(_) => ("unprocessable", None),
          };
          formatter.push_target_id_value_owned(secret_name, SecretEntry::new("error".to_string(), kind.to_string(), message, None));
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

struct SecretListAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretListAllocationStatus {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets with their allocation status");
    let start_instant = context.now();
    let non_system_secret_ids = client.get_secret_ids().await?.into_iter().filter(|id| !is_system_id(id)).collect_vec();
    let allocation_statuses = try_join_all(non_system_secret_ids.iter().map(|secret_id| client.get_secret_status(secret_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("secret id"), context);
    formatter.push_target_ids_and_values(non_system_secret_ids.as_slice(), allocation_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretListCertificates {}

#[async_trait]
impl CommandExecutor for SecretListCertificates {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that contain certificates");
    let start_instant = context.now();
    let non_system_secret_ids = client.get_secret_ids().await?.into_iter().filter(|id| !is_system_id(id)).collect_vec();
    let secret_values = try_join_all(non_system_secret_ids.iter().map(|secret_id| client.get_secret(secret_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&X509_CERTIFICATE_LABELS_LIST, Some("secret id"), context);
    for (secret_id, secret_value) in non_system_secret_ids.into_iter().zip(secret_values) {
      if let Some(certificate_entries) = get_certificates_from_pem(&secret_value) {
        for certificate_entry in certificate_entries {
          formatter.push_target_id_value_owned(secret_id.clone(), certificate_entry);
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

struct SecretListIds {}

#[async_trait]
impl CommandExecutor for SecretListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secret ids");
    let start_instant = context.now();
    let non_system_secrets = client.get_secret_ids().await?.into_iter().filter(|id| !is_system_id(id)).collect_vec();
    context.print_execution_time(start_instant);
    let header = format!("secret ids ({})", non_system_secrets.len());
    let mut formatter = IdsFormatter::new(&header, context);
    formatter.push_target_ids(non_system_secrets.as_slice());
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretListKeys {}

#[async_trait]
impl CommandExecutor for SecretListKeys {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that contain keys");
    let start_instant = context.now();
    let non_system_secret_ids = client.get_secret_ids().await?.into_iter().filter(|id| !is_system_id(id)).collect_vec();
    let secret_values = try_join_all(non_system_secret_ids.iter().map(|secret_id| client.get_secret(secret_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&KEY_LABELS_LIST, Some("secret id"), context);
    for (secret_id, secret_value) in non_system_secret_ids.into_iter().zip(secret_values) {
      if let Some(private_keys) = get_keys_from_pem(&secret_value) {
        for (rsa_private_key, label, kind) in private_keys {
          formatter.push_target_id_value_owned(secret_id.clone(), KeyEntry::from((kind.to_string(), label.to_string(), rsa_private_key)));
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

struct SecretListSystem {}

#[async_trait]
impl CommandExecutor for SecretListSystem {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all system secret ids");
    let start_instant = context.now();
    let system_secret_ids = client.get_secret_ids().await?.into_iter().filter(|id| is_system_id(id)).collect_vec();
    let secret_names: Vec<String> = system_secret_ids
      .iter()
      .map(|secret_id| secret_id_to_secret_name(secret_id).into_owned())
      .collect_vec();
    let allocation_statuses = try_join_all(system_secret_ids.iter().map(|secret_id| client.get_secret_status(secret_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, Some("system secret id"), context);
    formatter.push_target_ids_and_values(secret_names.as_slice(), allocation_statuses.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that are used in apps or services");
    let start_instant = context.now();
    let secrets_with_dependants: Vec<(String, Vec<Dependant<SecretInjection>>)> = client.secrets_with_dependants().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, Some("secret id"), context);
    for (secret_id, dependants) in &secrets_with_dependants {
      for dependant in dependants {
        formatter.push_target_id_value(secret_id.clone(), dependant);
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no secrets found in apps or services");
    } else {
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretShowAllocationStatus {}

#[async_trait]
impl CommandExecutor for SecretShowAllocationStatus {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show allocation status for secret '{}'", secret_id));
    let start_instant = context.now();
    let allocation_status = client.get_secret_status(&secret_id).await?;
    context.print_execution_time(start_instant);
    UnitFormatter::new(secret_id, &DEFAULT_ALLOCATION_STATUS_LABELS, Some("secret id"), context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  // TODO Add other usage, e.g. in proxies
  // TODO Improve rendering app/service
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the apps and services that use secret '{}'", secret_id));
    let start_instant = context.now();
    let usages = client.applications_dependant_on_secret(&secret_id).await?;
    context.print_execution_time(start_instant);
    if usages.is_empty() {
      context.print_outcome("secret not used")
    } else {
      let mut formatter = ListFormatter::new(&DEPENDANT_LABELS, Some("secret id"), context);
      formatter.push_values(&usages);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretShow {}

#[async_trait]
impl CommandExecutor for SecretShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let (secret, allocation_status) = try_join!(client.get_secret(&secret_id), client.get_secret_status(&secret_id))?;
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, "secret");
    match parse_x509_pem(secret.as_bytes()) {
      Ok((_, pem)) => match pem.parse_x509() {
        Ok(x509_certificate) => {
          context.print_explanation(format!("secret '{}' is a valid pem encoded certificate", secret_id));
          UnitFormatter::new(secret_id.clone(), &X509_CERTIFICATE_LABELS_SHOW, None, context).print_non_serializable(&x509_certificate, None)?;
        }
        Err(_) => {
          context.print_explanation(format!("secret '{}' is pem encoded", secret_id));
        }
      },
      Err(_) => match X509Certificate::from_der(secret.as_bytes()) {
        Ok((_, x509_certificate)) => {
          context.print_explanation(format!("secret '{}' is a valid der encoded certificate", secret_id));
          UnitFormatter::new(secret_id.clone(), &X509_CERTIFICATE_LABELS_SHOW, None, context).print_non_serializable(&x509_certificate, None)?;
        }
        Err(_) => {
          context.print_explanation(format!("secret '{}' is a plain text secret, use --value to see its value", secret_id));
        }
      },
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretShowValue {}

#[async_trait]
impl CommandExecutor for SecretShowValue {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let secret = client.get_secret(&secret_id).await?;
    context.print_execution_time(start_instant);
    context.print_explanation(format!("show the value of secret '{}'", secret_id));
    context.print(secret);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretUpdate {}

#[async_trait]
impl CommandExecutor for SecretUpdate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    if client.get_secret(&secret_id).await.is_err() {
      return Err(error!("secret '{}' does not exist", secret_id));
    }
    if context.stdin_is_terminal() {
      if matches.get_flag(ModifierFlagType::MultiLine.id()) {
        context.print_explanation(format!("update multi-line secret '{}'", secret_id));
        let secret = context.read_multi_line("enter multi-line secret (terminate input with ctrl-d after last line)")?;
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not updated");
        } else {
          client.put_secret(&secret_id, secret).await?;
          context.print_outcome(format!("secret '{}' updated", secret_id));
        }
      } else {
        context.print_explanation(format!("update single line secret '{}'", secret_id));
        let secret = context.read_single_line_password("enter secret")?;
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not updated");
        } else {
          client.put_secret(&secret_id, secret).await?;
          context.print_outcome(format!("secret '{}' updated", secret_id));
        }
      }
    } else {
      let secret = context.read_multi_line("")?;
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not updated");
      } else {
        client.put_secret(&secret_id, secret).await?;
        context.print_outcome(format!("secret '{}' updated", secret_id));
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum X509CertificateLabel {
  Label,
  Issuer,
  IssuerUid,
  NotAfter,
  NotBefore,
  SecretId,
  Serial,
  Signature,
  SignatureAlgorithm,
  SignatureValue,
  Subject,
  SubjectPki,
  SubjectUid,
  TbsCertificate,
  Version,
}

impl Label for X509CertificateLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Label => "label",
      Self::Issuer => "issuer",
      Self::IssuerUid => "issuer uid",
      Self::NotAfter => "not after",
      Self::NotBefore => "not before",
      Self::SecretId => "secret id",
      Self::Serial => "serial",
      Self::Signature => "signature",
      Self::SignatureAlgorithm => "signature algorithm",
      Self::SignatureValue => "signature value",
      Self::Subject => "subject",
      Self::SubjectPki => "subject pki",
      Self::SubjectUid => "subject uid",
      Self::TbsCertificate => "tbs certificate",
      Self::Version => "version",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretId)
  }
}

impl SubjectFormatter<X509CertificateLabel> for X509Certificate<'_> {
  fn value(&self, label: &X509CertificateLabel, target_id: &str) -> String {
    match label {
      X509CertificateLabel::Label => "".to_string(),
      X509CertificateLabel::Issuer => self.issuer().to_string(),
      X509CertificateLabel::IssuerUid => self
        .issuer_uid
        .clone()
        .map(|uid| String::from_utf8(uid.0.data.to_vec()).unwrap_or_default().to_string())
        .unwrap_or_default(),
      X509CertificateLabel::NotAfter => self.validity.not_after.to_string(),
      X509CertificateLabel::NotBefore => self.validity.not_before.to_string(),
      X509CertificateLabel::SecretId => target_id.to_string(),
      X509CertificateLabel::Serial => self.serial.to_string(),
      X509CertificateLabel::Signature => self.signature.algorithm.to_string(),
      X509CertificateLabel::SignatureAlgorithm => self.signature_algorithm.algorithm.to_string(),
      X509CertificateLabel::SignatureValue => self.signature.algorithm.to_string(),
      X509CertificateLabel::Subject => self.subject().to_string(),
      X509CertificateLabel::SubjectPki => self.subject_pki.algorithm.oid().to_string(),
      X509CertificateLabel::SubjectUid => self
        .subject_uid
        .clone()
        .map(|uid| String::from_utf8(uid.0.data.to_vec()).unwrap_or_default().to_string())
        .unwrap_or_default(),
      X509CertificateLabel::TbsCertificate => "TODO".to_string(),
      X509CertificateLabel::Version => self.version.to_string(),
    }
  }
}

#[derive(Clone, Serialize)]
struct CertificateEntry {
  subject: String,
  not_after: String,
  not_before: String,
  issuer: String,
  label: String,
}

fn x509_name_to_string(x509_name: &X509Name) -> String {
  [
    x509_name
      .iter_common_name()
      .next()
      .and_then(|common_name| common_name.as_str().ok().map(|name| format!("CN={}", name))),
    x509_name
      .iter_organization()
      .next()
      .and_then(|organization| organization.as_str().ok().map(|org| format!("O={}", org))),
  ]
  .iter()
  .flatten()
  .join(", ")
}

impl From<(X509Certificate<'_>, String)> for CertificateEntry {
  fn from((certificate, label): (X509Certificate, String)) -> Self {
    Self {
      subject: x509_name_to_string(&certificate.subject),
      not_after: certificate.validity.not_after.to_string(),
      not_before: certificate.validity.not_before.to_string(),
      issuer: x509_name_to_string(&certificate.issuer),
      label,
    }
  }
}

impl SubjectFormatter<X509CertificateLabel> for CertificateEntry {
  fn value(&self, label: &X509CertificateLabel, target_id: &str) -> String {
    match label {
      X509CertificateLabel::Label => self.label.to_string(),
      X509CertificateLabel::Issuer => self.issuer.to_string(),
      X509CertificateLabel::NotAfter => self.not_after.to_string(),
      X509CertificateLabel::NotBefore => self.not_before.to_string(),
      X509CertificateLabel::SecretId => target_id.to_string(),
      X509CertificateLabel::Subject => self.subject.to_string(),
      _ => "".to_string(),
    }
  }
}

#[derive(Clone, Serialize)]
struct KeyEntry {
  kind: String,
  private: bool,
  size: usize,
  validated: Option<bool>,
  label: String,
}

impl From<(String, String, RsaPrivateKey)> for KeyEntry {
  fn from((kind, label, rsa_private_key): (String, String, RsaPrivateKey)) -> Self {
    Self { kind, private: true, size: rsa_private_key.size(), validated: Some(rsa_private_key.validate().is_ok()), label }
  }
}

impl From<(String, String, RsaPublicKey)> for KeyEntry {
  fn from((kind, label, rsa_public_key): (String, String, RsaPublicKey)) -> Self {
    Self { kind, private: false, size: rsa_public_key.size(), validated: None, label }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum KeyLabel {
  Kind,
  Label,
  Private,
  SecretId,
  Size,
}

impl Label for KeyLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Kind => "kind",
      Self::Label => "label",
      Self::Private => "private",
      Self::SecretId => "secret id",
      Self::Size => "size",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretId)
  }
}

impl SubjectFormatter<KeyLabel> for KeyEntry {
  fn value(&self, label: &KeyLabel, target_id: &str) -> String {
    match label {
      KeyLabel::Kind => self.kind.clone(),
      KeyLabel::Label => self.label.clone(),
      KeyLabel::Private => {
        if self.private {
          "private".to_string()
        } else {
          "public".to_string()
        }
      }
      KeyLabel::SecretId => target_id.to_string(),
      KeyLabel::Size => self.size.to_string(),
    }
  }
}

#[derive(Clone, Serialize)]
struct SecretEntry {
  kind: String,
  format: String,
  description: Vec<String>,
  expires: Option<String>,
}

impl SecretEntry {
  fn new(kind: String, format: String, description: Option<String>, expires: Option<String>) -> Self {
    Self { kind, format, description: description.map(|desc| vec![desc]).unwrap_or_default(), expires }
  }
}

impl From<(String, String, X509Certificate<'_>)> for SecretEntry {
  fn from((kind, format, certificate): (String, String, X509Certificate)) -> Self {
    Self { kind, format, description: vec![x509_name_to_string(&certificate.subject)], expires: Some(certificate.validity().not_after.to_string()) }
  }
}

impl From<(String, String, RsaPrivateKey)> for SecretEntry {
  fn from((kind, format, _rsa_private_key): (String, String, RsaPrivateKey)) -> Self {
    Self::new(kind, format, None, None)
  }
}

impl From<(String, String, RsaPublicKey)> for SecretEntry {
  fn from((kind, format, _rsa_public_key): (String, String, RsaPublicKey)) -> Self {
    Self::new(kind, format, None, None)
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum SecretLabel {
  Description,
  Expires,
  Format,
  Kind,
  SecretId,
}

impl Label for SecretLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Description => "description",
      Self::Expires => "expires",
      Self::Format => "format",
      Self::Kind => "kind",
      Self::SecretId => "secret id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretId)
  }
}

impl SubjectFormatter<SecretLabel> for SecretEntry {
  fn value(&self, label: &SecretLabel, target_id: &str) -> String {
    match label {
      SecretLabel::Description => self.description.join("\n"),
      SecretLabel::Expires => self.expires.clone().unwrap_or_default(),
      SecretLabel::Format => self.format.clone(),
      SecretLabel::Kind => self.kind.clone(),
      SecretLabel::SecretId => target_id.to_string(),
    }
  }
}

static X509_CERTIFICATE_LABELS_SHOW: [X509CertificateLabel; 14] = [
  X509CertificateLabel::SecretId,
  X509CertificateLabel::Issuer,
  X509CertificateLabel::IssuerUid,
  X509CertificateLabel::NotAfter,
  X509CertificateLabel::NotBefore,
  X509CertificateLabel::Serial,
  X509CertificateLabel::Signature,
  X509CertificateLabel::SignatureAlgorithm,
  X509CertificateLabel::SignatureValue,
  X509CertificateLabel::Subject,
  X509CertificateLabel::SubjectPki,
  X509CertificateLabel::SubjectUid,
  X509CertificateLabel::TbsCertificate,
  X509CertificateLabel::Version,
];

static X509_CERTIFICATE_LABELS_LIST: [X509CertificateLabel; 5] =
  [X509CertificateLabel::SecretId, X509CertificateLabel::Subject, X509CertificateLabel::NotAfter, X509CertificateLabel::Issuer, X509CertificateLabel::Label];

static KEY_LABELS_LIST: [KeyLabel; 5] = [KeyLabel::SecretId, KeyLabel::Private, KeyLabel::Size, KeyLabel::Kind, KeyLabel::Label];

static SECRET_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretId, SecretLabel::Kind, SecretLabel::Format, SecretLabel::Description, SecretLabel::Expires];

fn get_certificates_from_pem(secret_value: &str) -> Option<Vec<CertificateEntry>> {
  let certificates = Pem::iter_from_buffer(secret_value.as_bytes())
    .flat_map(|pem_entry| pem_entry.ok())
    .flat_map(|pem| pem.parse_x509().map(|certificate| CertificateEntry::from((certificate, pem.label.clone()))))
    .collect_vec();
  if certificates.is_empty() {
    None
  } else {
    Some(certificates)
  }
}

fn get_keys_from_pem(secret_value: &str) -> Option<Vec<(RsaPrivateKey, String, &str)>> {
  let keys = Pem::iter_from_buffer(secret_value.as_bytes())
    .flat_map(|pem_entry| pem_entry.ok())
    .flat_map(|pem| {
      if let Ok(pkcs1_pem_private_key) = RsaPrivateKey::from_pkcs1_pem(secret_value) {
        Some((pkcs1_pem_private_key, pem.label.clone(), "pkcs1"))
      } else if let Ok(pkcs8_pem_private_key) = RsaPrivateKey::from_pkcs8_pem(secret_value) {
        Some((pkcs8_pem_private_key, pem.label.clone(), "pkcs8"))
      } else {
        None
      }
    })
    .collect_vec();
  if keys.is_empty() {
    None
  } else {
    Some(keys)
  }
}

fn secret_size(secret: &str) -> String {
  let trimmed_secret = secret.trim();
  if trimmed_secret.is_empty() {
    "empty".to_string()
  } else {
    let number_of_lines = trimmed_secret.lines().count();
    if number_of_lines == 1 {
      format!("single line, {} characters", secret.len())
    } else {
      format!("{} lines, {} characters", number_of_lines, secret.len())
    }
  }
}

fn get_json(secret_value: &str) -> Option<String> {
  match serde_json::from_str::<Value>(secret_value) {
    Ok(Value::Array(array)) => Some(format!("json array with {} elements ({})", array.len(), secret_size(secret_value))),
    Ok(Value::Object(_)) => Some(format!("json object ({})", secret_size(secret_value))),
    _ => None,
  }
}

fn get_toml(secret_value: &str) -> Option<String> {
  match toml::from_str::<Value>(secret_value) {
    Ok(Value::Array(array)) => Some(format!("toml array with {} elements ({})", array.len(), secret_size(secret_value))),
    Ok(Value::Object(_)) => Some(format!("toml object ({})", secret_size(secret_value))),
    _ => None,
  }
}

fn get_yaml(secret_value: &str) -> Option<String> {
  match serde_yaml::from_str::<Value>(secret_value) {
    Ok(Value::Array(array)) => Some(format!("yaml array with {} elements ({})", array.len(), secret_size(secret_value))),
    Ok(Value::Object(_)) => Some(format!("yaml object ({})", secret_size(secret_value))),
    _ => None,
  }
}

fn get_multiline(secret_value: &str) -> Option<String> {
  if secret_value.lines().count() > 1 {
    Some(secret_size(secret_value))
  } else {
    None
  }
}
