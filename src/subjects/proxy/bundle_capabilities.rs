use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::bundle::proxy::{LocalProxyCertificateBundle, ProxyCertificateBundle};
use crate::capability::CommandExecutor;
use crate::code::{delete_example_code, example_code_exists, generate_example_code};
use crate::context::Context;
use crate::directory::{
  certificate_bundle_exists, delete_certificate_bundle, list_proxy_certificate_bundles, read_local_certificate_bundle, store_proxy_certificate_bundle, BundleKind,
};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::global_options::get_expiration_days;
use crate::secret_metadata::SecretMetadata;
use crate::subject::Requirements;
use crate::subjects::aclgroup::options::ACL_GROUP_NAME_OPTION;
use crate::subjects::certificate::labels::CertificateLabel;
use crate::subjects::proxy::labels::ProxyBundleLabel;
use crate::subjects::proxy::options::{get_ca_common_name, get_number_of_dns_records, get_vhost_zone, ENABLE_SCHEMA_STORE_OPTION, LANGUAGE_ARGUMENT};
use crate::subjects::secret::labels::{SecretLabel, SecretMetadataExpirationDays};
use crate::target_platform::get_target_platform;
use crate::target_tenant::get_target_tenant;
use crate::verbosity::Verbosity;
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::platform::VhostZone;
use itertools::Itertools;
use log::trace;

pub(crate) struct BundleCode {}

