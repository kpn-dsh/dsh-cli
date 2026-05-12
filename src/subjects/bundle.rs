use crate::capability::{Capability, CommandExecutor, CODE_COMMAND, CREATE_COMMAND, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::directory::{
  delete_proxy_certificate_bundle, list_proxy_certificate_bundles, proxy_certificate_bundle_exists, read_local_certificate_bundle, store_proxy_certificate_bundle,
};
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::unit_formatter::UnitFormatter;
use crate::formatters::Value;
use crate::formatters::{Label, SubjectFormatter};
use crate::global_options::{expiration_option, get_expiration_days};
use crate::proxy_bundles::{Language, LocalCertificateBundle, ProxyCertificateBundle, ProxyCertificateBundleConfig};
use crate::secret_metadata::secret_metadata;
use crate::subject::{Requirements, Subject};
use crate::subjects::certificate::CertificateLabel;
use crate::subjects::secret::SecretLabel;
use crate::target_platform::{get_target_platform, platform_name_argument};
use crate::target_tenant::{get_target_tenant, tenant_name_argument};
use crate::verbosity::Verbosity;
use crate::{err, DshCliResult};
use async_trait::async_trait;
use clap::builder::{EnumValueParser, PossibleValue};
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::platform::VhostZone;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::trace;

use crate::arguments::proxy_id_argument;
use crate::code::{delete_example_code, example_code_exists, generate_example_code};
use serde::Serialize;
use std::convert::AsRef;
use std::str::FromStr;
use std::sync::LazyLock;
use whoami::username;

struct BundleSubject {}

const BUNDLE_SUBJECT_TARGET: &str = "bundle";

lazy_static! {
  pub(crate) static ref BUNDLE_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(BundleSubject {});
}

#[async_trait]
impl Subject for BundleSubject {
  fn subject(&self) -> &'static str {
    BUNDLE_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list proxy certificate bundles.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Show, manage and list proxy certificate bundles supporting DSH Kafka proxies.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CODE_COMMAND => Some(BUNDLE_CODE_CAPABILITY.as_ref()),
      CREATE_COMMAND => Some(BUNDLE_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(BUNDLE_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(BUNDLE_LIST_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(BUNDLE_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &BUNDLE_CAPABILITIES
  }
}

static BUNDLE_CODE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CODE_COMMAND, None, &BundleCode {}, "Generate example client code")
      .add_target_argument(proxy_id_argument().required(true))
      .add_extra_argument(language_option().required(true)),
  )
});
static BUNDLE_CREATE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, None, &BundleCreate {}, "Create local proxy certificates bundle")
      .add_target_argument(proxy_id_argument().required(true))
      .add_target_argument(platform_name_argument())
      .add_target_argument(tenant_name_argument())
      .add_extra_argument(acl_group_id_option())
      .add_extra_argument(number_of_dns_records_option())
      .add_extra_argument(ca_common_name_option())
      .add_extra_argument(vhost_zone_option())
      .add_extra_argument(enable_schema_store_option()),
  )
});
static BUNDLE_DELETE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &BundleDelete {}, "Delete local proxy certificates bundle")
      .add_target_argument(proxy_id_argument().required(true))
      .add_target_argument(platform_name_argument())
      .add_target_argument(tenant_name_argument()),
  )
});
static BUNDLE_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &BundleList {}, "List dsh proxies")
      .set_long_about("Lists all Kafka proxies used by the services and apps on the DSH."),
  )
});
static BUNDLE_SHOW_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), &BundleShow {}, "Show local proxy certificate bundle")
      .add_target_argument(proxy_id_argument().required(true))
      .add_extra_argument(expiration_option()),
  )
});

static BUNDLE_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![BUNDLE_CODE_CAPABILITY.as_ref(), BUNDLE_CREATE_CAPABILITY.as_ref(), BUNDLE_DELETE_CAPABILITY.as_ref(), BUNDLE_LIST_CAPABILITY.as_ref(), BUNDLE_SHOW_CAPABILITY.as_ref()]
});

const ACL_GROUP_ID_OPTION: &str = "acl-group-id-option";

