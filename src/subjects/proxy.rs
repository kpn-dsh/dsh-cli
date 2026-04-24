use crate::arguments::proxy_id_argument;
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, DELETE_COMMAND, DEPLOY_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::directory::{list_proxy_certificate_bundles, proxy_certificate_bundle_exists, read_proxy_certificate_bundle, store_proxy_certificate_bundle};
use crate::error::DshCliError;
use crate::flags::FlagType;
use crate::formatters::ids_formatter::IdsFormatter;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::{Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::global_options::{expiration_option, get_expiration_days};
use crate::proxy_bundles::{ProxyCertificateBundle, ProxyCertificateBundleConfig};
use crate::secret_metadata::{secret_metadata, SecretMetadata};
use crate::subject::{Requirements, Subject};
use crate::subjects::certificate::{CertificateLabel, CERTIFICATE_LABELS_SHOW};
use crate::subjects::secret::SecretLabel;
use crate::target_platform::{get_target_platform, platform_name_argument};
use crate::target_tenant::{get_target_tenant, tenant_name_argument};
use crate::verbosity::Verbosity;
use crate::{cli_error, err, DshCliResult};
use async_trait::async_trait;
use clap::builder::PossibleValue;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::platform::VhostZone;
use dsh_api::types::{Certificate, KafkaProxy, KafkaProxyZone, Secret};
use futures::future::try_join_all;
use futures::join;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::trace;
use serde::Serialize;
use std::convert::AsRef;
use std::num::NonZeroU64;
use std::str::FromStr;
use std::sync::LazyLock;
use whoami::username;

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
      CREATE_COMMAND => Some(PROXY_CERTIFICATE_BUNDLE_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(PROXY_DELETE_CAPABILITY.as_ref()),
      DEPLOY_COMMAND => Some(PROXY_DEPLOY_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(PROXY_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(PROXY_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &PROXY_CAPABILITIES
  }
}

static PROXY_CERTIFICATE_BUNDLE_CREATE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, None, &ProxyCertificateBundleCreate {}, "Create proxy certificates bundle")
      .add_target_argument(proxy_id_argument().required(true))
      .add_target_argument(platform_name_argument())
      .add_target_argument(tenant_name_argument())
      .add_extra_argument(broker_prefix_option())
      .add_extra_argument(number_of_dns_records_option())
      .add_extra_argument(ca_common_name_option())
      .add_extra_argument(vhost_zone_option())
      .add_extra_argument(include_schema_store_dns_option()),
  )
});
static PROXY_DELETE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &ProxyDelete {}, "Delete proxy")
      .set_long_about("Delete a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true)),
  )
});
static PROXY_DEPLOY_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DEPLOY_COMMAND, None, &ProxyDeploy {}, "Deploy proxy")
      .set_long_about("Deploy a Kafka proxy.")
      .add_target_argument(proxy_id_argument().required(true)),
  )
});
static PROXY_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &ProxyList {}, "List proxies")
      .set_long_about("Lists all Kafka proxies used by the services and apps on the DSH.")
      .add_command_executor(FlagType::Bundles, &ProxyListBundles {}, None)
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

static PROXY_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
    PROXY_CERTIFICATE_BUNDLE_CREATE_CAPABILITY.as_ref(),
    PROXY_DELETE_CAPABILITY.as_ref(),
    PROXY_DEPLOY_CAPABILITY.as_ref(),
    PROXY_LIST_CAPABILITY.as_ref(),
    PROXY_SHOW_CAPABILITY.as_ref(),
  ]
});

const BROKER_PREFIX_OPTION: &str = "broker-prefix-option";

fn broker_prefix_option() -> Arg {
  Arg::new(BROKER_PREFIX_OPTION)
    .long("broker-prefix")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("BROKER_PREFIX")
    .help("Broker prefix")
    .long_help("Prefix used to generate the dsn and schema store name.")
}

const NUMBER_OF_DNS_RECORDS_OPTION: &str = "number-of-dns-records-option";

fn number_of_dns_records_option() -> Arg {
  Arg::new(NUMBER_OF_DNS_RECORDS_OPTION)
    .long("number-of-dns-records")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<usize>::new().range(1..11))
    .value_name("NUMBER_OF_DNS_RECORDS")
    .help("Number of dns records")
    .long_help(
      "Number of broker dns records that will be generated. Do not use this \
         option unless you know what you are doing.",
    )
}

