use crate::argument_parsers::RangedValueParser;
use clap::{builder, Arg, ArgAction};
use std::num::NonZeroU64;

pub(crate) const CERTIFICATE_COUNT_OPTION: &str = "certificate-count-option";
pub(crate) const CONSUMER_RATE_OPTION: &str = "consumer-rate-option";
pub(crate) const CPU_OPTION: &str = "cpu-option";
pub(crate) const KAFKA_ACL_GROUP_COUNT_OPTION: &str = "kafka-acl-group-count-option";
pub(crate) const MEM_OPTION: &str = "mem-option";
pub(crate) const PARTITION_COUNT_OPTION: &str = "partition-count-option";
pub(crate) const PRODUCER_RATE_OPTION: &str = "producer-rate-option";
pub(crate) const REQUEST_RATE_OPTION: &str = "request-rate-option";
pub(crate) const SECRET_COUNT_OPTION: &str = "secret-count-option";
pub(crate) const STREAM_READ_OPTION: &str = "stream-read-option";
pub(crate) const STREAM_RW_OPTION: &str = "stream-rw-option";
pub(crate) const STREAM_WRITE_OPTION: &str = "stream-write-option";
pub(crate) const TOPIC_COUNT_OPTION: &str = "topic-count-option";
pub(crate) const TRACING_OPTION: &str = "tracing-option";
pub(crate) const VPN_OPTION: &str = "vpn-option";

pub(crate) fn certificate_count_flag() -> Arg {
  Arg::new(CERTIFICATE_COUNT_OPTION)
    .long("certificate-count")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<NonZeroU64>::new(NonZeroU64::new(1).unwrap(), NonZeroU64::new(40).unwrap()))
    .value_name("COUNT")
    .help("Limit for number of certificates")
    .long_help(
      "Set the limit for the number of certificates available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn consumer_rate_flag() -> Arg {
  Arg::new(CONSUMER_RATE_OPTION)
    .long("consumer-rate")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<i64>::new(1048576, 1250000000))
    .value_name("RATE")
    .help("Limit for consumer rate")
    .long_help(
      "Set the limit for the maximum allowed consumer rate (bytes/sec). \
          The value must be greater than or equal to 1048576 \
          and lower than or equal to 1250000000.",
    )
}

pub(crate) fn cpu_flag() -> Arg {
  Arg::new(CPU_OPTION)
    .long("cpu")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<f64>::new(0.01, 16.0))
    .value_name("CPU")
    .help("Limit for number of cpus")
    .long_help(
      "Set the limit for the number of cpus to provision for the managed tenant \
          (factions of a vCPU core, 1.0 equals 1 vCPU). \
          The value must be greater than or equal to 0.01 \
          and lower than or equal to 16.0.",
    )
}

pub(crate) fn kafka_acl_group_flag() -> Arg {
  Arg::new(KAFKA_ACL_GROUP_COUNT_OPTION)
    .long("kafka-acl-group-count")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<i64>::new(0, 100))
    .value_name("COUNT")
    .help("Limit for number of kafka acl groups")
    .long_help(
      "Set the limit for the number of Kafka ACL groups available for the managed tenant. \
          The value must be greater than or equal to 0 and lower than or equal to 100.",
    )
}

pub(crate) fn mem_flag() -> Arg {
  Arg::new(MEM_OPTION)
    .long("mem")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<NonZeroU64>::new(NonZeroU64::new(1).unwrap(), NonZeroU64::new(131072).unwrap()))
    .value_name("MEM")
    .help("Limit for amount of memory")
    .long_help(
      "Set the limit for the amount of memory available for the managed tenant (MiB). \
          The value must be greater than or equal to 1 and lower than or equal to 131072.",
    )
}

