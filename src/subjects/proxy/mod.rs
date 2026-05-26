pub(crate) mod labels;

use crate::arguments::proxy_id_argument;
use crate::capability::{Capability, CommandExecutor, DEPLOY_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS, UNDEPLOY_COMMAND, UPDATE_COMMAND};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::directory::{proxy_certificate_bundle_exists, read_proxy_certificate_bundle};
use crate::error::DshCliError;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::global_options::{expiration_option, get_expiration_days};
use crate::secret_metadata::{secret_metadata, SecretMetadata};
use crate::subject::{Requirements, Subject};
use crate::subjects::certificate::{CertificateLabel, CERTIFICATE_LABELS_SHOW};
use crate::subjects::proxy::labels::KafkaProxyLabel;
use crate::subjects::secret::SecretLabel;
use crate::subjects::service::{cpus_option, instances_option, mem_option, CPUS_OPTION, INSTANCES_OPTION, MEM_OPTION};
use crate::target_platform::get_target_platform;
use crate::target_tenant::get_target_tenant;
use crate::{cli_error, err, DshCliResult, COMMAND_OPTIONS_HEADING};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::platform::VhostZone;
use dsh_api::types::{Certificate, KafkaProxy, KafkaProxyZone, Secret, Validations};
use futures::future::try_join_all;
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::convert::AsRef;
use std::num::NonZeroU64;
use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::time::sleep;

struct ProxySubject {}

const PROXY_SUBJECT_TARGET: &str = "proxy";

lazy_static! {
  pub(crate) static ref PROXY_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ProxySubject {});
}

#[async_trait]
impl Subject for ProxySubject {
  fn subject(&self) -> &'static str {
    PROXY_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list DSH Kafka proxies.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list Kafka proxies used by the services and apps on the DSH.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DEPLOY_COMMAND => Some(PROXY_DEPLOY_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(PROXY_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(PROXY_SHOW_CAPABILITY.as_ref()),
      UNDEPLOY_COMMAND => Some(PROXY_UNDEPLOY_CAPABILITY.as_ref()),
      UPDATE_COMMAND => Some(PROXY_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &PROXY_CAPABILITIES
  }
}

static PROXY_DEPLOY_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DEPLOY_COMMAND, None, &ProxyDeploy {}, "Deploy local proxy on dsh")
      .set_long_about("Deploy a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true))
      .add_extra_argument(cpus_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(instances_option().help_heading(COMMAND_OPTIONS_HEADING))
      .add_extra_argument(mem_option().help_heading(COMMAND_OPTIONS_HEADING)),
  )
});
static PROXY_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ProxyList {}, "List dsh proxies")
      .set_long_about("Lists all Kafka proxies used by the services and apps on the DSH.")
      .add_command_executor(FlagType::Ids, &ProxyListIds {}, None),
  )
});
static PROXY_SHOW_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &ProxyShow {}, "Show Kafka proxy configuration")
      .add_target_argument(proxy_id_argument().required(true))
      .add_extra_argument(expiration_option()),
  )
});
static PROXY_UNDEPLOY_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UNDEPLOY_COMMAND, None, &ProxyUndeploy {}, "Undeploy proxy from dsh")
      .set_long_about("Undeploy a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true)),
  )
});
static PROXY_UPDATE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &ProxyUpdate {}, "Update proxy on dsh")
      .set_long_about("Update an existing Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true)),
  )
});

static PROXY_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![PROXY_DEPLOY_CAPABILITY.as_ref(), PROXY_LIST_CAPABILITY.as_ref(), PROXY_SHOW_CAPABILITY.as_ref(), PROXY_UNDEPLOY_CAPABILITY.as_ref(), PROXY_UPDATE_CAPABILITY.as_ref()]
});

static GENERATED_CERTIFICATE_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

static SECRET_LABELS_SHOW: [SecretLabel; 10] = [
  SecretLabel::SecretName,
  SecretLabel::Kind,
  SecretLabel::FormatKind,
  SecretLabel::Size,
  SecretLabel::Description,
  SecretLabel::NotBefore,
  SecretLabel::NotAfter,
  SecretLabel::Subject,
  SecretLabel::Issuer,
  SecretLabel::SerialNumber,
];

static PROXY_LABELS_SHOW: [KafkaProxyLabel; 11] = [
  KafkaProxyLabel::Target,
  KafkaProxyLabel::Name,
  KafkaProxyLabel::Zone,
  KafkaProxyLabel::Certificate,
  KafkaProxyLabel::CaChainSecretName,
  KafkaProxyLabel::Cpus,
  KafkaProxyLabel::Mem,
  KafkaProxyLabel::Instances,
  KafkaProxyLabel::SchemaStore,
  KafkaProxyLabel::AclGroupsEnabled,
  KafkaProxyLabel::Validations,
];