const CA_COMMON_NAME_OPTION: &str = "ca-common-name-option";

fn ca_common_name_option() -> Arg {
  Arg::new(CA_COMMON_NAME_OPTION)
    .long("ca-common-name")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .help("Certificate authority common name")
    .long_help("This option specifies the common name used to create certificate authority certificate.")
}

const VHOST_ZONE_OPTION: &str = "vhost-zone-option";

fn vhost_zone_option() -> Arg {
  let possible_values = [PossibleValue::new("private").help("Private vhost"), PossibleValue::new("public").help("Public vhost")];
  Arg::new(VHOST_ZONE_OPTION)
    .long("vhost-zone")
    .action(ArgAction::Set)
    .value_parser(possible_values)
    .help("Vhost zone")
    .long_help("This option indicates whether the certificates will be created for a public or a private vhost.")
}

const INCLUDE_SCHEMA_STORE_DNS_OPTION: &str = "include-schema-store-dns-option";

fn include_schema_store_dns_option() -> Arg {
  Arg::new(INCLUDE_SCHEMA_STORE_DNS_OPTION)
    .long("include-schema-store-dns")
    .action(ArgAction::SetTrue)
    .help("Include schema store dns")
    .long_help(
      "If this option is provided the created certificates will include a dns entry \
    for a schema store.",
    )
}

static GENERATED_CERTIFICATE_LABELS: [CertificateLabel; 6] = [
  CertificateLabel::Target,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::SerialNumber,
];
static PROXY_BUNDLE_LABELS_CREATE: [ProxyBundleLabel; 8] = [
  ProxyBundleLabel::Platform,
  ProxyBundleLabel::Tenant,
  ProxyBundleLabel::BundleName,
  ProxyBundleLabel::BrokerPrefix,
  ProxyBundleLabel::CaCommonName,
  ProxyBundleLabel::HasSchemaStoreDnsRecord,
  ProxyBundleLabel::VhostZone,
  ProxyBundleLabel::NumberOfDsnRecords,
];

struct ProxyCertificateBundleCreate {}

#[async_trait]
impl CommandExecutor for ProxyCertificateBundleCreate {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let proxy_bundle_id = target.unwrap_or_else(|| unreachable!());

    if proxy_certificate_bundle_exists(&platform, &tenant, &proxy_bundle_id)? {
      context.print_warning(format!(
        "proxy certificate bundle '{}' already exists for '{}@{}'",
        proxy_bundle_id, platform, tenant
      ));
      if !context.confirmed("do you want to override the existing bundle?")? {
        context.print_outcome("cancelled");
        return Ok(());
      }
    }

    context.print_explanation(format!("create proxy certificates bundle '{}' for '{}@{}'", proxy_bundle_id, platform, tenant));

    let broker_prefix: String = match matches.get_one::<String>(BROKER_PREFIX_OPTION) {
      Some(prefix) => prefix.clone(),
      None => {
        let prefix = context.read_single_line(format!("broker prefix [{}]", proxy_bundle_id))?;
        if prefix.is_empty() {
          proxy_bundle_id.clone()
        } else {
          prefix
        }
      }
    };
    let vhost_zone = match matches.get_one::<VhostZone>(VHOST_ZONE_OPTION) {
      Some(vhost_zone) => vhost_zone.clone(),
      None => {
        let vhost_zone_string = context.read_single_line("vhost zone [PRIVATE/public]")?;
        if vhost_zone_string.is_empty() {
          VhostZone::Private
        } else {
          VhostZone::from_str(&vhost_zone_string)?
        }
      }
    };

    let ca_common_name = match matches.get_one::<String>(CA_COMMON_NAME_OPTION) {
      Some(ca_common_name) => ca_common_name.to_string(),
      None => {
        let default_username = username()?;
        let ca_common_name = context.read_single_line(format!("certificate authority common name [{}]", default_username))?;
        if ca_common_name.is_empty() {
          default_username
        } else {
          ca_common_name
        }
      }
    };