pub(crate) fn partition_count_flag() -> Arg {
  Arg::new(PARTITION_COUNT_OPTION)
    .long("partition-count")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<NonZeroU64>::new(NonZeroU64::new(1).unwrap(), NonZeroU64::new(40).unwrap()))
    .value_name("COUNT")
    .help("Limit for number of partitions")
    .long_help(
      "Set the limit for the number of partitions available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn producer_rate_flag() -> Arg {
  Arg::new(PRODUCER_RATE_OPTION)
    .long("producer-rate")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<i64>::new(1048576, 1250000000))
    .value_name("RATE")
    .help("Limit for producer rate")
    .long_help(
      "Set the limit for the maximum allowed producer rate (bytes/sec). \
          The value must be greater than or equal to 1048576 \
          and lower than or equal to 1250000000.",
    )
}

pub(crate) fn request_rate_flag() -> Arg {
  Arg::new(REQUEST_RATE_OPTION)
    .long("request-rate")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<NonZeroU64>::new(NonZeroU64::new(1).unwrap(), NonZeroU64::new(100).unwrap()))
    .value_name("RATE")
    .help("Limit for request rate")
    .long_help(
      "Set the limit for the maximum allowed request rate (%). \
          The value must be greater than or equal to 1 and lower than or equal to 100.",
    )
}

pub(crate) fn secret_count_flag() -> Arg {
  Arg::new(SECRET_COUNT_OPTION)
    .long("secret-count")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<NonZeroU64>::new(NonZeroU64::new(1).unwrap(), NonZeroU64::new(40).unwrap()))
    .value_name("COUNT")
    .help("Limit for number of secrets")
    .long_help(
      "Set the limit for the number of secrets available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn stream_read_flag(action: &str) -> Arg {
  Arg::new(STREAM_READ_OPTION)
    .long("stream-read")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("STREAM")
    .help(format!("{} read access", action))
    .long_help(format!("{} the managed tenant read access rights to a managed stream.", action))
    .conflicts_with_all([STREAM_RW_OPTION, STREAM_WRITE_OPTION])
}

pub(crate) fn stream_rw_flag(action: &str) -> Arg {
  Arg::new(STREAM_RW_OPTION)
    .long("stream-rw")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("STREAM")
    .help(format!("{} read/write access", action))
    .long_help(format!("{} the managed tenant read and write access rights to a managed stream.", action))
}

pub(crate) fn stream_write_flag(action: &str) -> Arg {
  Arg::new(STREAM_WRITE_OPTION)
    .long("stream-write")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("STREAM")
    .help(format!("{} write access", action))
    .long_help(format!("{} the managed tenant write access rights to a managed stream.", action))
}

pub(crate) fn topic_count_flag() -> Arg {
  Arg::new(TOPIC_COUNT_OPTION)
    .long("topic-count")
    .action(ArgAction::Set)
    .value_parser(RangedValueParser::<NonZeroU64>::new(NonZeroU64::new(1).unwrap(), NonZeroU64::new(40).unwrap()))
    .value_name("COUNT")
    .help("Limit for number of topics")
    .long_help(
      "Set the limit for the number of topics available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn tracing_flag() -> Arg {
  Arg::new(TRACING_OPTION)
    .long("tracing")
    .action(ArgAction::Set)
    .value_parser(builder::BoolValueParser::new())
    .value_name("TRACING")
    .help("Enable tracing capabilities")
    .long_help(
      "Indicates whether tracing capabilities for the managed tenant will be enabled. \
          The provided value must be 'true' or 'false'. \
          If this option is not provided, tracing capabilities will be disabled.",
    )
}

pub(crate) fn vpn_flag() -> Arg {
  Arg::new(VPN_OPTION)
    .long("vpn")
    .action(ArgAction::Set)
    .value_parser(builder::BoolValueParser::new())
    .value_name("VPN")
    .help("Enable vpn capabilities")
    .long_help(
      "Indicates whether vpn capabilities for the managed tenant will be enabled. \
          The provided value must be 'true' or 'false'. \
          If this option is not provided, vpn capabilities will be disabled.",
    )
}