struct ProxyDeploy {}

#[async_trait]
impl CommandExecutor for ProxyDeploy {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let proxy_bundle_id = target.unwrap_or_else(|| unreachable!());

    if client.get_kafkaproxy_configuration(&proxy_bundle_id).await.is_ok() {
      return err!("proxy '{}' already exists", proxy_bundle_id);
    }
    if !proxy_certificate_bundle_exists(&platform, &tenant, &proxy_bundle_id)? {
      return err!("proxy certificate bundle '{}' does not exist", proxy_bundle_id);
    }

    let proxy_certificate_name = format!("{}-certificate", proxy_bundle_id);
    let proxy_ca_certificate_secret_name = format!("{}-ca-certificate", proxy_bundle_id);
    let proxy_private_key_secret_name = format!("{}-private-key", proxy_bundle_id);
    let proxy_server_certificate_secret_name = format!("{}-server-certificate", proxy_bundle_id);

    let (proxy_certificate_result, ca_secret_result, key_secret_result, cert_secret_result) = join!(
      client.get_certificate(&proxy_certificate_name),
      client.get_secret(&proxy_ca_certificate_secret_name),
      client.get_secret(&proxy_private_key_secret_name),
      client.get_secret(&proxy_server_certificate_secret_name),
    );
    if proxy_certificate_result.is_ok() {
      context.print_error(format!("certificate '{}' already exists", proxy_certificate_name));
    }
    if ca_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", proxy_ca_certificate_secret_name));
    }
    if key_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", proxy_private_key_secret_name));
    }
    if cert_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", proxy_server_certificate_secret_name));
    }
    if proxy_certificate_result.is_ok() || ca_secret_result.is_ok() || key_secret_result.is_ok() || cert_secret_result.is_ok() {
      return err!("cancelled, some resources already exist");
    }

    let (proxy_server_certificate, proxy_private_key, proxy_ca_certificate, configuration) = read_proxy_certificate_bundle(&platform, &tenant, &proxy_bundle_id)?;

    let proxy_cpus = match matches.get_one::<f64>(CPUS_OPTION) {
      Some(cpus) => *cpus,
      None => f64::from_str(&context.read_single_line_with_default("proxy cpus", "0.1")?)?,
    };

    let proxy_mem = match matches.get_one::<u64>(MEM_OPTION) {
      Some(mem) => *mem,
      None => u64::from_str(&context.read_single_line_with_default("proxy memory", "256")?)?,
    };

    let proxy_instances = NonZeroU64::new(match matches.get_one::<u64>(INSTANCES_OPTION) {
      Some(instances) => *instances,
      None => u64::from_str(&context.read_single_line_with_default("proxy instances", "1")?)?,
    })
    .ok_or(cli_error!("number of instances cannot be zero"))?;

    let kafka_proxy = KafkaProxy {
      name: Some(proxy_bundle_id.clone()),
      secret_name_ca_chain: proxy_ca_certificate_secret_name.clone(),
      certificate: proxy_certificate_name.clone(),
      cpus: proxy_cpus,
      mem: proxy_mem as i64,
      instances: proxy_instances,
      enable_kafka_acl_groups: Some(configuration.acl_group_name.is_some()),
      validations: vec![Validations { common_name: None, country: None, locality: None, organization: None, organizational_unit: None, province: None, subject_type: None }],
      schema_store: Some(configuration.enable_schema_store),
      schema_store_cpus: Some(0.1),
      schema_store_mem: Some(256),
      zone: match configuration.vhost_zone {
        VhostZone::Private => KafkaProxyZone::Private,
        VhostZone::Public => KafkaProxyZone::Public,
      },
    };

    let certificate_body =
      Certificate { cert_chain_secret: proxy_server_certificate_secret_name.clone(), key_secret: proxy_private_key_secret_name.clone(), passphrase_secret: None };

    UnitFormatter::new(&proxy_bundle_id, &PROXY_LABELS_SHOW, context).print(&kafka_proxy, None)?;
    UnitFormatter::new(&proxy_certificate_name, &GENERATED_CERTIFICATE_LABELS, context).print(&certificate_body, None)?;
    UnitFormatter::new(&proxy_ca_certificate_secret_name, &SECRET_LABELS_SHOW, context).print(&(secret_metadata(&proxy_ca_certificate), None), None)?;
    UnitFormatter::new(&proxy_server_certificate_secret_name, &SECRET_LABELS_SHOW, context).print(&(secret_metadata(&proxy_server_certificate), None), None)?;
    UnitFormatter::new(&proxy_private_key_secret_name, &SECRET_LABELS_SHOW, context).print(&(secret_metadata(&proxy_private_key), None), None)?;

    if !context.confirmed(format!("deploy proxy '{}'?", proxy_bundle_id))? {
      return err!("cancelled, proxy '{}' not deployed", proxy_bundle_id);
    }

    if context.dry_run() {
      context.print_warning("dry-run mode, proxy not deployed");
    } else {
      let ca_certificate_secret = Secret::new(&proxy_ca_certificate_secret_name, &proxy_ca_certificate);
      let private_key_secret = Secret::new(&proxy_private_key_secret_name, &proxy_private_key);
      let server_certificate_secret = Secret::new(&proxy_server_certificate_secret_name, &proxy_server_certificate);
      let (cert_secret_result, key_secret_result, ca_secret_result) = join!(
        client.post_secret(&server_certificate_secret),
        client.post_secret(&private_key_secret),
        client.post_secret(&ca_certificate_secret),
      );
      context.print_outcome(format!("server certificate secret '{}' created", &server_certificate_secret));
      context.print_outcome(format!("private key secret '{}' created", &private_key_secret));
      context.print_outcome(format!("ca certificate secret '{}' created", &ca_certificate_secret));

      if let Err(error) = cert_secret_result {
        context.print_error(format!("error writing certificate secret '{}' ({})", proxy_server_certificate_secret_name, error));
      }
      if let Err(error) = key_secret_result {
        context.print_error(format!("error writing key secret '{}' ({})", proxy_private_key_secret_name, error));
      }
      if let Err(error) = ca_secret_result {
        context.print_error(format!("error writing ca certificate secret '{}' ({})", proxy_ca_certificate_secret_name, error));
      }

      client
        .put_certificate_configuration(&proxy_certificate_name, &certificate_body)
        .await
        .map_err(|error| cli_error!("error writing certificate configuration '{}' ({})", proxy_certificate_name, error))?;
      context.print_outcome(format!("certificate '{}' created", &proxy_certificate_name));

      client
        .put_kafkaproxy_configuration(&proxy_bundle_id, &kafka_proxy)
        .await
        .map_err(|error| cli_error!("error writing proxy configuration '{}' ({})", proxy_bundle_id, error))?;

      context.print_outcome(format!("proxy '{}' deployed", proxy_bundle_id));
    }

    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static PROXY_LABELS_LIST: [KafkaProxyLabel; 7] = [
  KafkaProxyLabel::Target,
  KafkaProxyLabel::Certificate,
  KafkaProxyLabel::Cpus,
  KafkaProxyLabel::Mem,
  KafkaProxyLabel::Zone,
  KafkaProxyLabel::SchemaStore,
  KafkaProxyLabel::AclGroupsEnabled,
];