    let include_schema_store_dns_record = matches.get_flag(INCLUDE_SCHEMA_STORE_DNS_OPTION);
    let number_of_dns_records = match matches.get_one::<usize>(NUMBER_OF_DNS_RECORDS_OPTION) {
      Some(number_of_dns_records) if *number_of_dns_records < 10 => {
        context.print_warning("the number of dns records should almost always be set to the default value of 10");
        if context.confirmed(format!("are you sure you want to set the number of dns records to {}?", number_of_dns_records))? {
          *number_of_dns_records
        } else {
          return err!("cancelled");
        }
      }
      _ => 10,
    };

    let config = ProxyCertificateBundleConfig {
      broker_prefix: broker_prefix.clone(),
      ca_common_name: ca_common_name.clone(),
      include_schema_store_dns_record,
      number_of_dns_records,
      platform: platform.clone(),
      tenant: tenant.clone(),
      vhost_zone: vhost_zone.clone(),
    };
    trace!("{:#?}", config);

    if !context.quiet() {
      match context.verbosity() {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium | Verbosity::High => UnitFormatter::new(&proxy_bundle_id, &PROXY_BUNDLE_LABELS_CREATE, context).print(&config, None)?,
      }
    }

    let cert_bundle = ProxyCertificateBundle::try_from(config)?;

    if !context.quiet() {
      match context.verbosity() {
        Verbosity::Off | Verbosity::Low => (),
        Verbosity::Medium | Verbosity::High => {
          UnitFormatter::new("ca certificate", &GENERATED_CERTIFICATE_LABELS, context).print_non_serializable(&cert_bundle.ca_certificate, None)?;
          UnitFormatter::new("client certificate", &GENERATED_CERTIFICATE_LABELS, context).print_non_serializable(&cert_bundle.client_certificate, None)?;
          UnitFormatter::new("server certificate", &GENERATED_CERTIFICATE_LABELS, context).print_non_serializable(&cert_bundle.server_certificate, None)?;
        }
      }
    }

    context.println(format!(
      "broker prefix: '{}', number of brokers: {}, {} vhost zone, schema dns entry {}included, ca common name: {}",
      broker_prefix,
      number_of_dns_records,
      vhost_zone,
      if include_schema_store_dns_record { "" } else { "not " },
      ca_common_name
    ));

