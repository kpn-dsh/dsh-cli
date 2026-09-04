use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::global_options::get_expiration_days;
use crate::issues::{Issue, IssueLabel, Severity};
use crate::modifier_flags::ModifierFlagType;
use crate::secret_metadata::{secrets_with_metadata, SecretMetadata};
use crate::subject::Requirements;
use crate::subjects::secret::labels::{CertificateSecretLabel, PkiSecretLabel, SecretLabel, SecretMetadataIssue};
use crate::subjects::secret::{has_issues, SecretWithMetadata, SECRET_SUBJECT_TARGET};
use crate::subjects::{DEFAULT_ALLOCATION_STATUS_LABELS, DEPENDANT_LABELS, DEPENDANT_LABELS_LIST};
use crate::{err, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::error::DshApiError;
use dsh_api::secret::{normalize_secret_name, SecretInjection};
use dsh_api::types::Secret;
use dsh_api::Dependant;
use futures::future::try_join_all;
use futures::join;
use itertools::Itertools;
use log::debug;

pub(crate) struct SecretCopy {}

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

pub(crate) struct SecretCreate {}

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

pub(crate) struct SecretDelete {}

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

static SECRET_LABELS_LIST: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Kind, SecretLabel::Format, SecretLabel::Bytes, SecretLabel::Issues];

pub(crate) struct SecretList {}

