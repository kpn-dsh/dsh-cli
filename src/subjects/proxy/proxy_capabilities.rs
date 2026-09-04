use crate::capability::CommandExecutor;
use crate::context::Context;
use crate::directory::{certificate_bundle_exists, read_proxy_certificate_bundle, BundleKind};
use crate::error::DshCliError;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::OutputFormat;
use crate::global_options::get_expiration_days;
use crate::secret_metadata::SecretMetadata;
use crate::subject::Requirements;
use crate::subjects::certificate::capabilities::CERTIFICATE_LABELS_SHOW;
use crate::subjects::certificate::labels::CertificateLabel;
use crate::subjects::proxy::labels::KafkaProxyLabel;
use crate::subjects::proxy::PROXY_SUBJECT_TARGET;
use crate::subjects::secret::capabilities::{print_certificate_secret, print_key_secret};
use crate::subjects::secret::labels::SecretLabel;
use crate::subjects::service::capabilities::{CPUS_OPTION, INSTANCES_OPTION, MEM_OPTION};
use crate::target_platform::get_target_platform;
use crate::target_tenant::get_target_tenant;
use crate::{cli_error, err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::platform::VhostZone;
use dsh_api::types::{Certificate, KafkaProxy, KafkaProxyZone, Secret, Validations};
use futures::future::try_join_all;
use futures::join;
use itertools::Itertools;
use std::num::NonZeroU64;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

static GENERATED_CERTIFICATE_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

static SECRET_LABELS_SHOW: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Kind, SecretLabel::Format, SecretLabel::Size, SecretLabel::Description];

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

pub(crate) struct ProxyDeploy {}

#[async_trait]
impl CommandExecutor for ProxyDeploy {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let proxy_bundle_id = target.unwrap_or_else(|| unreachable!());

    if client.get_kafkaproxy_configuration(&proxy_bundle_id).await.is_ok() {
      return err!("proxy '{}' already exists", proxy_bundle_id);
    }
    if !certificate_bundle_exists(&platform, &tenant, BundleKind::Proxy, &proxy_bundle_id)? {
      return err!("proxy certificate bundle '{}' does not exist", proxy_bundle_id);
    }

    let proxy_certificate_name = format!("{}-cert", proxy_bundle_id);
    let proxy_ca_certificate_secret_name = format!("{}-ca-cert", proxy_bundle_id);
    let proxy_private_key_secret_name = format!("{}-key", proxy_bundle_id);
    let proxy_server_certificate_secret_name = format!("{}-server-cert", proxy_bundle_id);

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
    UnitFormatter::new(&proxy_ca_certificate_secret_name, &SECRET_LABELS_SHOW, context).print(&SecretMetadata::from(proxy_ca_certificate.as_str()), None)?;
    UnitFormatter::new(&proxy_server_certificate_secret_name, &SECRET_LABELS_SHOW, context).print(&SecretMetadata::from(proxy_server_certificate.as_str()), None)?;
    UnitFormatter::new(&proxy_private_key_secret_name, &SECRET_LABELS_SHOW, context).print(&SecretMetadata::from(proxy_private_key.as_str()), None)?;

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

pub(crate) struct ProxyList {}

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

pub(crate) struct ProxyListIds {}

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

pub(crate) struct ProxyShow {}

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

        if let Some(actual_certificate) = &certificate_status.actual {
          let validated_dns = actual_certificate
            .dns_names
            .first()
            .and_then(|first_dns| client.platform().validate_vhost_domain(first_dns).ok());
          context.print_explanation(format!("certificate '{}'", proxy.certificate));
          UnitFormatter::new(proxy.certificate.clone(), &CERTIFICATE_LABELS_SHOW, context).print(&(actual_certificate, Some(expiration_days), validated_dns), None)?;
        }

        if let Some(actual_certificate) = &certificate_status.actual {
          context.print_explanation(format!("ca certificate secret '{}'", proxy.secret_name_ca_chain));
          print_certificate_secret(proxy.secret_name_ca_chain.as_str(), expiration_days, client, context).await?;

          context.print_explanation(format!("cert chain secret '{}'", actual_certificate.cert_chain_secret));
          print_certificate_secret(actual_certificate.cert_chain_secret.as_str(), expiration_days, client, context).await?;

          context.print_explanation(format!("key secret '{}'", actual_certificate.key_secret));
          print_key_secret(actual_certificate.key_secret.as_str(), client, context).await?;
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

pub(crate) struct ProxyUndeploy {}

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
