use clap::{builder, Arg, ArgAction};

pub(crate) const APP_ID_ARGUMENT: &str = "app-id-argument";
pub(crate) const BUCKET_ID_ARGUMENT: &str = "bucket-id-argument";
pub(crate) const CERTIFICATE_ID_ARGUMENT: &str = "certificate-id-argument";
#[cfg(feature = "manage")]
pub(crate) const MANAGED_STREAM_ARGUMENT: &str = "managed-stream-argument";
#[cfg(feature = "manage")]
pub(crate) const MANAGED_TENANT_NAME_ARGUMENT: &str = "managed-tenant-name-argument";
pub(crate) const MANIFEST_ID_ARGUMENT: &str = "manifest-id-argument";
pub(crate) const NODEPOOL_ID_ARGUMENT: &str = "node-pool-id-argument";
pub(crate) const PROXY_ID_ARGUMENT: &str = "proxy-id-argument";
pub(crate) const QUERY_ARGUMENT: &str = "query-argument";
pub(crate) const SECRET_ID_ARGUMENT: &str = "secret-id-argument";
pub(crate) const SERVICE_ID_ARGUMENT: &str = "service-id-argument";
pub(crate) const TASK_ID_ARGUMENT: &str = "task-id-argument";
pub(crate) const TOPIC_ID_ARGUMENT: &str = "topic-id-argument";
pub(crate) const VENDOR_NAME_ARGUMENT: &str = "vendor-name-argument";
pub(crate) const MANIFEST_VERSION_ARGUMENT: &str = "version-argument";
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

#[cfg(feature = "manage")]
pub(crate) fn managed_stream_argument() -> Arg {
  Arg::new(MANAGED_STREAM_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("STREAM")
    .help("Stream identifier")
    .long_help("Identifies a managed stream on the DSH.")
}

#[cfg(feature = "manage")]
pub(crate) fn managed_tenant_argument() -> Arg {
  Arg::new(MANAGED_TENANT_NAME_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Managed tenant name")
    .long_help("The name of the managed tenant.")
}

pub(crate) fn manifest_id_argument() -> Arg {
  Arg::new(MANIFEST_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("MANIFEST")
    .help("Manifest identifier")
    .long_help("Identifies an app manifest from the app catalog.")
}

pub(crate) fn manifest_version_argument() -> Arg {
  Arg::new(MANIFEST_VERSION_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("VERSION")
    .help("App manifest version")
    .long_help("Identifies the version of an app manifest from the app catalog.")
}

pub(crate) fn nodepool_id_argument() -> Arg {
  Arg::new(NODEPOOL_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("NODEPOOL")
    .help("Node pool identifier")
    .long_help("Identifies a node pool on the DSH.")
}

pub(crate) fn proxy_id_argument() -> Arg {
  Arg::new(PROXY_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("PROXY")
    .help("Proxy identifier")
    .long_help("Identifies a proxy configured on the DSH.")
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

pub(crate) fn task_id_argument() -> Arg {
  Arg::new(TASK_ID_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TASK")
    .help("Task identifier")
    .long_help("Identifies a task within a service deployed on the DSH.")
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
