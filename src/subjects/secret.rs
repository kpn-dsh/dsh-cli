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
use crate::formatters::Value;
use crate::formatters::{Label, OutputFormat, SubjectFormatter};
use crate::modifier_flags::ModifierFlagType;
use crate::secret_metadata::{secret_metadata, SecretMetadata};
use crate::subject::{Requirements, Subject};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::{err, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::DshApiError;
use dsh_api::secret::SecretInjection;
use dsh_api::types::{AllocationStatus, Secret};
use dsh_api::Dependant;
use futures::future::{join, join_all, try_join_all};
use futures::{join, FutureExt};
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

const EXPIRATION_CHECK_DAYS: Option<u64> = Some(30);

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
      return err!("secret '{}' already exists", secret_id);
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
      return err!("secret '{}' does not exist", secret_id);
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
    let secrets: Vec<(String, Option<String>, Vec<SecretMetadata>, Option<AllocationStatus>)> = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, context);
    for (secret_name, secret_id, secret_metadatas, allocation_status) in secrets.into_iter() {
      for secret_metadata in secret_metadatas {
        formatter.push_target_id_value_owned(secret_name.clone(), (secret_id.clone(), secret_metadata, allocation_status.clone()));
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
    let non_system_secret_ids = client.secret_names_non_system().await?;
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
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&CERTIFICATE_LABELS_LIST, context);
    for (secret_name, _, secret_metadatas, _) in secrets_with_metadata {
      for secret_metadata in secret_metadatas {
        if let SecretMetadata::Certificate(_subject, _not_after, _not_before, _issuer, _label) = &secret_metadata {
          formatter.push_target_id_value_owned(secret_name.clone(), secret_metadata);
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
    let secret_names = client.secret_names().await?.into_iter().map(|(secret_name, _)| secret_name).collect_vec();
    context.print_execution_time(start_instant);
    let header = format!("secret ids ({})", secret_names.len());
    let mut formatter = IdsFormatter::new(&header, context);
    formatter.push_target_ids(secret_names.as_slice());
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
    context.print_explanation("list all secrets that contain private of public keys");
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&KEY_LABELS_LIST, context);
    for (secret_name, _, secret_metadatas, _) in secrets_with_metadata {
      for secret_metadata in secret_metadatas {
        if let SecretMetadata::Pki(_secret_format, _private, _label, _algorithm) = &secret_metadata {
          formatter.push_target_id_value_owned(secret_name.clone(), secret_metadata);
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
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SYSTEM_LABELS_LIST, context);
    for (secret_name, secret_id, secret_metadatas, _) in secrets_with_metadata {
      if secret_id.is_some() {
        for secret_metadata in secret_metadatas {
          formatter.push_target_id_value_owned(secret_name.clone(), (secret_id.clone(), secret_metadata, None));
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

struct SecretListUsage {}

#[async_trait]
impl CommandExecutor for SecretListUsage {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that are used in apps or services");
    let start_instant = context.now();
    let secrets_with_dependants: Vec<(String, Option<String>, Vec<Dependant<SecretInjection>>)> = client.secrets_with_dependants().await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&DEPENDANT_LABELS_LIST, context);
    for (secret_name, _, dependants) in &secrets_with_dependants {
      for dependant in dependants {
        formatter.push_target_id_value(secret_name.clone(), dependant);
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
    for secret_entry in secret_metadata(&secret_value?) {
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
      return err!("secret '{}' does not exist", secret_id);
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

/// Get secret with metadata
///
/// Gets the secret value and optional allocation status. The value will be converted to a
/// `SecretMetadata` struct. Note that system secrets do not have an allocation status.
///
/// # Parameters
/// * `secret_name` - Secret name.
/// * `client` - Dsh api client.
///
/// # Returns
/// Tuple consisting of:
/// * `Vec<SecretMetadata>` - List containing the metadata items.
/// * `Option<AllocationStatus>` - Secrets allocation status for non-system secrets,
///   empty otherwise.
pub(crate) async fn secret_with_metadata(secret_name: &str, client: &DshApiClient) -> (Vec<SecretMetadata>, Option<AllocationStatus>) {
  match join(
    client.get_secret(secret_name),
    client.get_secret_status(secret_name).map(|allocation_status| allocation_status.ok()),
  )
  .await
  {
    (Ok(secret_value), status) => (secret_metadata(&secret_value), status),
    (Err(error), allocation_status) => match error {
      DshApiError::NotFound(_) => (vec![SecretMetadata::Error("secret not found, possibly pending".to_string())], allocation_status),
      DshApiError::BadRequest(_) => (vec![SecretMetadata::Error("bad request".to_string())], None),
      DshApiError::Configuration(_) => (vec![SecretMetadata::Error("configuration error".to_string())], None),
      DshApiError::Conversion(_) => (vec![SecretMetadata::Error("conversion error".to_string())], None),
      DshApiError::NotAuthorized(_) => (vec![SecretMetadata::Error("not authorized".to_string())], None),
      DshApiError::Parameter(_) => (vec![SecretMetadata::Error("parameter error".to_string())], None),
      DshApiError::Unexpected(_, _) => (vec![SecretMetadata::Error("unexpected error, possibly a network failure".to_string())], None),
      DshApiError::Unprocessable(_) => (vec![SecretMetadata::Error("unprocessable".to_string())], None),
    },
  }
}

/// Get all secrets with metadata and allocation status
///
/// # Returns
/// List of tuples, each consisting of:
/// * `String` - Secret name.
/// * `Option<String>` - Secret id when secret is a system secret, empty otherwise.
/// * `Vec<SecretMetadata>` - List containing the metadata.
/// * `Option<AllocationStatus>` - Secrets allocation status for non-system secrets,
pub(crate) async fn secrets_with_metadata(client: &DshApiClient) -> DshCliResult<Vec<(String, Option<String>, Vec<SecretMetadata>, Option<AllocationStatus>)>> {
  let secret_names: Vec<(String, Option<String>)> = client.secret_names().await?;
  Ok(
    join_all(secret_names.iter().map(|(secret_name, secret_id)| {
      secret_with_metadata(secret_name, client).map(|(metadata, allocation_status)| (secret_name.clone(), secret_id.clone(), metadata, allocation_status))
    }))
    .await,
  )
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum SecretLabel {
  Description,
  Expires,
  _Format,
  FormatKind,
  Issuer,
  Kind,
  Label,
  _NotBefore,
  NotAfter,
  Notifications,
  _NumberOfEntries,
  Private,
  SecretId,
  SecretName,
  Size,
  Status,
  Subject,
  System,
}

impl Label for SecretLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Description => "description",
      Self::Expires => "expires",
      Self::_Format => "format",
      Self::FormatKind => "format",
      Self::Issuer => "issuer",
      Self::Kind => "kind",
      Self::Label => "label",
      Self::Notifications => "notifications",
      Self::_NotBefore => "not before",
      Self::NotAfter => "not after",
      Self::_NumberOfEntries => "entries",
      Self::Private => "private",
      Self::SecretId => "secret id",
      Self::SecretName => "secret name",
      Self::Size => "size",
      Self::Status => "status",
      Self::Subject => "subject",
      Self::System => "system",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretName)
  }
}

impl SubjectFormatter<SecretLabel> for (Option<String>, SecretMetadata, Option<AllocationStatus>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    let (secret_id, secret_metadata, allocation_status) = self;
    match label {
      SecretLabel::Notifications => match allocation_status {
        Some(allocation_status) => Value::warn(allocation_status.notifications.iter().map(|notification| notification.to_string()).join("\n")),
        None => Value::empty(),
      },
      SecretLabel::SecretId => match secret_id {
        Some(secret_id) => Value::target(secret_id),
        None => Value::empty(),
      },
      SecretLabel::SecretName => Value::target(target_id),
      SecretLabel::Status => match allocation_status {
        Some(allocation_status) => {
          if allocation_status.provisioned {
            Value::empty()
          } else {
            Value::warn("not provisioned")
          }
        }
        None => Value::empty(),
      },
      SecretLabel::System => {
        if secret_id.is_some() {
          Value::plain("yes")
        } else {
          Value::empty()
        }
      }
      _ => secret_metadata.value(label, target_id),
    }
  }
}

impl SubjectFormatter<SecretLabel> for SecretMetadata {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    match label {
      SecretLabel::Description => match self.kind() {
        "error" => Value::error(self.additional_info().map(|info| info.to_string()).unwrap_or_default()),
        _ => Value::plain(self.kind()),
      },
      SecretLabel::Expires => match self.not_after() {
        Some(not_after) => Value::timestamp_seconds_expired(not_after as i64, EXPIRATION_CHECK_DAYS),
        None => Value::empty(),
      },
      SecretLabel::_Format => Value::plain(self.format()),
      SecretLabel::FormatKind => Value::option(self.format_kind()),
      SecretLabel::Issuer => match self {
        SecretMetadata::Certificate(_, _, _, issuer, _) => Value::plain(issuer),
        _ => Value::not_applicable(),
      },
      SecretLabel::Kind => match self.kind() {
        "error" => Value::error(self.kind()),
        _ => Value::plain(self.kind()),
      },
      SecretLabel::Label => match self {
        SecretMetadata::Certificate(_, _, _, _, label) => Value::plain(label),
        SecretMetadata::Pki(_, _, label, _) => Value::plain(label),
        _ => Value::not_applicable(),
      },
      SecretLabel::_NotBefore => match self {
        SecretMetadata::Certificate(_, _, not_before, _, _) => Value::plain(not_before),
        _ => Value::not_applicable(),
      },
      SecretLabel::NotAfter => match self {
        SecretMetadata::Certificate(_, not_after, _, _, _) => Value::plain(not_after),
        _ => Value::not_applicable(),
      },
      SecretLabel::_NumberOfEntries => Value::option(self.number_of_entries()),
      SecretLabel::Private => match self {
        SecretMetadata::Pki(_, private, _, _) => Value::plain(if *private { "private" } else { "public" }),
        _ => Value::not_applicable(),
      },
      SecretLabel::SecretName => Value::target(target_id),
      SecretLabel::Size => Value::option(self.secret_size()),
      SecretLabel::Subject => match self {
        SecretMetadata::Certificate(subject, _, _, _, _) => Value::plain(subject),
        _ => Value::not_applicable(),
      },
      _ => Value::not_applicable(),
    }
  }
}

static CERTIFICATE_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Subject, SecretLabel::NotAfter, SecretLabel::Issuer, SecretLabel::Label];

static KEY_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Private, SecretLabel::Size, SecretLabel::Kind, SecretLabel::Label];

static SYSTEM_LABELS_LIST: [SecretLabel; 4] = [SecretLabel::SecretName, SecretLabel::SecretId, SecretLabel::FormatKind, SecretLabel::Size];

pub(crate) static SECRET_LABELS_LIST: [SecretLabel; 9] = [
  SecretLabel::SecretName,
  SecretLabel::System,
  SecretLabel::Kind,
  SecretLabel::FormatKind,
  SecretLabel::Size,
  SecretLabel::Description,
  SecretLabel::Expires,
  SecretLabel::Status,
  SecretLabel::Notifications,
];