    if context.dry_run() {
      context.print_warning("dry-run mode, proxy certificates bundle not stored");
    } else {
      let bundle_directory = store_proxy_certificate_bundle(&platform, &tenant, &proxy_bundle_id, &cert_bundle)?;
      context.println(format!(
        "proxy certificates bundle '{}' stored in directory '{}'",
        proxy_bundle_id, bundle_directory
      ));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct ProxyDelete {}

#[async_trait]
impl CommandExecutor for ProxyDelete {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, _: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let proxy_id = target.unwrap_or_else(|| unreachable!());
    if client.get_kafkaproxy_configuration(&proxy_id).await.is_err() {
      return err!("proxy '{}' does not exists", proxy_id);
    }
    if context.confirmed(format!("delete proxy '{}'?", proxy_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, proxy not deleted");
      } else {
        client.delete_kafkaproxy_configuration(&proxy_id).await?;
        context.print_outcome(format!("proxy '{}' deleted", proxy_id));
      }
    } else {
      context.print_outcome(format!("cancelled, proxy '{}' not deleted", proxy_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

struct ProxyDeploy {}

#[async_trait]
impl CommandExecutor for ProxyDeploy {
  async fn execute_with_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let proxy_id = target.unwrap_or_else(|| unreachable!());

    if !proxy_certificate_bundle_exists(&platform, &tenant, &proxy_id)? {
      return err!("proxy certificate bundle '{}' does not exist", proxy_id);
    }

    let (server_certificate, private_key, ca_certificate, configuration) = read_proxy_certificate_bundle(&platform, &tenant, &proxy_id)?;

    if client.get_kafkaproxy_configuration(&proxy_id).await.is_ok() {
      context.print_warning(format!("proxy '{}' already exists", proxy_id));
      if !context.confirmed(format!("replace proxy '{}'?", proxy_id))? {
        return err!("cancelled, proxy '{}' not deployed", proxy_id);
      }
    }

    let server_certificate_secret_name = format!("{}-certificate-cert", proxy_id);
    let private_key_secret_name = format!("{}-certificate-key", proxy_id);
    let ca_certificate_secret_name = format!("{}-certificate-ca", proxy_id);
    let certificate_name = format!("{}-certificate", proxy_id);

    let (cert_secret_result, key_secret_result, ca_secret_result, certificate_result) = join!(
      client.get_secret(&server_certificate_secret_name),
      client.get_secret(&private_key_secret_name),
      client.get_secret(&ca_certificate_secret_name),
      client.get_certificate(&certificate_name)
    );
    if cert_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", server_certificate_secret_name));
    }
    if key_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", private_key_secret_name));
    }
    if ca_secret_result.is_ok() {
      context.print_error(format!("secret '{}' already exists", ca_certificate_secret_name));
    }
    if certificate_result.is_ok() {
      context.print_error(format!("certificate '{}' already exists", certificate_name));
    }
    if cert_secret_result.is_ok() || key_secret_result.is_ok() || ca_secret_result.is_ok() || certificate_result.is_ok() {
      return err!("cancelled, some resources already exists");
    }

    let server_certificate_secret = Secret::new(&server_certificate_secret_name, server_certificate);
    let private_key_secret = Secret::new(&private_key_secret_name, private_key);
    let ca_certificate_secret = Secret::new(&ca_certificate_secret_name, ca_certificate);

    let name = Some(proxy_id.clone());
    let secret_name_ca_chain = ca_certificate_secret_name.clone();
    let certificate = certificate_name.clone();
    let cpus = 0.1;
    let mem = 256;
    let instances = NonZeroU64::new(1).unwrap();
    let enable_kafka_acl_groups = Some(false);
    let validations = vec![];
    let (schema_store, schema_store_cpus, schema_store_mem) = if configuration.include_schema_store_dns_record { (Some(true), Some(0.1), Some(256)) } else { (None, None, None) };
    let zone = match configuration.vhost_zone {
      VhostZone::Private => KafkaProxyZone::Private,
      VhostZone::Public => KafkaProxyZone::Public,
    };

    let certificate_body = Certificate { cert_chain_secret: server_certificate_secret_name.clone(), key_secret: private_key_secret_name.clone(), passphrase_secret: None };

    let kafka_proxy =
      KafkaProxy { name, secret_name_ca_chain, certificate, cpus, mem, instances, enable_kafka_acl_groups, validations, schema_store, schema_store_cpus, schema_store_mem, zone };

    UnitFormatter::new(&server_certificate_secret_name, &SECRET_LABELS_SHOW, context).print(&(secret_metadata(&server_certificate_secret.value), None), None)?;
    UnitFormatter::new(&private_key_secret_name, &SECRET_LABELS_SHOW, context).print(&(secret_metadata(&private_key_secret.value), None), None)?;
    UnitFormatter::new(&ca_certificate_secret_name, &SECRET_LABELS_SHOW, context).print(&(secret_metadata(&ca_certificate_secret.value), None), None)?;
    UnitFormatter::new(&ca_certificate_secret_name, &GENERATED_CERTIFICATE_LABELS, context).print(&certificate_body, None)?;
    UnitFormatter::new(&ca_certificate_secret_name, &PROXY_LABELS_SHOW, context).print(&kafka_proxy, None)?;

    if context.dry_run() {
      context.print_warning("dry-run mode, proxy not deployed");
    } else {
      let (cert_secret_result, key_secret_result, ca_secret_result) = join!(
        client.post_secret(&server_certificate_secret),
        client.post_secret(&private_key_secret),
        client.post_secret(&ca_certificate_secret),
      );
      if let Err(error) = cert_secret_result {
        context.print_error(format!("error writing certificate secret '{}' ({})", server_certificate_secret_name, error));
      }
      if let Err(error) = key_secret_result {
        context.print_error(format!("error writing key secret '{}' ({})", private_key_secret_name, error));
      }
      if let Err(error) = ca_secret_result {
        context.print_error(format!("error writing ca certificate secret '{}' ({})", ca_certificate_secret_name, error));
      }

      client
        .put_certificate_configuration(&certificate_name, &certificate_body)
        .await
        .map_err(|error| cli_error!("error writing certificate configuration '{}' ({})", certificate_name, error))?;

      client
        .put_kafkaproxy_configuration(&proxy_id, &kafka_proxy)
        .await
        .map_err(|error| cli_error!("error writing proxy configuration '{}' ({})", proxy_id, error))?;

      context.print_outcome(format!("proxy '{}' deployed", proxy_id));
    }

    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_with_api()
  }
}

static PROXY_LABELS_LIST: [ProxyLabel; 6] = [ProxyLabel::Target, ProxyLabel::Certificate, ProxyLabel::Cpus, ProxyLabel::Mem, ProxyLabel::Zone, ProxyLabel::SchemaStore];

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

static PROXY_BUNDLE_LABELS_LIST: [ProxyBundleLabel; 7] = [
  ProxyBundleLabel::BundleName,
  ProxyBundleLabel::BrokerPrefix,
  ProxyBundleLabel::CaCommonName,
  ProxyBundleLabel::HasSchemaStoreDnsRecord,
  ProxyBundleLabel::VhostZone,
  ProxyBundleLabel::NumberOfDsnRecords,
  ProxyBundleLabel::BundleDirectory,
];

struct ProxyListBundles {}

#[async_trait]
impl CommandExecutor for ProxyListBundles {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    context.print_explanation(format!("list all local proxy certificate bundles for '{}@{}'", platform.name(), tenant));
    let list: Vec<(String, (ProxyCertificateBundleConfig, String))> = list_proxy_certificate_bundles(&platform, &tenant)?
      .into_iter()
      .map(|(bundle_name, bundle_config, bundle_directory)| (bundle_name, (bundle_config, bundle_directory)))
      .collect_vec();
    let mut formatter = ListFormatter::new(&PROXY_BUNDLE_LABELS_LIST, context);
    formatter.push_target_id_value_pairs(&list);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
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

pub(crate) static SECRET_LABELS_LIST: [SecretLabel; 6] =
  [SecretLabel::SecretName, SecretLabel::Kind, SecretLabel::FormatKind, SecretLabel::Size, SecretLabel::Description, SecretLabel::Expires];

static SECRET_LABELS_SHOW: [SecretLabel; 13] = [
  SecretLabel::SecretName,
  SecretLabel::SecretId,
  SecretLabel::System,
  SecretLabel::Kind,
  SecretLabel::FormatKind,
  SecretLabel::Size,
  SecretLabel::Description,
  SecretLabel::Expires,
  SecretLabel::Provisioned,
  SecretLabel::Notifications,
  SecretLabel::DerivedFrom,
  SecretLabel::Subject,
  SecretLabel::Issuer,
];

#[derive(Eq, Hash, PartialEq, Serialize)]
enum ProxyLabel {
  AclGroupsEnabled,
  CaChainSecretName,
  Certificate,
  Cpus,
  Instances,
  Mem,
  Name,
  SchemaStore,
  Target,
  Validations,
  Zone,
}

impl Label for ProxyLabel {
  fn as_str(&self) -> &str {
    match self {
      ProxyLabel::AclGroupsEnabled => "acl groups",
      ProxyLabel::CaChainSecretName => "ca certificate",
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "cpus",
      ProxyLabel::Instances => "instances",
      ProxyLabel::Mem => "memory",
      ProxyLabel::Name => "name",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validations",
      ProxyLabel::Zone => "zone",
    }
  }

  fn as_str_for_list(&self) -> &str {
    match self {
      ProxyLabel::AclGroupsEnabled => "acl groups",
      ProxyLabel::CaChainSecretName => "ca certificate",
      ProxyLabel::Certificate => "certificate",
      ProxyLabel::Cpus => "cpus",
      ProxyLabel::Instances => "instances",
      ProxyLabel::Mem => "memory",
      ProxyLabel::Name => "proxy name",
      ProxyLabel::SchemaStore => "schema store",
      ProxyLabel::Target => "proxy id",
      ProxyLabel::Validations => "validations",
      ProxyLabel::Zone => "zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<ProxyLabel> for KafkaProxy {
  fn value(&self, label: &ProxyLabel, target_id: &str) -> Value {
    match label {
      ProxyLabel::AclGroupsEnabled => Value::some_or_hide(self.enable_kafka_acl_groups),
      ProxyLabel::CaChainSecretName => Value::target(&self.secret_name_ca_chain),
      ProxyLabel::Certificate => Value::target(&self.certificate),
      ProxyLabel::Cpus => Value::plain(self.cpus),
      ProxyLabel::Instances => Value::plain(self.instances),
      ProxyLabel::Mem => Value::plain(self.mem),
      ProxyLabel::Name => Value::some_or_empty(self.name.clone()),
      ProxyLabel::SchemaStore => Value::some_or(
        self.schema_store.map(|enabled| {
          if enabled {
            format!(
              "true (cpus: {}, mem: {})",
              self.schema_store_cpus.map(|cpus| cpus.to_string()).unwrap_or("NA".to_string()),
              self.schema_store_mem.map(|mem| mem.to_string()).unwrap_or("NA".to_string())
            )
          } else {
            "false".to_string()
          }
        }),
        "NA",
      ),
      ProxyLabel::Target => Value::target(target_id),
      ProxyLabel::Validations => {
        if self.validations.is_empty() {
          Value::plain("none")
        } else {
          Value::plain(
            self
              .validations
              .iter()
              .map(|validation| validation.common_name.clone().unwrap_or_default())
              .join("\n"),
          )
        }
      }
      ProxyLabel::Zone => Value::plain(self.zone),
    }
  }
}

static PROXY_LABELS_SHOW: [ProxyLabel; 11] = [
  ProxyLabel::Target,
  ProxyLabel::Certificate,
  ProxyLabel::CaChainSecretName,
  ProxyLabel::Cpus,
  ProxyLabel::Instances,
  ProxyLabel::Zone,
  ProxyLabel::Mem,
  ProxyLabel::Name,
  ProxyLabel::SchemaStore,
  ProxyLabel::Validations,
  ProxyLabel::AclGroupsEnabled,
];

#[derive(Eq, Hash, PartialEq, Serialize)]
enum ProxyBundleLabel {
  CaCommonName,
  BrokerPrefix,
  BundleDirectory,
  BundleName,
  HasSchemaStoreDnsRecord,
  NumberOfDsnRecords,
  Platform,
  Tenant,
  VhostZone,
}

impl Label for ProxyBundleLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::BrokerPrefix => "prefix",
      Self::BundleDirectory => "directory",
      Self::BundleName => "bundle",
      Self::CaCommonName => "ca common name",
      Self::HasSchemaStoreDnsRecord => "schema",
      Self::Platform => "platform",
      Self::VhostZone => "vhost zone",
      Self::Tenant => "tenant",
      Self::NumberOfDsnRecords => "records",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, ProxyBundleLabel::BundleName)
  }
}

impl SubjectFormatter<ProxyBundleLabel> for ProxyCertificateBundleConfig {
  fn value(&self, label: &ProxyBundleLabel, target_id: &str) -> Value {
    match label {
      ProxyBundleLabel::BrokerPrefix => Value::plain(&self.broker_prefix),
      ProxyBundleLabel::BundleDirectory => Value::unreachable(),
      ProxyBundleLabel::BundleName => Value::target(target_id),
      ProxyBundleLabel::CaCommonName => Value::plain(&self.ca_common_name),
      ProxyBundleLabel::HasSchemaStoreDnsRecord => Value::plain(self.include_schema_store_dns_record),
      ProxyBundleLabel::Platform => Value::target(&self.platform),
      ProxyBundleLabel::Tenant => Value::target(&self.tenant),
      ProxyBundleLabel::VhostZone => Value::plain(&self.vhost_zone),
      ProxyBundleLabel::NumberOfDsnRecords => Value::plain(self.number_of_dns_records),
    }
  }
}

impl SubjectFormatter<ProxyBundleLabel> for (ProxyCertificateBundleConfig, String) {
  fn value(&self, label: &ProxyBundleLabel, target_id: &str) -> Value {
    let (config, directory) = self;
    match label {
      ProxyBundleLabel::BundleDirectory => Value::plain(directory),
      _ => config.value(label, target_id),
    }
  }
}
