use crate::arguments::proxy_id_argument;
use crate::capability::{Capability, CommandExecutor, CODE_COMMAND, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, DELETE_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::code::{delete_example_code, example_code_exists, generate_example_code};
use crate::context::Context;
use crate::directory::{
  delete_proxy_certificate_bundle, list_proxy_certificate_bundles, proxy_certificate_bundle_exists, read_local_certificate_bundle, store_proxy_certificate_bundle,
};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::global_options::get_expiration_days;
use crate::proxy_bundles::{LocalCertificateBundle, ProxyCertificateBundle, ProxyCertificateBundleConfig};
use crate::secret_metadata::secret_metadata;
use crate::subject::Requirements;
use crate::subjects::aclgroup::options::{acl_group_name_option, ACL_GROUP_NAME_OPTION};
use crate::subjects::certificate::CertificateLabel;
use crate::subjects::proxy::labels::BundleLabel;
use crate::subjects::proxy::options::{
  ca_common_name_option, enable_schema_store_option, example_argument, get_ca_common_name, get_number_of_dns_records, get_vhost_zone, language_argument,
  number_of_dns_records_option, vhost_zone_option, ENABLE_SCHEMA_STORE_OPTION, EXAMPLE_ARGUMENT, LANGUAGE_ARGUMENT,
};
use crate::subjects::secret::SecretLabel;
use crate::target_platform::{get_target_platform, platform_name_argument};
use crate::target_tenant::{get_target_tenant, tenant_name_argument};
use crate::verbosity::Verbosity;
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::ArgMatches;
use itertools::Itertools;
use log::trace;
use std::sync::LazyLock;

pub(crate) static BUNDLE_CODE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CODE_COMMAND, None, &BundleCode {}, "Generate example client code")
      .add_target_argument(proxy_id_argument().required(true))
      .add_target_argument(language_argument().required(true))
      .add_target_argument(example_argument()),
  )
});
pub(crate) static BUNDLE_CREATE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(
      CREATE_COMMAND,
      Some(CREATE_COMMAND_ALIAS),
      &BundleCreate {},
      "Create local proxy certificates bundle",
    )
    .add_target_argument(proxy_id_argument().required(true))
    .add_target_argument(platform_name_argument())
    .add_target_argument(tenant_name_argument())
    .add_extra_argument(acl_group_name_option())
    .add_extra_argument(ca_common_name_option())
    .add_extra_argument(enable_schema_store_option())
    .add_extra_argument(number_of_dns_records_option())
    .add_extra_argument(vhost_zone_option()),
  )
});
pub(crate) static BUNDLE_DELETE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(
      DELETE_COMMAND,
      Some(DELETE_COMMAND_ALIAS),
      &BundleDelete {},
      "Delete local proxy certificates bundle",
    )
    .add_target_argument(proxy_id_argument().required(true))
    .add_target_argument(platform_name_argument())
    .add_target_argument(tenant_name_argument()),
  )
});

static GENERATED_CERTIFICATE_LABELS: [CertificateLabel; 6] = [
  CertificateLabel::Target,
  CertificateLabel::DistinguishedName,
  CertificateLabel::DnsNames,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::SerialNumber,
];
static BUNDLE_LABELS_CREATE: [BundleLabel; 10] = [
  BundleLabel::Platform,
  BundleLabel::Tenant,
  BundleLabel::ProxyName,
  BundleLabel::BundleName,
  BundleLabel::GroupId,
  BundleLabel::CaCommonName,
  BundleLabel::SchemaStore,
  BundleLabel::VhostZone,
  BundleLabel::AclGroupName,
  BundleLabel::NumberOfDsnRecords,
];

struct BundleCode {}

