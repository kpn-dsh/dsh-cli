use crate::arguments::secret_id_argument;
use crate::capability::{
  Capability, CommandExecutor, COPY_COMMAND, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS,
  UPDATE_COMMAND,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::error::DshCliError;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::Value;
use crate::formatters::{Label, OutputFormat, SubjectFormatter};
use crate::global_options::{expiration_option, get_expiration_days};
use crate::issues::{Issue, IssueLabel, Severity};
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
use dsh_api::secret::{normalize_secret_name, SecretInjection};
use dsh_api::types::{AllocationStatus, Secret};
use dsh_api::Dependant;
use futures::future::{join, join_all, try_join_all};
use futures::{join, FutureExt};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;
use serde::Serialize;
use std::cmp::PartialEq;

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
        (FlagType::Errors, &SecretListErrors {}, None),
        (FlagType::Ids, &SecretListIds {}, None),
        (FlagType::Issues, &SecretListIssues {}, None),
        (FlagType::Keys, &SecretListKeys {}, None),
        (FlagType::System, &SecretListSystem {}, None),
        (FlagType::Usage, &SecretListUsage {}, None)
      ])
      .add_extra_argument(expiration_option())
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
      .add_extra_argument(expiration_option())
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
    let (secret_name, secret_id) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    if secret_id.is_some() {
      return err!("system secret '{}' cannot be created", secret_name);
    }
    if client.get_secret(&secret_name).await.is_ok() {
      return err!("secret '{}' already exists", secret_name);
    }
    if context.stdin_is_terminal() {
      if matches.get_flag(ModifierFlagType::MultiLine.id()) {
        context.print_explanation(format!("create new multi-line secret '{}'", secret_name));
        let secret = context.read_multi_line("enter multi-line secret (terminate input with ctrl-d after last line)")?;
        let secret = Secret { name: secret_name.clone(), value: secret };
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not created");
        } else {
          client.post_secret(&secret).await?;
          context.print_outcome(format!("secret '{}' created", secret_name));
        }
      } else {
        context.print_explanation(format!("create new single line secret '{}'", secret_name));
        let secret = context.read_single_line_password("enter secret")?;
        let secret = Secret { name: secret_name.clone(), value: secret };
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not created");
        } else {
          client.post_secret(&secret).await?;
          context.print_outcome(format!("secret '{}' created", secret_name));
        }
      }
    } else {
      let secret = context.read_multi_line("")?;
      let secret = Secret { name: secret_name.clone(), value: secret };
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not created");
      } else {
        client.post_secret(&secret).await?;
        context.print_outcome(format!("secret '{}' created", secret_name));
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
    let (secret_name, secret_id) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    if secret_id.is_some() {
      return err!("system secret '{}' cannot be deleted", secret_name);
    }
    if client.get_secret_configuration(&secret_name).await.is_err() {
      return err!("secret '{}' does not exist", secret_name);
    }
    if context.dependencies_warning("secret", client.secret_dependants(&secret_name).await?, &secret_name) && !context.confirmed("do you want to continue?")? {
      context.print_outcome(format!("cancelled, secret '{}' not deleted", secret_name));
      return Ok(());
    }
    if context.confirmed(format!("delete secret '{}'?", secret_name))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not deleted");
      } else {
        client.delete_secret_configuration(&secret_name).await?;
        context.print_outcome(format!("secret '{}' deleted", secret_name));
      }
    } else {
      context.print_outcome(format!("cancelled, secret '{}' not deleted", secret_name));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static SECRET_LABELS_LIST: [SecretLabel; 10] = [
  SecretLabel::SecretName,
  SecretLabel::System,
  SecretLabel::Kind,
  SecretLabel::FormatKind,
  SecretLabel::Size,
  SecretLabel::Description,
  SecretLabel::NotBefore,
  SecretLabel::NotAfter,
  SecretLabel::Provisioned,
  SecretLabel::Notifications,
];

struct SecretList {}

#[async_trait]
impl CommandExecutor for SecretList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secrets: Vec<(String, Option<String>, SecretMetadata, Option<AllocationStatus>, Vec<Dependant<SecretInjection>>)> = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, context);
    for (secret_name, secret_id, secret_metadata, allocation_status, _dependants) in secrets.into_iter() {
      formatter.push_target_id_value_owned(
        secret_name.clone(),
        (secret_id.clone(), secret_metadata, Some(expiration_days), allocation_status.clone()),
      );
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

static CERTIFICATE_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Subject, SecretLabel::NotAfter, SecretLabel::Issuer, SecretLabel::Label];