fn acl_group_id_option() -> Arg {
  Arg::new(ACL_GROUP_ID_OPTION)
    .long("acl-group-id")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("ACL_GROUP_ID")
    .help("Acl group id")
    .long_help("Acl group id used for fine-grained access control.")
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

const ENABLE_SCHEMA_STORE_OPTION: &str = "enable-schema-store-option";

fn enable_schema_store_option() -> Arg {
  Arg::new(ENABLE_SCHEMA_STORE_OPTION)
    .long("enable-schema-store")
    .action(ArgAction::Set)
    .value_parser(builder::BoolValueParser::new())
    .help("Enable schema store")
    .long_help(
      "If this option is enabled the created certificates will include a dns entry \
    for a schema store.",
    )
}

const LANGUAGE_OPTION: &str = "language-option";

fn language_option() -> Arg {
  Arg::new(LANGUAGE_OPTION)
    .long("language")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<Language>::new())
    .help("Language")
    .long_help("Programming language for which example code will be generated.")
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
      "Number of dns records that will be generated in the proxy. Do not use this \
         option unless you know what you are doing.",
    )
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
  BundleLabel::AclGroupId,
  BundleLabel::NumberOfDsnRecords,
];

struct BundleCode {}

#[async_trait]
impl CommandExecutor for BundleCode {
  async fn execute_without_client(&self, target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    let bundle_id = target.unwrap_or_else(|| unreachable!());

    let (bundle_configuration, directory) = match read_local_certificate_bundle(&platform, &tenant, &bundle_id) {
      Ok(LocalCertificateBundle { configuration, .. }) => configuration,
      Err(_) => return err!("proxy certificate bundle '{}' for '{}@{}' does not exist", bundle_id, platform, tenant),
    };

    let language = match matches.get_one::<Language>(LANGUAGE_OPTION) {
      Some(language) => language.clone(),
      None => {
        let language_string = context.read_single_line_with_default("language", "rust")?;
        if language_string.is_empty() {
          return err!("language string cannot be empty");
        } else {
          Language::from_str(&language_string)?
        }
      }
    };
    context.print_explanation(format!("generating {} example for bundle '{}' for '{}@{}'", language, bundle_id, platform, tenant));

    if example_code_exists(&bundle_configuration, &language, context)? {
      context.print_warning(format!("'{}' {} example code already exists for '{}@{}'", bundle_id, language, platform, tenant));
      if !context.confirmed("do you want to delete the existing example code?")? {
        context.print_outcome("cancelled");
        return Ok(());
      } else if context.dry_run() {
        context.print_warning("dry-run mode, existing example code not deleted");
        return Ok(());
      } else {
        delete_example_code(&bundle_configuration, &language, context)?;
      }
    }

    if context.dry_run() {
      context.print_warning("dry-run mode, no code generated");
    } else {
      let example_directory = generate_example_code(&bundle_configuration, &language, &directory, context)?;
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

    let vhost_zone = match matches.get_one::<String>(VHOST_ZONE_OPTION) {
      Some(vhost_zone) => VhostZone::from_str(vhost_zone)?,
      None => {
        let vhost_zone_string = context.read_single_line("vhost zone [PRIVATE/public]")?;
        if vhost_zone_string.is_empty() {
          VhostZone::Private
        } else {
          VhostZone::from_str(&vhost_zone_string)?
        }
      }
    };

    let enable_schema_store = match matches.get_one::<bool>(ENABLE_SCHEMA_STORE_OPTION) {
      Some(enable_schema_store) => *enable_schema_store,
      None => context.confirmed("enable schema store?")?,
    };

    let acl_group_id: Option<String> = match matches.get_one::<String>(ACL_GROUP_ID_OPTION) {
      Some(acl_group_id) => Some(acl_group_id.clone()),
      None => {
        if context.confirmed("enable acl groups?")? {
          let acl_group_id = context.read_single_line("acl group id")?;
          if acl_group_id.is_empty() {
            return err!("acl group id cannot be empty");
          } else {
            Some(acl_group_id)
          }
        } else {
          None
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
      acl_group_id,
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
  BundleLabel::AclGroupId,
  BundleLabel::BundleDirectory,
];

struct BundleList {}

#[async_trait]
impl CommandExecutor for BundleList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let platform = get_target_platform(matches, context.settings())?;
    let tenant = get_target_tenant(matches, context.settings())?;
    context.print_explanation(format!("list all local proxy certificate bundles for '{}@{}'", platform.name(), tenant));
    let list: Vec<(String, (ProxyCertificateBundleConfig, String))> = list_proxy_certificate_bundles(&platform, &tenant)?
      .into_iter()
      .map(|(bundle_name, bundle_config, bundle_directory)| (bundle_name, (bundle_config, bundle_directory)))
      .collect_vec();
    let mut formatter = ListFormatter::new(&BUNDLE_LABELS_LIST, context);
    formatter.push_target_id_value_pairs(&list);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

static BUNDLE_SECRET_LABELS_SHOW: [SecretLabel; 10] = [
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

static BUNDLE_LABELS_SHOW: [BundleLabel; 9] = [
  BundleLabel::AclGroupId,
  BundleLabel::CaCommonName,
  BundleLabel::SchemaStore,
  BundleLabel::NumberOfDsnRecords,
  BundleLabel::Platform,
  BundleLabel::ProxyName,
  BundleLabel::Tenant,
  BundleLabel::VhostZone,
  BundleLabel::BundleDirectory,
];

static BUNDLE_DERIVED_LABELS_SHOW: [BundleLabel; 6] =
  [BundleLabel::BundleName, BundleLabel::PlatformDomain, BundleLabel::DnsEntries, BundleLabel::GroupId, BundleLabel::OrganizationalUnitName, BundleLabel::ProxyCommonName];

struct BundleShow {}

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
    UnitFormatter::new(&bundle_id, &BUNDLE_LABELS_SHOW, context).print(&bundle.configuration, None)?;
    context.print_explanation("derived values");
    UnitFormatter::new(&bundle_id, &BUNDLE_DERIVED_LABELS_SHOW, context).print(&bundle.configuration, None)?;
    context.print_explanation(format!("server certificate file '{}'", bundle.server_pem.filename));
    UnitFormatter::new(&bundle_id, &BUNDLE_SECRET_LABELS_SHOW, context).print(&(secret_metadata(&bundle.server_pem.value), Some(expiration_days)), None)?;
    context.print_explanation(format!("client key file '{}'", bundle.client_key.filename));
    UnitFormatter::new(&bundle_id, &BUNDLE_SECRET_LABELS_SHOW, context).print(&(secret_metadata(&bundle.client_key.value), Some(expiration_days)), None)?;
    context.print_explanation(format!("certificate authority certificate file '{}'", bundle.ca_pem.filename));
    UnitFormatter::new(&bundle_id, &BUNDLE_SECRET_LABELS_SHOW, context).print(&(secret_metadata(&bundle.ca_pem.value), Some(expiration_days)), None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum BundleLabel {
  AclGroupId,
  BundleDirectory,
  BundleName,
  CaCommonName,
  DnsEntries,
  GroupId,
  NumberOfDsnRecords,
  OrganizationalUnitName,
  Platform,
  PlatformDomain,
  ProxyCommonName,
  ProxyName,
  SchemaStore,
  Tenant,
  VhostZone,
}

impl Label for BundleLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::AclGroupId => "acl group id",
      Self::BundleDirectory => "directory",
      Self::BundleName => "bundle",
      Self::CaCommonName => "ca common name",
      Self::DnsEntries => "dns entries",
      Self::GroupId => "group id",
      Self::NumberOfDsnRecords => "records",
      Self::OrganizationalUnitName => "organizational unit name",
      Self::Platform => "platform",
      Self::PlatformDomain => "platform domain",
      Self::ProxyCommonName => "proxy common name",
      Self::ProxyName => "proxy name",
      Self::SchemaStore => "schema store",
      Self::Tenant => "tenant",
      Self::VhostZone => "vhost zone",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, BundleLabel::BundleName)
  }
}

impl SubjectFormatter<BundleLabel> for (ProxyCertificateBundleConfig, String) {
  fn value(&self, label: &BundleLabel, target_id: &str) -> Value {
    let (config, directory) = self;
    match label {
      BundleLabel::BundleDirectory => Value::plain(directory),
      _ => config.value(label, target_id),
    }
  }
}

impl SubjectFormatter<BundleLabel> for ProxyCertificateBundleConfig {
  fn value(&self, label: &BundleLabel, target_id: &str) -> Value {
    match label {
      BundleLabel::AclGroupId => Value::some_or_hide(self.acl_group_id.clone()),
      BundleLabel::BundleDirectory => Value::unreachable(),
      BundleLabel::BundleName => Value::target(target_id),
      BundleLabel::CaCommonName => Value::plain(&self.ca_common_name),
      BundleLabel::DnsEntries => Value::result(self.dns_entries().map(|dns_entry| dns_entry.join("\n"))),
      BundleLabel::GroupId => Value::plain(self.group_id(0)),
      BundleLabel::NumberOfDsnRecords => Value::plain(self.number_of_dns_records),
      BundleLabel::OrganizationalUnitName => Value::some_or_hide(self.organizational_unit_name()),
      BundleLabel::Platform => Value::target(&self.platform),
      BundleLabel::PlatformDomain => Value::result(self.domain_from_platform()),
      BundleLabel::ProxyCommonName => Value::result(self.common_name()),
      BundleLabel::ProxyName => Value::target(&self.proxy_name),
      BundleLabel::SchemaStore => {
        if self.enable_schema_store {
          Value::plain("enabled")
        } else {
          Value::plain("disabled")
        }
      }
      BundleLabel::Tenant => Value::target(&self.tenant),
      BundleLabel::VhostZone => Value::plain(&self.vhost_zone),
    }
  }
}
