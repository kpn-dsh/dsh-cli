use crate::arguments::secret_id_argument;
use crate::capability::{
  Capability, CommandExecutor, COPY_COMMAND, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS,
  UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::Value as FormatterValue;
use crate::formatters::{Label, OutputFormat, SubjectFormatter};
use crate::modifier_flags::ModifierFlagType;
use crate::secret_entry::{get_certificates_from_pem, get_keys_from_pem, secret_entries_from, CertificateEntry, KeyEntry, SecretEntry, SecretKind};
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::{error, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::secret::{is_system_id, secret_id_to_secret_name, SecretInjection};
use dsh_api::types::Secret;
use dsh_api::{Dependant, DshApiError};
use futures::future::{join_all, try_join_all};
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use serde::Serialize;

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
      COPY_COMMAND => Some(SECRET_COPY_CAPABILITY.as_ref()),
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
  static ref SECRET_COPY_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(COPY_COMMAND, None, &SecretCopy {}, "Copy secret to clipboard").add_target_argument(secret_id_argument().required(true)));
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
  static ref SECRET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    SECRET_COPY_CAPABILITY.as_ref(),
    SECRET_CREATE_CAPABILITY.as_ref(),
    SECRET_DELETE_CAPABILITY.as_ref(),
    SECRET_LIST_CAPABILITY.as_ref(),
    SECRET_SHOW_CAPABILITY.as_ref(),
    SECRET_UPDATE_CAPABILITY.as_ref()
  ];
}

struct SecretCopy {}

#[async_trait]
impl CommandExecutor for SecretCopy {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    let start_instant = context.now();
    let secret = client.get_secret(&secret_id).await?;
    context.print_execution_time(start_instant);
    context.print_explanation(format!("show the value of secret '{}'", secret_id));
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(secret)) {
      Ok(_) => {
        context.print_outcome("secret copied to clipboard");
      }
      Err(error) => {
        debug!("clipboard error {}", error);
        context.print_error("could not secret token to clipboard")
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
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
    let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, context);
    for ((secret_name, is_system), secret_value_result) in secret_names.into_iter().zip(secret_value_results) {
      match secret_value_result {
        Ok(secret_value) => {
          for secret_entry in secret_entries_from(&secret_value, is_system) {
            formatter.push_target_id_value_owned(secret_name.clone(), secret_entry);
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
          formatter.push_target_id_value_owned(secret_name, SecretEntry::new(SecretKind::Error, kind.to_string(), message, None));
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
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
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
    let mut formatter = ListFormatter::new(&CERTIFICATE_LABELS_LIST, context);
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
    let mut formatter = ListFormatter::new(&KEY_LABELS_LIST, context);
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
    let mut formatter = ListFormatter::new(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
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
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, context);
    for (secret_id, dependants) in &secrets_with_dependants {
      for dependant in dependants {
        formatter.push_target_id_value(secret_id.clone(), dependant);
      }
    }
    if formatter.is_empty() {
      context.print_outcome("no secrets found in apps, proxies or services");
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
    UnitFormatter::new(secret_id, &DEFAULT_ALLOCATION_STATUS_LABELS, context).print(&allocation_status, None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let secret_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show the apps and services that use secret '{}'", secret_id));
    let start_instant = context.now();
    let usages = client.secret_dependants(&secret_id).await?;
    context.print_execution_time(start_instant);
    if usages.is_empty() {
      context.print_outcome("secret not used")
    } else {
      let mut formatter = ListFormatter::new(&DEPENDANT_LABELS, context);
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
    let (secret_value, allocation_status) = join!(client.get_secret(&secret_id), client.get_secret_status(&secret_id));
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, SECRET_SUBJECT_TARGET);
    for secret_entry in secret_entries_from(&secret_value?, is_system_id(&secret_id)) {
      _ = UnitFormatter::new(&secret_id, &SECRET_LABELS_LIST, context).print(&secret_entry, None);
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
pub(crate) enum CertificateLabel {
  Label,
  Issuer,
  NotAfter,
  _NotBefore,
  SecretId,
  Subject,
}

impl Label for CertificateLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Label => "label",
      Self::Issuer => "issuer",
      Self::NotAfter => "not after",
      Self::_NotBefore => "not before",
      Self::SecretId => "secret id",
      Self::Subject => "subject",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretId)
  }
}

impl SubjectFormatter<CertificateLabel> for CertificateEntry {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> FormatterValue {
    match label {
      CertificateLabel::Label => FormatterValue::plain(&self.label),
      CertificateLabel::Issuer => FormatterValue::plain(&self.issuer),
      CertificateLabel::NotAfter => FormatterValue::plain(&self.not_after), // TODO
      CertificateLabel::_NotBefore => FormatterValue::plain(&self.not_before),
      CertificateLabel::SecretId => FormatterValue::target(target_id),
      CertificateLabel::Subject => FormatterValue::plain(&self.subject),
    }
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
  fn value(&self, label: &KeyLabel, target_id: &str) -> crate::formatters::Value {
    match label {
      KeyLabel::Kind => FormatterValue::plain(&self.kind),
      KeyLabel::Label => FormatterValue::plain(&self.label),
      KeyLabel::Private => {
        if self.private {
          FormatterValue::plain("private")
        } else {
          FormatterValue::plain("public")
        }
      }
      KeyLabel::SecretId => FormatterValue::target(target_id),
      KeyLabel::Size => FormatterValue::plain(self.size),
    }
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
  fn value(&self, label: &SecretLabel, target_id: &str) -> FormatterValue {
    match label {
      SecretLabel::Description => FormatterValue::plain(self.description.join("\n")),
      SecretLabel::Expires => FormatterValue::option(self.expires.clone()),
      SecretLabel::Format => FormatterValue::plain(&self.format),
      SecretLabel::Kind => FormatterValue::plain(&self.kind),
      SecretLabel::SecretId => FormatterValue::target(target_id),
    }
  }
}

static CERTIFICATE_LABELS_LIST: [CertificateLabel; 5] =
  [CertificateLabel::SecretId, CertificateLabel::Subject, CertificateLabel::NotAfter, CertificateLabel::Issuer, CertificateLabel::Label];

static KEY_LABELS_LIST: [KeyLabel; 5] = [KeyLabel::SecretId, KeyLabel::Private, KeyLabel::Size, KeyLabel::Kind, KeyLabel::Label];

pub(crate) static SECRET_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretId, SecretLabel::Kind, SecretLabel::Format, SecretLabel::Description, SecretLabel::Expires];