struct SecretListCertificates {}

#[async_trait]
impl CommandExecutor for SecretListCertificates {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that contain certificates");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&CERTIFICATE_LABELS_LIST, context);
    for (secret_name, _, secret_metadata, _, _) in secrets_with_metadata {
      if matches!(secret_metadata, SecretMetadata::Certificate { .. }) {
        formatter.push_target_id_value_owned(secret_name.clone(), (secret_metadata, Some(expiration_days)));
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretListErrors {}

#[async_trait]
impl CommandExecutor for SecretListErrors {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that have errors");
    list_issues(client, matches, context, true).await?
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

struct SecretListIssues {}

#[async_trait]
impl CommandExecutor for SecretListIssues {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that have potential issues");
    list_issues(client, matches, context, false).await?
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static SECRET_LIST_ISSUES_LABELS_LIST: [IssueLabel; 5] =
  [IssueLabel::Target, IssueLabel::IssueKind, IssueLabel::SubjectKind, IssueLabel::SubjectDescription, IssueLabel::IssueDetails];

async fn list_issues(client: &DshApiClient, matches: &ArgMatches, context: &Context, only_errors: bool) -> Result<Result<(), DshCliError>, DshCliError> {
  let expiration_days = get_expiration_days(matches, context.settings())?;
  let start_instant = context.now();
  let secrets: Vec<(String, Option<String>, SecretMetadata, Option<AllocationStatus>, Vec<Dependant<SecretInjection>>)> = secrets_with_metadata(client).await?;
  context.print_execution_time(start_instant);
  let secrets_issues: Vec<(String, SecretMetadata, Vec<Issue>)> = secrets
    .iter()
    .flat_map(|secret_tuple| has_issues(secret_tuple, Some(expiration_days), only_errors).map(|issues| (secret_tuple.0.clone(), secret_tuple.2.clone(), issues)))
    .collect_vec();
  let mut formatter = ListFormatter::new_override_target_id_label(&SECRET_LIST_ISSUES_LABELS_LIST, "secret id", context);
  for (secret_name, secret_metadata, issues) in secrets_issues.into_iter() {
    for issue in issues {
      formatter.push_target_id_value_owned(secret_name.clone(), (secret_metadata.clone(), issue));
    }
  }
  formatter.print(None)?;
  Ok(Ok(()))
}

static KEY_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Private, SecretLabel::Size, SecretLabel::Kind, SecretLabel::Label];

struct SecretListKeys {}

#[async_trait]
impl CommandExecutor for SecretListKeys {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that contain private of public keys");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&KEY_LABELS_LIST, context);
    for (secret_name, _, secret_metadata, _, _) in secrets_with_metadata {
      if let SecretMetadata::Pki { .. } = &secret_metadata {
        formatter.push_target_id_value_owned(secret_name.clone(), (secret_metadata, Some(expiration_days)));
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static SYSTEM_LABELS_LIST: [SecretLabel; 4] = [SecretLabel::SecretName, SecretLabel::SecretId, SecretLabel::FormatKind, SecretLabel::Size];

struct SecretListSystem {}

#[async_trait]
impl CommandExecutor for SecretListSystem {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all system secret ids");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SYSTEM_LABELS_LIST, context);
    for (secret_name, secret_id, secret_metadata, _, _) in secrets_with_metadata {
      if secret_id.is_some() {
        formatter.push_target_id_value_owned(secret_name.clone(), (secret_id.clone(), secret_metadata, Some(expiration_days), None));
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

static SECRET_LABELS_SHOW: [SecretLabel; 14] = [
  SecretLabel::SecretName,
  SecretLabel::SecretId,
  SecretLabel::System,
  SecretLabel::Kind,
  SecretLabel::FormatKind,
  SecretLabel::Size,
  SecretLabel::Description,
  SecretLabel::NotBefore,
  SecretLabel::NotAfter,
  SecretLabel::Provisioned,
  SecretLabel::Notifications,
  SecretLabel::DerivedFrom,
  SecretLabel::Subject,
  SecretLabel::Issuer,
];

struct SecretShow {}

#[async_trait]
impl CommandExecutor for SecretShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let (secret_name, secret_id) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let (secret_value, allocation_status) = join!(client.get_secret(&secret_name), client.get_secret_status(&secret_name));
    context.print_execution_time(start_instant);
    context.print_allocation_status(&allocation_status, SECRET_SUBJECT_TARGET);
    _ = UnitFormatter::new(&secret_name, &SECRET_LABELS_SHOW, context).print(
      &(
        secret_id.clone(),
        secret_metadata(&secret_value?),
        Some(expiration_days),
        allocation_status.clone().ok(),
      ),
      None,
    );
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
    let (secret_name, secret_id) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    if secret_id.is_some() {
      context.print_warning(format!("system secret '{}' has no allocation status", secret_name));
      Ok(())
    } else {
      context.print_explanation(format!("show allocation status for secret '{}'", secret_name));
      let start_instant = context.now();
      let allocation_status = client.get_secret_status(&secret_name).await;
      context.print_execution_time(start_instant);
      context.print_allocation_status(&allocation_status, SECRET_SUBJECT_TARGET);
      UnitFormatter::new(secret_name, &DEFAULT_ALLOCATION_STATUS_LABELS, context).print(&allocation_status?, None)
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct SecretShowUsage {}

#[async_trait]
impl CommandExecutor for SecretShowUsage {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let (secret_name, _) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    context.print_explanation(format!("show the apps and services that use secret '{}'", secret_name));
    let start_instant = context.now();
    let usages = client.secret_dependants(&secret_name).await?;
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

struct SecretShowValue {}

#[async_trait]
impl CommandExecutor for SecretShowValue {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let (secret_name, _) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    let start_instant = context.now();
    let secret = client.get_secret(&secret_name).await?;
    context.print_execution_time(start_instant);
    context.print_explanation(format!("show the value of secret '{}'", secret_name));
    context.println(secret);
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::new(false, None, true)
  }
}

struct SecretUpdate {}

#[async_trait]
impl CommandExecutor for SecretUpdate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let (secret_name, secret_id) = normalize_secret_name(target.unwrap_or_else(|| unreachable!()));
    if secret_id.is_some() {
      return err!("system secret '{}' cannot be updated", secret_name);
    }
    if client.get_secret(&secret_name).await.is_err() {
      return err!("secret '{}' does not exist", secret_name);
    }
    if context.dependencies_warning("secret", client.secret_dependants(&secret_name).await?, &secret_name) && !context.confirmed("do you want to continue?")? {
      context.print_outcome(format!("cancelled, secret '{}' not updated", secret_name));
      return Ok(());
    }
    if context.stdin_is_terminal() {
      if matches.get_flag(ModifierFlagType::MultiLine.id()) {
        context.print_explanation(format!("update multi-line secret '{}'", secret_name));
        let secret = context.read_multi_line("enter multi-line secret (terminate input with ctrl-d after last line)")?;
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not updated");
        } else {
          client.put_secret(&secret_name, secret).await?;
          context.print_outcome(format!("secret '{}' updated", secret_name));
        }
      } else {
        context.print_explanation(format!("update single line secret '{}'", secret_name));
        let secret = context.read_single_line_password("enter secret")?;
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not updated");
        } else {
          client.put_secret(&secret_name, secret).await?;
          context.print_outcome(format!("secret '{}' updated", secret_name));
        }
      }
    } else {
      let secret = context.read_multi_line("")?;
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not updated");
      } else {
        client.put_secret(&secret_name, secret).await?;
        context.print_outcome(format!("secret '{}' updated", secret_name));
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

/// Gets secret with metadata.
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
pub(crate) async fn secret_with_metadata(secret_name: String, client: &DshApiClient) -> (SecretMetadata, Option<AllocationStatus>) {
  match join(client.get_secret(&secret_name), client.get_secret_status(&secret_name)).await {
    (Ok(secret_value), Ok(allocation_status)) => (secret_metadata(&secret_value), Some(allocation_status)),
    (Ok(secret_value), Err(_)) => (secret_metadata(&secret_value), None),
    (Err(get_secret_error), Ok(allocation_status)) => match get_secret_error {
      DshApiError::NotFound { .. } => (
        SecretMetadata::Misconfiguration { message: "secret not found, possibly pending".to_string() },
        Some(allocation_status),
      ),
      other_error => (SecretMetadata::from(other_error), Some(allocation_status)),
    },
    (Err(get_secret_error), Err(_)) => match get_secret_error {
      DshApiError::NotFound { .. } => (
        SecretMetadata::Misconfiguration { message: "secret allocation status not found, possibly pending".to_string() },
        None,
      ),
      other_error => (SecretMetadata::from(other_error), None),
    },
  }
}

/// Gets all secrets with metadata and allocation status.
///
/// # Returns
/// List of tuples, each consisting of:
/// * `String` - Secret name.
/// * `Option<String>` - Secret id when secret is a system secret, empty otherwise.
/// * `Vec<SecretMetadata>` - List containing the metadata.
/// * `Option<AllocationStatus>` - Secrets allocation status for non-system secrets.
/// * `Vec<Dependant<SecretInjection>>` - Apps, applications and proxies that depend on the secret.
pub(crate) async fn secrets_with_metadata(
  client: &DshApiClient,
) -> DshCliResult<Vec<(String, Option<String>, SecretMetadata, Option<AllocationStatus>, Vec<Dependant<SecretInjection>>)>> {
  let secrets: Vec<(String, Option<String>, Vec<Dependant<SecretInjection>>)> = client.secrets_with_dependants().await?;
  Ok(
    join_all(secrets.into_iter().map(|(secret_name, secret_id, dependants)| {
      secret_with_metadata(secret_name.clone(), client).map(|(metadata, allocation_status)| (secret_name, secret_id, metadata, allocation_status, dependants))
    }))
    .await,
  )
}

/// Check if a secret has issues.
///
/// # Parameters
/// * `secret_tuple` - Tuple of secret parameters, consisting of
///   * `String` Secret name.
///   * `Option<String>` - Secret id when it is a system secret.
///   * `SecretMetadata` - Secret metadata.
///   * `Option<AllocationStatus>` - Secret allocation status.
///   * `Vec<Dependant<SecretInjection>>` List of apps, applications and proxies that depend
///     on the secret.
/// * `days` - Number of days until expiration.
/// * `only_errors` - If `true` only issues with severity level `Severity::Error` will be returned.
///
/// # Returns
/// * `Some(Vec<Issue>)` - List of found issues (at least one).
/// * `None` - No issues where found.
pub(crate) fn has_issues(
  secret_tuple: &(String, Option<String>, SecretMetadata, Option<AllocationStatus>, Vec<Dependant<SecretInjection>>),
  days: Option<u64>,
  only_errors: bool,
) -> Option<Vec<Issue>> {
  let (_, secret_id, secret_metadata, allocation_status, dependants) = secret_tuple;
  let mut issues: Vec<Issue> = vec![];
  match secret_metadata {
    SecretMetadata::Certificate { not_after, not_before, .. } => {
      if let Some(issue) = Issue::timestamp_expired(*not_after as i64, days) {
        issues.push(issue);
      }
      if let Some(issue) = Issue::timestamp_before(*not_before as i64) {
        issues.push(issue);
      }
    }
    SecretMetadata::Empty => {
      issues.push(Issue::Empty);
    }
    SecretMetadata::Error { message } => {
      issues.push(Issue::IncorrectValue { explanation: message.clone() });
    }
    SecretMetadata::Misconfiguration { message } => {
      issues.push(Issue::Misconfiguration { explanation: message.clone() });
    }
    SecretMetadata::NotFound { message } => {
      issues.push(Issue::Misconfiguration { explanation: message.clone().unwrap_or_default().to_string() });
    }
    SecretMetadata::Pki { .. } => {}
    SecretMetadata::Regular { .. } => {}
    SecretMetadata::Settings { .. } => {}
  }
  if secret_id.is_none() && dependants.is_empty() {
    issues.push(Issue::NotUsed)
  }
  if let Some(allocation_status) = allocation_status {
    if !allocation_status.provisioned {
      issues.push(Issue::NotProvisioned)
    }
    for notification in &allocation_status.notifications {
      if notification.remove {
        issues.push(Issue::RemovalNotification { notification: notification.clone() });
      } else {
        issues.push(Issue::CreationUpdateNotification { notification: notification.clone() });
      }
    }
  }
  if only_errors {
    issues.retain(|issue| issue.severity() == Severity::Error);
  }
  if issues.is_empty() {
    None
  } else {
    Some(issues)
  }
}

impl SubjectFormatter<IssueLabel> for (SecretMetadata, Issue) {
  fn value(&self, label: &IssueLabel, target_id: &str) -> Value {
    let (secret_metadata, issue) = self;
    match label {
      IssueLabel::IssueDetails => issue.value(label, target_id),
      IssueLabel::IssueKind => issue.value(label, target_id),
      IssueLabel::SubjectDescription => Value::some_or_hide(secret_metadata.value_description()),
      IssueLabel::SubjectKind => Value::some_or_hide(secret_metadata.kind()),
      IssueLabel::Target => Value::target(target_id),
      IssueLabel::DependencyName => Value::not_applicable(),
      IssueLabel::DependencySubject => Value::not_applicable(),
      IssueLabel::DependencyValue => Value::not_applicable(),
    }
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum SecretLabel {
  CaChain,
  DerivedFrom,
  Description,
  // Expires,
  Format,
  FormatKind,
  Issuer,
  Kind,
  Label,
  NotBefore,
  NotAfter,
  Notifications,
  NumberOfEntries,
  Private,
  Provisioned,
  SecretId,
  SecretName,
  Size,
  Subject,
  System,
}

impl Label for SecretLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CaChain => "ca chain",
      Self::DerivedFrom => "derived from",
      Self::Description => "description",
      Self::Format => "format",
      Self::FormatKind => "format kind",
      Self::Issuer => "issuer",
      Self::Kind => "kind",
      Self::Label => "label",
      Self::Notifications => "notifications",
      Self::NotBefore => "not before",
      Self::NotAfter => "not after",
      Self::NumberOfEntries => "entries",
      Self::Private => "private",
      Self::Provisioned => "provisioned",
      Self::SecretId => "secret id",
      Self::SecretName => "secret name",
      Self::Size => "size",
      Self::Subject => "subject",
      Self::System => "system",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::SecretName)
  }
}

/// * `Option<String>` - Secret id.
/// * `SecretMetadata` - Secret metadata.
/// * `Option<u64>` - Expiration days.
/// * `AllocationStatus` - Secret allocation status.
impl SubjectFormatter<SecretLabel> for (Option<String>, SecretMetadata, Option<u64>, Option<AllocationStatus>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    let (secret_id, secret_metadata, expiration_days, allocation_status) = self;
    match label {
      SecretLabel::DerivedFrom => match allocation_status.clone().and_then(|allocation_status| allocation_status.derived_from) {
        Some(derived_from) => Value::plain(derived_from),
        None => Value::hide(),
      },
      SecretLabel::Notifications => match allocation_status {
        Some(allocation_status) if !allocation_status.notifications.is_empty() => {
          Value::warn(allocation_status.notifications.iter().map(|notification| notification.to_string()).join("\n"))
        }
        _ => Value::hide(),
      },
      SecretLabel::Provisioned => match allocation_status {
        Some(allocation_status) => {
          if allocation_status.provisioned {
            Value::plain("yes")
          } else {
            Value::plain("no")
          }
        }
        None => Value::hide(),
      },
      SecretLabel::SecretId => match secret_id {
        Some(secret_id) => Value::target(secret_id),
        None => Value::hide(),
      },
      SecretLabel::SecretName => Value::target(target_id),
      SecretLabel::System => {
        if secret_id.is_some() {
          Value::plain("yes")
        } else {
          Value::plain("no")
        }
      }
      _ => (secret_metadata.clone(), *expiration_days).value(label, target_id),
    }
  }
}

/// * `SecretMetadata` - Secret metadata.
/// * `Option<u64>` - Expiration days.
impl SubjectFormatter<SecretLabel> for (SecretMetadata, Option<u64>) {
  fn value(&self, label: &SecretLabel, target_id: &str) -> Value {
    let (secret_metadata, expiration_days) = self;
    match label {
      SecretLabel::Description => match secret_metadata.kind() {
        Some("error") => Value::error(secret_metadata.additional_info().map(|info| info.to_string()).unwrap_or_default()),
        _ => Value::some_or_hide(secret_metadata.additional_info()),
      },
      SecretLabel::Format => Value::plain(secret_metadata.format()),
      SecretLabel::FormatKind => Value::some_or_hide(secret_metadata.format_kind()),
      SecretLabel::Issuer => match secret_metadata {
        SecretMetadata::Certificate { issuer, .. } => Value::distinguished_name(issuer),
        _ => Value::hide(),
      },
      SecretLabel::Kind => match secret_metadata.kind() {
        Some("error") => Value::error("ERROR"),
        _ => Value::some_or_hide(secret_metadata.kind()),
      },
      SecretLabel::Label => match secret_metadata {
        SecretMetadata::Certificate { label, .. } => Value::plain(label),
        SecretMetadata::Pki { labels, .. } => Value::plain(labels.join("/")),
        _ => Value::hide(),
      },
      SecretLabel::NotBefore => match secret_metadata {
        SecretMetadata::Certificate { not_before, .. } => Value::timestamp_seconds_not_before(*not_before as i64),
        _ => Value::hide(),
      },
      SecretLabel::NotAfter => match secret_metadata {
        SecretMetadata::Certificate { not_after, .. } => Value::timestamp_seconds_expired(*not_after as i64, *expiration_days),
        _ => Value::hide(),
      },
      SecretLabel::NumberOfEntries => Value::some_or_hide(secret_metadata.number_of_entries()),
      SecretLabel::Private => match secret_metadata {
        SecretMetadata::Pki { private, .. } => Value::plain(if *private { "private" } else { "public" }),
        _ => Value::hide(),
      },
      SecretLabel::SecretName => Value::target(target_id),
      SecretLabel::Size => Value::some_or_hide(secret_metadata.secret_size()),
      SecretLabel::Subject => match secret_metadata {
        SecretMetadata::Certificate { subject, .. } => Value::distinguished_name(subject),
        _ => Value::hide(),
      },
      _ => Value::unreachable(),
    }
  }
}