#[async_trait]
impl CommandExecutor for BundleCode {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());
    let language = matches.get_one::<String>(LANGUAGE_ARGUMENT).unwrap_or_else(|| unreachable!());
    let example = match matches.get_one::<String>(EXAMPLE_ARGUMENT) {
      Some(example) => example.to_string(),
      None => "topics".to_string(),
    };

    let (bundle_configuration, directory) = match read_local_certificate_bundle(&platform, &tenant, &bundle_id) {
      Ok(LocalCertificateBundle { configuration, .. }) => configuration,
      Err(_) => return err!("proxy certificate bundle '{}' for '{}@{}' does not exist", bundle_id, platform, tenant),
    };

    context.print_explanation(format!(
      "generating {} {} example for bundle '{}' for '{}@{}'",
      language, example, bundle_id, platform, tenant
    ));

    if example_code_exists(language, &example, &bundle_configuration, context)? {
      context.print_warning(format!(
        "'{}' {} {} example code already exists for '{}@{}'",
        bundle_id, language, example, platform, tenant
      ));
      if !context.confirmed("do you want to delete the existing example code?")? {
        context.print_outcome("cancelled");
        return Ok(());
      } else if context.dry_run() {
        context.print_warning("dry-run mode, existing example code not deleted");
        return Ok(());
      } else {
        delete_example_code(language, &example, &bundle_configuration, context)?;
      }
    }

    if context.dry_run() {
      context.print_warning("dry-run mode, no code generated");
    } else {
      let example_directory = generate_example_code(language, &example, &bundle_configuration, &directory, context)?;
      context.print_outcome(format!(
        "{} code for bundle '{}' generated in directory '{}'",
        language, bundle_id, example_directory
      ));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct BundleCreate {}

#[async_trait]
impl CommandExecutor for BundleCreate {
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

    let acl_group_name = get_acl_group_name(matches, context)?;

    let ca_common_name = get_ca_common_name(matches, context)?;
    let enable_schema_store = match matches.get_one::<bool>(ENABLE_SCHEMA_STORE_OPTION) {
      Some(enable_schema_store) => *enable_schema_store,
      None => context.confirmed("enable schema store?")?,
    };
    let number_of_dns_records = get_number_of_dns_records(matches, context)?;
    let vhost_zone = get_vhost_zone(matches, context)?;

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
        Verbosity::Medium | Verbosity::High => UnitFormatter::new(&proxy_bundle_id, &BUNDLE_LABELS_CREATE, context).print(&config, None)?,
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

struct BundleDelete {}

#[async_trait]
impl CommandExecutor for BundleDelete {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());
    if !proxy_certificate_bundle_exists(&platform, &tenant, &bundle_id)? {
      return err!("proxy certificate bundle '{}' for '{}@{}' does not exist", bundle_id, platform, tenant);
    }
    context.print_explanation(format!("delete proxy certificates bundle '{}' for '{}@{}'", bundle_id, platform, tenant));
    if context.confirmed(format!("delete proxy certificate bundle '{}'?", bundle_id))? {
      if context.dry_run() {
        context.print_warning("dry-run mode, proxy certificate bundle not deleted");
      } else {
        delete_proxy_certificate_bundle(&platform, &tenant, &bundle_id)?;
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

static BUNDLE_LABELS_LIST: [BundleLabel; 7] = [
  BundleLabel::BundleName,
  BundleLabel::CaCommonName,
  BundleLabel::SchemaStore,
  BundleLabel::VhostZone,
  BundleLabel::NumberOfDsnRecords,
  BundleLabel::AclGroupName,
  BundleLabel::BundleDirectory,
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

static BUNDLE_SECRET_LABELS_SHOW: [SecretLabel; 11] = [
  SecretLabel::SecretName,
  SecretLabel::Kind,
  SecretLabel::FormatKind,
  SecretLabel::Size,
  SecretLabel::Description,
  SecretLabel::NotBefore,
  SecretLabel::NotAfter,
  SecretLabel::Subject,
  SecretLabel::Issuer,
  SecretLabel::CaChain,
  SecretLabel::Subject,
];

static BUNDLE_LABELS_SHOW: [BundleLabel; 9] = [
  BundleLabel::CaCommonName,
  BundleLabel::SchemaStore,
  BundleLabel::NumberOfDsnRecords,
  BundleLabel::Platform,
  BundleLabel::ProxyName,
  BundleLabel::Tenant,
  BundleLabel::VhostZone,
  BundleLabel::AclGroupName,
  BundleLabel::BundleDirectory,
];

static BUNDLE_DERIVED_LABELS_SHOW: [BundleLabel; 5] =
  [BundleLabel::BundleName, BundleLabel::PlatformDomain, BundleLabel::DnsEntries, BundleLabel::GroupId, BundleLabel::ProxyCommonName];

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
    UnitFormatter::new(format!("{} / server certificate", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context)
      .print(&(secret_metadata(&bundle.server_pem.value), Some(expiration_days)), None)?;
    context.print_explanation(format!("server key file '{}'", bundle.server_key.filename));
    UnitFormatter::new(format!("{} / server key", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context)
      .print(&(secret_metadata(&bundle.server_key.value), Some(expiration_days)), None)?;

    context.print_explanation(format!("client certificate file '{}'", bundle.client_pem.filename));
    UnitFormatter::new(format!("{} / client certificate", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context)
      .print(&(secret_metadata(&bundle.client_pem.value), Some(expiration_days)), None)?;
    context.print_explanation(format!("client key file '{}'", bundle.client_key.filename));
    UnitFormatter::new(format!("{} / client key", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context)
      .print(&(secret_metadata(&bundle.client_key.value), Some(expiration_days)), None)?;

    context.print_explanation(format!("certificate authority certificate file '{}'", bundle.ca_pem.filename));
    UnitFormatter::new(format!("{} / ca certificate", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context)
      .print(&(secret_metadata(&bundle.ca_pem.value), Some(expiration_days)), None)?;
    context.print_explanation(format!("certificate authority key file '{}'", bundle.ca_key.filename));
    UnitFormatter::new(format!("{} / ca key", bundle_id), &BUNDLE_SECRET_LABELS_SHOW, context).print(&(secret_metadata(&bundle.ca_key.value), Some(expiration_days)), None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}
