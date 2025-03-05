use clap::builder::PossibleValue;
use clap::{builder, Arg, ArgAction};
use dsh_api::platform::DshPlatform;

pub(crate) const APP_ID_ARGUMENT: &str = "app-id-argument";
pub(crate) const BUCKET_ID_ARGUMENT: &str = "bucket-id-argument";
pub(crate) const CERTIFICATE_ID_ARGUMENT: &str = "certificate-id-argument";
#[cfg(feature = "appcatalog")]
pub(crate) const MANIFEST_ID_ARGUMENT: &str = "manifest-id-argument";
pub(crate) const PLATFORM_NAME_ARGUMENT: &str = "platform-name-argument";
pub(crate) const PROXY_ID_ARGUMENT: &str = "proxy-argument";
pub(crate) const QUERY_ARGUMENT: &str = "query-argument";
pub(crate) const SECRET_ID_ARGUMENT: &str = "secret-id-argument";
pub(crate) const SERVICE_ID_ARGUMENT: &str = "service-id-argument";
pub(crate) const TENANT_NAME_ARGUMENT: &str = "tenant-name-argument";
pub(crate) const TOPIC_ID_ARGUMENT: &str = "topic-id-argument";
pub(crate) const VENDOR_NAME_ARGUMENT: &str = "vendor-name-argument";
pub(crate) const VHOST_ID_ARGUMENT: &str = "vhost-id-argument";
pub(crate) const VOLUME_ID_ARGUMENT: &str = "volume-id-argument";

pub(crate) fn app_id_argument() -> Arg {
  Arg::new(APP_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("APP")
    .help("App identifier")
    .long_help("Identifies an app from the app catalog.")
}

pub(crate) fn bucket_id_argument() -> Arg {
  Arg::new(BUCKET_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("BUCKET")
    .help("Bucket identifier")
    .long_help("Identifies an S3 bucket on the DSH.")
}

pub(crate) fn certificate_id_argument() -> Arg {
  Arg::new(CERTIFICATE_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("CERT")
    .help("Certificate identifier")
    .long_help("Identifies a certificate on the DSH.")
}

#[cfg(feature = "appcatalog")]
pub(crate) fn manifest_id_argument() -> Arg {
  Arg::new(MANIFEST_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("MANIFEST")
    .help("Manifest identifier")
    .long_help("Identifies a manifest from the app catalog.")
}

pub(crate) fn platform_name_argument() -> Arg {
  let possible_values = DshPlatform::all()
    .iter()
    .map(|platform| {
      PossibleValue::new(platform.name())
        .alias(platform.alias())
        .help(format!("{} ({})", platform.description(), platform.alias()))
    })
    .collect::<Vec<_>>();
  Arg::new(PLATFORM_NAME_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(possible_values)
    .value_name("PLATFORM")
    .help("Platform")
    .long_help("The name or alias of the platform.")
}

pub(crate) fn query_argument(long_help: Option<&str>) -> Arg {
  let mut query_argument = Arg::new(QUERY_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("QUERY")
    .help("Query");
  if let Some(long_help) = long_help {
    query_argument = query_argument.long_help(long_help.to_string())
  }
  query_argument
}

pub(crate) fn proxy_id_argument() -> Arg {
  Arg::new(PROXY_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("PROXY")
    .help("Proxy identifier")
    .long_help("Identifies a proxy configured on the DSH.")
}

pub(crate) fn secret_id_argument() -> Arg {
  Arg::new(SECRET_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("SECRET")
    .help("Secret identifier")
    .long_help("Identifies a secret configured on the DSH.")
}

pub(crate) fn service_id_argument() -> Arg {
  Arg::new(SERVICE_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("SERVICE")
    .help("Service identifier")
    .long_help("Identifies a service deployed on the DSH.")
}

pub(crate) fn tenant_name_argument() -> Arg {
  Arg::new(TENANT_NAME_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Tenant name")
    .long_help("The name of the tenant.")
}

pub(crate) fn topic_id_argument() -> Arg {
  Arg::new(TOPIC_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Topic identifier")
    .long_help("Identifies a topic deployed on the DSH.")
}

pub(crate) fn vendor_name_argument() -> Arg {
  Arg::new(VENDOR_NAME_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VENDOR")
    .help("Provide app vendor")
    .long_help("This option specifies the name of an app vendor. Allowed values are \"kpn\".")
}

pub(crate) fn vhost_id_argument() -> Arg {
  Arg::new(VHOST_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VHOST")
    .help("Vhost identifier")
    .long_help("Identifies a vhost configured on the DSH.")
}

pub(crate) fn volume_id_argument() -> Arg {
  Arg::new(VOLUME_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VOLUME")
    .help("Volume identifier")
    .long_help("Identifies a volume configured on the DSH.")
}