struct ProxyList {}

#[async_trait]
impl CommandExecutor for ProxyList {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all proxies with parameters");
    let start_instant = context.now();
    let proxy_ids = client.get_kafkaproxy_ids().await?;
    let proxys = try_join_all(proxy_ids.iter().map(|proxy_id| client.get_kafkaproxy_configuration(proxy_id))).await?;
    context.print_execution_time(start_instant);
    let mut formatter = ListFormatter::new(&PROXY_LABELS_LIST, context);
    formatter.push_target_ids_and_values(proxy_ids.as_slice(), proxys.as_slice());
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyListIds {}

#[async_trait]
impl CommandExecutor for ProxyListIds {
  async fn execute_with_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all proxy ids");
    let start_instant = context.now();
    let proxy_ids = client.get_kafkaproxy_ids().await?;
    context.print_execution_time(start_instant);
    let mut formatter = IdsFormatter::new("proxy id", context);
    formatter.push_target_ids(&proxy_ids);
    formatter.print(Some(OutputFormat::Plain))?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

pub(crate) static SECRET_LABELS_LIST: [SecretLabel; 6] =
  [SecretLabel::SecretName, SecretLabel::Kind, SecretLabel::FormatKind, SecretLabel::Size, SecretLabel::Description, SecretLabel::NotAfter];

struct ProxyShow {}

#[async_trait]
impl CommandExecutor for ProxyShow {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show configuration of proxy '{}'", proxy_id));
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let (proxy, allocation_status) = join!(client.get_kafkaproxy_configuration(&proxy_id), client.get_kafkaproxy_status(&proxy_id));
    match proxy {
      Ok(proxy) => {
        let certificate_status = client.get_certificate(&proxy.certificate).await?;
        context.print_allocation_status(&allocation_status, PROXY_SUBJECT_TARGET);
        UnitFormatter::new(proxy_id, &PROXY_LABELS_SHOW, context).print(&proxy, None)?;
        let mut secret_names: Vec<String> = vec![proxy.secret_name_ca_chain];
        if let Some(actual_certificate) = &certificate_status.actual {
          UnitFormatter::new(proxy.certificate, &CERTIFICATE_LABELS_SHOW, context).print(&(actual_certificate, Some(expiration_days)), None)?;
          secret_names.push(actual_certificate.key_secret.clone());
          secret_names.push(actual_certificate.cert_chain_secret.clone());
          if let Some(passphrase_secret) = &actual_certificate.passphrase_secret {
            secret_names.push(passphrase_secret.clone());
          }
        }
        secret_names.sort();
        let secrets = try_join_all(secret_names.iter().map(|secret_id| client.secret_with_status(secret_id))).await?;
        let mut formatter = ListFormatter::new(&SECRET_LABELS_LIST, context);
        for (secret_name, (secret_value, _)) in secret_names.iter().zip(&secrets) {
          formatter.push_target_id_value_owned(secret_name.clone(), (secret_metadata(secret_value), Some(expiration_days)));
        }
        formatter.print(None)?;
        for (secret_name, (secret_value, allocation_status)) in secret_names.iter().zip(secrets) {
          let secret_metadata = secret_metadata(&secret_value);
          match secret_metadata {
            SecretMetadata::Certificate { .. } | SecretMetadata::Pki { .. } => {
              UnitFormatter::new(secret_name, &SECRET_LABELS_SHOW, context).print(&(None, secret_metadata, Some(expiration_days), Some(allocation_status)), None)?
            }
            _ => {}
          }
        }
        Ok(())
      }
      Err(error) => {
        context.print_allocation_status(&allocation_status, PROXY_SUBJECT_TARGET);
        Err(DshCliError::from(error))
      }
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyUndeploy {}

#[async_trait]
impl CommandExecutor for ProxyUndeploy {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    let kafka_proxy = client
      .get_kafkaproxy_configuration(&proxy_id)
      .await
      .map_err(|_| DshCliError::from(format!("proxy '{}' does not exists", proxy_id)))?;
    let certificate = client.get_certificate_configuration(&kafka_proxy.certificate).await?;
    let certificate_secrets = match certificate.passphrase_secret {
      Some(passphrase_secret) => vec![certificate.cert_chain_secret, certificate.key_secret, kafka_proxy.secret_name_ca_chain, passphrase_secret],
      None => vec![certificate.cert_chain_secret, certificate.key_secret, kafka_proxy.secret_name_ca_chain],
    };
    if context.confirmed(format!("undeploy proxy '{}'?", proxy_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, proxy not undeployed");
      } else {
        client.delete_kafkaproxy_configuration(&proxy_id).await?;
        context.print_outcome(format!("proxy '{}' undeployed", proxy_id));
        if context.confirmed(format!("delete certificate '{}'?", &kafka_proxy.certificate))? {
          if context.dry_run() {
            context.print_warning("dry-run mode, certificate not deleted");
          } else {
            // Wait until the proxy is gone
            loop {
              context.print_progress_step();
              sleep(Duration::from_millis(1000)).await;
              match client.get_kafkaproxy_configuration(&proxy_id).await {
                Ok(_) => {}
                Err(_) => break,
              }
            }
            context.print_error("");
            client.delete_certificate_configuration(&kafka_proxy.certificate).await?;
            context.print_outcome(format!("certificate '{}' deleted", &kafka_proxy.certificate));
            if context.confirmed(format!(
              "delete secrets {}?",
              certificate_secrets.iter().map(|secret| format!("'{}'", secret)).join(", ")
            ))? {
              if context.dry_run() {
                context.print_warning("dry-run mode, secrets not deleted");
              } else {
                // Wait until the certificate is gone
                loop {
                  context.print_progress_step();
                  sleep(Duration::from_millis(1000)).await;
                  match client.get_certificate_configuration(&kafka_proxy.certificate).await {
                    Ok(_) => {}
                    Err(_) => break,
                  }
                }
                context.print_error("");
                for secret in certificate_secrets {
                  client.delete_secret_configuration(&secret).await?;
                  context.print_outcome(format!("secret '{}' deleted", &secret));
                }
              }
            }
          }
        }
      }
    } else {
      context.print_outcome(format!("cancelled, proxy '{}' not undeployed", proxy_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyUpdate {}

#[async_trait]
impl CommandExecutor for ProxyUpdate {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    if client.get_kafkaproxy_configuration(&proxy_id).await.is_err() {
      err!("proxy '{}' does not exists", proxy_id)
    } else if context.confirmed(format!("update proxy '{}'?", proxy_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, proxy not updated");
        Ok(())
      } else {
        err!("capability not yet implemented, proxy not updated")
      }
    } else {
      context.print_outcome(format!("cancelled, proxy '{}' not updated", proxy_id));
      Ok(())
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}
