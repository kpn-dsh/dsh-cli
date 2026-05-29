use clap::{builder, Arg, ArgAction};

pub(crate) const READ_TOPIC_OPTION: &str = "read-topic-option";

pub(crate) fn read_topic_option() -> Arg {
  Arg::new(READ_TOPIC_OPTION)
    .long("read-topic")
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Grant read access to topic for ACL group")
}

pub(crate) const WRITE_TOPIC_OPTION: &str = "write-topic-option";

pub(crate) fn write_topic_option() -> Arg {
  Arg::new(WRITE_TOPIC_OPTION)
    .long("write-topic")
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Grant write access to topic for ACL group")
}

pub(crate) const READ_INTERNAL_OPTION: &str = "read-internal-option";

pub(crate) fn read_internal_option() -> Arg {
  Arg::new(READ_INTERNAL_OPTION)
    .long("read-internal")
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Grant read access to internal topic for ACL group")
}

pub(crate) const WRITE_INTERNAL_OPTION: &str = "write-internal-option";

pub(crate) fn write_internal_option() -> Arg {
  Arg::new(WRITE_INTERNAL_OPTION)
    .long("write-internal")
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Grant write access to internal topic for ACL group")
}

pub(crate) const READ_PUBLIC_OPTION: &str = "read-public-option";

pub(crate) fn read_public_option() -> Arg {
  Arg::new(READ_PUBLIC_OPTION)
    .long("read-public")
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Grant read access to public topic for ACL group")
}

pub(crate) const WRITE_PUBLIC_OPTION: &str = "write-public-option";

pub(crate) fn write_public_option() -> Arg {
  Arg::new(WRITE_PUBLIC_OPTION)
    .long("write-public")
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TOPIC")
    .help("Grant write access to public topic for ACL group")
}

pub(crate) const ACL_GROUP_NAME_OPTION: &str = "acl-group-name-option";

pub(crate) fn acl_group_name_option() -> Arg {
  Arg::new(ACL_GROUP_NAME_OPTION)
    .long("acl-group-name")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("ID")
    .help("Acl group id")
    .long_help("Acl group name used for fine-grained access control.")
}