#[async_trait]
impl CommandExecutor for SecretList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secrets: Vec<SecretWithMetadata> = secrets_with_metadata(client).await?;
    let secrets_issues: Vec<Vec<Issue>> = secrets
      .iter()
      .map(|secret_tuple| has_issues(secret_tuple, Some(expiration_days)).unwrap_or_default())
      .collect_vec();
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, context);
    for (secret_with_metadata, issues) in secrets.into_iter().zip(secrets_issues) {
      formatter.push_target_id_value_owned(secret_with_metadata.name.clone(), (secret_with_metadata, Some(expiration_days), issues));
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct SecretListAllocationStatus {}

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

pub(crate) static CERTIFICATE_SECRET_LABELS_LIST: [CertificateSecretLabel; 4] =
  [CertificateSecretLabel::SecretName, CertificateSecretLabel::SubjectCommonName, CertificateSecretLabel::Issuer, CertificateSecretLabel::NotAfter];

pub(crate) struct SecretListCertificates {}

#[async_trait]
impl CommandExecutor for SecretListCertificates {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that contain certificates");
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&CERTIFICATE_SECRET_LABELS_LIST, context);
    for SecretWithMetadata { name, metadata, .. } in secrets_with_metadata {
      if let SecretMetadata::Certificate { parts, .. } = &metadata {
        if let Some(certificate_secret_metadata) = parts.first() {
          formatter.push_target_id_value_owned(name.clone(), (certificate_secret_metadata.clone(), Some(expiration_days)));
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

pub(crate) struct SecretListErrors {}

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

pub(crate) struct SecretListIds {}

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

pub(crate) struct SecretListIssues {}

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

async fn list_issues(client: &DshApiClient, matches: &ArgMatches, context: &Context, only_errors: bool) -> DshCliResult<DshCliResult<()>> {
  let expiration_days = get_expiration_days(matches, context.settings())?;
  let start_instant = context.now();
  let secrets: Vec<SecretWithMetadata> = secrets_with_metadata(client).await?;
  context.print_execution_time(start_instant);
  let secrets_issues: Vec<(String, SecretMetadata, Vec<Issue>)> = secrets
    .iter()
    .flat_map(|secret| {
      has_issues(secret, Some(expiration_days)).map(|issues| {
        (
          secret.name.clone(),
          secret.metadata.clone(),
          issues.into_iter().filter(|issue| !only_errors || issue.severity() == Severity::Error).collect_vec(),
        )
      })
    })
    .collect_vec();
  let mut formatter = ListFormatter::new_override_target_id_label(&SECRET_LIST_ISSUES_LABELS_LIST, "secret id", context);
  for (secret_name, secret_metadata, issues) in secrets_issues.into_iter() {
    for issue in issues {
      formatter.push_target_id_value_owned(secret_name.clone(), SecretMetadataIssue { metadata: secret_metadata.clone(), issue });
    }
  }
  formatter.print(None)?;
  Ok(Ok(()))
}

static KEY_LABELS_LIST: [SecretLabel; 4] = [SecretLabel::SecretName, SecretLabel::Size, SecretLabel::Kind, SecretLabel::Label];

pub(crate) struct SecretListKeys {}

#[async_trait]
impl CommandExecutor for SecretListKeys {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all secrets that contain private of public keys");
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&KEY_LABELS_LIST, context);
    for SecretWithMetadata { name, metadata, .. } in secrets_with_metadata {
      if let SecretMetadata::Pki { .. } = &metadata {
        formatter.push_target_id_value_owned(name.clone(), metadata);
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static SYSTEM_LABELS_LIST: [SecretLabel; 4] = [SecretLabel::SecretName, SecretLabel::SecretId, SecretLabel::Format, SecretLabel::Size];

pub(crate) struct SecretListSystem {}

#[async_trait]
impl CommandExecutor for SecretListSystem {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all system secret ids");
    let start_instant = context.now();
    let secrets_with_metadata = secrets_with_metadata(client).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&SYSTEM_LABELS_LIST, context);
    for secret_with_metadata in secrets_with_metadata {
      if secret_with_metadata.id.is_some() {
        formatter.push_target_id_value_owned(secret_with_metadata.name.clone(), secret_with_metadata);
      }
    }
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct SecretListUsage {}

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

static SECRET_LABELS_SHOW: [SecretLabel; 13] = [
  SecretLabel::SecretName,
  SecretLabel::SecretId,
  SecretLabel::System,
  SecretLabel::Kind,
  SecretLabel::Format,
  SecretLabel::Size,
  SecretLabel::Bytes,
  SecretLabel::Description,
  SecretLabel::Provisioned,
  SecretLabel::Notifications,
  SecretLabel::DerivedFrom,
  SecretLabel::Issues,
  SecretLabel::Dependants,
];

pub(crate) static CERTIFICATE_LABELS_SHOW: [CertificateSecretLabel; 6] = [
  CertificateSecretLabel::SubjectCommonName,
  CertificateSecretLabel::SubjectDistinguishedName,
  CertificateSecretLabel::IssuerDistinguishedName,
  CertificateSecretLabel::NotBefore,
  CertificateSecretLabel::NotAfter,
  CertificateSecretLabel::SerialNumber,
];

static PKI_LABELS_SHOW: [PkiSecretLabel; 3] = [PkiSecretLabel::Algorithm, PkiSecretLabel::Labels, PkiSecretLabel::Private];

async fn get_secret_with_metadata(secret_id_name: String, client: &DshApiClient) -> SecretWithMetadata {
  let (name, id) = normalize_secret_name(secret_id_name);
  let (secret_value, allocation_status, dependants) = join!(client.get_secret(&name), client.get_secret_status(&name), client.secret_dependants(&name));
  match secret_value {
    Ok(value) => {
      SecretWithMetadata { name, id, metadata: SecretMetadata::from(value.as_str()), allocation_status: allocation_status.ok(), dependants: dependants.unwrap_or_default() }
    }
    Err(error) => SecretWithMetadata {
      name,
      id,
      metadata: SecretMetadata::Error { message: error.to_string() },
      allocation_status: allocation_status.ok(),
      dependants: dependants.unwrap_or_default(),
    },
  }
}

pub(crate) struct SecretShow {}

#[async_trait]
impl CommandExecutor for SecretShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let target = target.unwrap_or_else(|| unreachable!());
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let start_instant = context.now();
    let secret_with_metadata = get_secret_with_metadata(target, client).await;
    context.print_execution_time(start_instant);
    context.print_allocation_status(
      &secret_with_metadata
        .allocation_status
        .clone()
        .ok_or_else(|| DshApiError::Unexpected { message: "allocation status not available".to_string(), cause: None }),
      SECRET_SUBJECT_TARGET,
    );
    let issues = has_issues(&secret_with_metadata, Some(expiration_days));
    UnitFormatter::new(&secret_with_metadata.name, &SECRET_LABELS_SHOW, context).print(&(&secret_with_metadata, Some(expiration_days), &issues.unwrap_or_default()), None)?;

    match &secret_with_metadata.metadata {
      SecretMetadata::Certificate { parts, .. } => {
        for part in parts {
          UnitFormatter::new(&part.subject_common_name, &CERTIFICATE_LABELS_SHOW, context).print(&(part, Some(expiration_days)), None)?;
        }
      }
      SecretMetadata::Empty => context.print_warning("secret is empty"),
      SecretMetadata::Error { message } => context.print_warning(format!("secret has an error ({})", message)),
      SecretMetadata::Misconfiguration { message } => context.print_warning(format!("secret is misconfigured ({})", message)),
      SecretMetadata::NotFound { message } => match message {
        Some(message) => context.print_warning(format!("secret could not be found ({})", message)),
        None => context.print_warning("secret could not be found"),
      },
      SecretMetadata::Pki { metadata, .. } => {
        UnitFormatter::new(&secret_with_metadata.name, &PKI_LABELS_SHOW, context).print(&metadata, None)?;
      }
      _ => {}
    }

    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) struct SecretShowAllocationStatus {}

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

pub(crate) struct SecretShowUsage {}

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

pub(crate) struct SecretShowValue {}

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

pub(crate) struct SecretUpdate {}

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

pub(crate) async fn print_certificate_secret(secret_name: &str, expiration_days: u64, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
  let secret_value = client.get_secret(&secret_name).await?;
  let secret_metadata = SecretMetadata::from(secret_value.as_str());
  if let SecretMetadata::Certificate { parts, .. } = &secret_metadata.clone() {
    for part in parts {
      UnitFormatter::new(&part.subject_common_name, &CERTIFICATE_LABELS_SHOW, context).print(&(part, Some(expiration_days)), None)?;
    }
  } else {
    context.print_error(format!("secret '{}' does not contain a certificate", secret_name))
  }
  Ok(())
}

pub(crate) async fn print_key_secret(secret_name: &str, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
  let secret_value = client.get_secret(&secret_name).await?;
  let secret_metadata = SecretMetadata::from(secret_value.as_str());
  if let SecretMetadata::Pki { metadata, .. } = &secret_metadata.clone() {
    UnitFormatter::new(secret_name, &PKI_LABELS_SHOW, context).print(&metadata, None)?;
  } else {
    context.print_error(format!("secret '{}' does not contain a key", secret_name))
  }
  Ok(())
}