#[async_trait]
impl CommandExecutor for BundleCode {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());
    let language = match matches.get_one::<String>(LANGUAGE_ARGUMENT) {
      Some(language) => language,
      None => return err!("language argument missing"),
    };
    let (bundle_configuration, bundle_directory) = match read_local_certificate_bundle(&platform, &tenant, &bundle_id) {
      Ok(LocalProxyCertificateBundle { configuration, .. }) => configuration,
      Err(_) => return err!("proxy certificate bundle '{}' for '{}@{}' does not exist", bundle_id, platform, tenant),
    };

    context.print_explanation(format!("generating {} example for bundle '{}' for '{}@{}'", language, bundle_id, platform, tenant));

    if example_code_exists(language, &bundle_configuration, context)? {
      context.print_warning(format!("'{}' {} example code already exists for '{}@{}'", bundle_id, language, platform, tenant));
      if !context.confirmed("do you want to delete the existing example code?")? {
        context.print_outcome("cancelled");
        return Ok(());
      } else if context.dry_run() {
        context.print_warning("dry-run mode, existing example code not deleted");
        return Ok(());
      } else {
        delete_example_code(language, &bundle_configuration, context)?;
      }
    }

    if context.dry_run() {
      context.print_warning("dry-run mode, no code generated");
    } else {
      match generate_example_code(language, &bundle_configuration, &bundle_directory, context)? {
        Some(example_directory) => context.print_outcome(format!(
          "{} code for bundle '{}' generated in directory '{}'",
          language, bundle_id, example_directory
        )),
        None => context.print_outcome(format!("{} code for bundle '{}' generated", language, bundle_id)),
      }
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

static BUNDLE_LABELS_CODE_CONFIGURATION: [ProxyBundleLabel; 8] = [
  ProxyBundleLabel::ClientId,
  ProxyBundleLabel::GroupId,
  ProxyBundleLabel::BundleDirectory,
  ProxyBundleLabel::PkiCaCertificateFilename,
  ProxyBundleLabel::PkiClientCertificateFilename,
  ProxyBundleLabel::PkiClientKeyFilename,
  ProxyBundleLabel::Brokers,
  ProxyBundleLabel::SchemaStoreEndpoint,
];

pub(crate) struct BundleCodeConfiguration {}

#[async_trait]
impl CommandExecutor for BundleCodeConfiguration {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());
    let (bundle_configuration, bundle_directory) = match read_local_certificate_bundle(&platform, &tenant, &bundle_id) {
      Ok(LocalProxyCertificateBundle { configuration, .. }) => configuration,
      Err(_) => return err!("proxy certificate bundle '{}' for '{}@{}' does not exist", bundle_id, platform, tenant),
    };
    context.print_explanation(format!(
      "listing Kafka client property values for bundle '{}' for '{}@{}'",
      bundle_id, platform, tenant
    ));
    UnitFormatter::new(&bundle_id, &BUNDLE_LABELS_CODE_CONFIGURATION, context).print(&(bundle_configuration, bundle_directory), None)
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

static GENERATED_CERTIFICATE_LABELS: [CertificateLabel; 6] = [
  CertificateLabel::Target,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::SerialNumber,
];

static PROXY_BUNDLE_LABELS_CREATE: [ProxyBundleLabel; 10] = [
  ProxyBundleLabel::Platform,
  ProxyBundleLabel::Tenant,
  ProxyBundleLabel::ProxyName,
  ProxyBundleLabel::BundleName,
  ProxyBundleLabel::GroupId,
  ProxyBundleLabel::CaCommonName,
  ProxyBundleLabel::SchemaStore,
  ProxyBundleLabel::VhostZone,
  ProxyBundleLabel::AclGroupName,
  ProxyBundleLabel::NumberOfDsnRecords,
];

pub(crate) struct BundleCreate {}

#[async_trait]
impl CommandExecutor for BundleCreate {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let proxy_bundle_id = target.unwrap_or_else(|| unreachable!());

    if certificate_bundle_exists(&platform, &tenant, BundleKind::Proxy, &proxy_bundle_id)? {
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

    let acl_group_name = get_acl_group_name(matches, context)?;

    let ca_common_name = get_ca_common_name(matches, context)?;
    let enable_schema_store = match matches.get_one::<bool>(ENABLE_SCHEMA_STORE_OPTION) {
      Some(enable_schema_store) => *enable_schema_store,
      None => context.confirmed("enable schema store?")?,
    };
    let number_of_dns_records = get_number_of_dns_records(matches, context)?;
    let vhost_zone = get_vhost_zone(matches, context, VhostZone::Private)?;

    let config = ProxyCertificateBundleConfig {
      acl_group_name,
      ca_common_name,
      enable_schema_store,
      number_of_dns_records,
      platform: platform.clone(),
      proxy_name: proxy_bundle_id.clone(),
      tenant: tenant.clone(),
      vhost_zone,
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

fn get_acl_group_name(matches: &ArgMatches, context: &Context) -> DshCliResult<Option<String>> {
  match matches.get_one::<String>(ACL_GROUP_NAME_OPTION) {
    Some(acl_group_name) => Ok(Some(acl_group_name.clone())),
    None => {
      if context.confirmed("enable acl groups?")? {
        let acl_group_name = context.read_single_line("acl group name")?;
        if acl_group_name.is_empty() {
          err!("acl group name cannot be empty")
        } else {
          Ok(Some(acl_group_name))
        }
      } else {
        Ok(None)
      }
    }
  }
}

pub(crate) struct BundleDelete {}

#[async_trait]
impl CommandExecutor for BundleDelete {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());
    if !certificate_bundle_exists(&platform, &tenant, BundleKind::Proxy, &bundle_id)? {
      return err!("proxy certificate bundle '{}' for '{}@{}' does not exist", bundle_id, platform, tenant);
    }
    context.print_explanation(format!("delete proxy certificates bundle '{}' for '{}@{}'", bundle_id, platform, tenant));
    if context.confirmed(format!("delete proxy certificate bundle '{}'?", bundle_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, proxy certificate bundle not deleted");
      } else {
        delete_certificate_bundle(&platform, &tenant, BundleKind::Proxy, &bundle_id)?;
        context.print_outcome(format!("proxy certificate bundle '{}' deleted", bundle_id));
      }
    } else {
      context.print_outcome(format!("cancelled, proxy certificate bundle '{}' not deleted", bundle_id));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

static BUNDLE_LABELS_LIST: [ProxyBundleLabel; 7] = [
  ProxyBundleLabel::BundleName,
  ProxyBundleLabel::CaCommonName,
  ProxyBundleLabel::SchemaStore,
  ProxyBundleLabel::VhostZone,
  ProxyBundleLabel::NumberOfDsnRecords,
  ProxyBundleLabel::AclGroupName,
  ProxyBundleLabel::BundleDirectory,
];

pub(crate) struct BundleList {}

#[async_trait]
impl CommandExecutor for BundleList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    context.print_explanation(format!("list all local proxy certificate bundles for '{}@{}'", platform.name(), tenant));
    let mut list: Vec<(String, (ProxyCertificateBundleConfig, String))> = list_proxy_certificate_bundles(&platform, &tenant)?
      .into_iter()
      .map(|(bundle_name, bundle_config, bundle_directory)| (bundle_name, (bundle_config, bundle_directory)))
      .collect_vec();
    list.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));
    let mut formatter = ListFormatter::new(&BUNDLE_LABELS_LIST, context);
    formatter.push_target_id_value_pairs(&list);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

static BUNDLE_SECRET_LABELS_SHOW: [SecretLabel; 5] = [SecretLabel::SecretName, SecretLabel::Kind, SecretLabel::Format, SecretLabel::Size, SecretLabel::Description];

static BUNDLE_LABELS_SHOW: [ProxyBundleLabel; 9] = [
  ProxyBundleLabel::CaCommonName,
  ProxyBundleLabel::SchemaStore,
  ProxyBundleLabel::NumberOfDsnRecords,
  ProxyBundleLabel::Platform,
  ProxyBundleLabel::ProxyName,
  ProxyBundleLabel::Tenant,
  ProxyBundleLabel::VhostZone,
  ProxyBundleLabel::AclGroupName,
  ProxyBundleLabel::BundleDirectory,
];

static BUNDLE_DERIVED_LABELS_SHOW: [ProxyBundleLabel; 5] =
  [ProxyBundleLabel::BundleName, ProxyBundleLabel::PlatformDomain, ProxyBundleLabel::DnsEntries, ProxyBundleLabel::GroupId, ProxyBundleLabel::ProxyCommonName];

pub(crate) struct BundleShow {}

#[async_trait]
impl CommandExecutor for BundleShow {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("show local certificate bundle '{}'", bundle_id));
    let expiration_days = get_expiration_days(matches, context.settings())?;
    let bundle = read_local_certificate_bundle(&platform, &tenant, &bundle_id)?;

    context.print_explanation(format!("configuration file '{}'", bundle.configuration.1));
    UnitFormatter::new(format!("{} / configuration", bundle_id), &BUNDLE_LABELS_SHOW, context).print(&bundle.configuration, None)?;

    context.print_explanation("derived values");
    UnitFormatter::new(format!("{} / derived values", bundle_id), &BUNDLE_DERIVED_LABELS_SHOW, context).print(&bundle.configuration, None)?;

    context.print_explanation(format!("server certificate file '{}'", bundle.server_pem.filename));
    UnitFormatter::new(format!("{} / server certificate", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(
      &SecretMetadataExpirationDays::new(SecretMetadata::from(bundle.server_pem.value.as_str()), Some(expiration_days)),
      None,
    )?;
    context.print_explanation(format!("server key file '{}'", bundle.server_key.filename));
    UnitFormatter::new(format!("{} / server key", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(
      &SecretMetadataExpirationDays::new(SecretMetadata::from(bundle.server_key.value.as_str()), Some(expiration_days)),
      None,
    )?;

    context.print_explanation(format!("client certificate file '{}'", bundle.client_pem.filename));
    UnitFormatter::new(format!("{} / client certificate", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(
      &SecretMetadataExpirationDays::new(SecretMetadata::from(bundle.client_pem.value.as_str()), Some(expiration_days)),
      None,
    )?;
    context.print_explanation(format!("client key file '{}'", bundle.client_key.filename));
    UnitFormatter::new(format!("{} / client key", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(
      &SecretMetadataExpirationDays::new(SecretMetadata::from(bundle.client_key.value.as_str()), Some(expiration_days)),
      None,
    )?;

    context.print_explanation(format!("certificate authority certificate file '{}'", bundle.ca_pem.filename));
    UnitFormatter::new(format!("{} / ca certificate", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(
      &SecretMetadataExpirationDays::new(SecretMetadata::from(bundle.ca_pem.value.as_str()), Some(expiration_days)),
      None,
    )?;
    context.print_explanation(format!("certificate authority key file '{}'", bundle.ca_key.filename));
    UnitFormatter::new(format!("{} / ca key", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(
      &SecretMetadataExpirationDays::new(SecretMetadata::from(bundle.ca_key.value.as_str()), Some(expiration_days)),
      None,
    )?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}
