use clap::{builder, Arg, ArgAction};

pub(crate) const CERTIFICATE_COUNT_FLAG: &str = "certificate-count";
pub(crate) const CONSUMER_RATE_FLAG: &str = "consumer-rate";
pub(crate) const CPU_FLAG: &str = "cpu";
pub(crate) const KAFKA_ACL_GROUP_COUNT_FLAG: &str = "kafka-acl-group-count";
pub(crate) const MEM_FLAG: &str = "mem";
pub(crate) const PARTITION_COUNT_FLAG: &str = "partition-count";
pub(crate) const PRODUCER_RATE_FLAG: &str = "producer-rate";
pub(crate) const REQUEST_RATE_FLAG: &str = "request-rate";
pub(crate) const SECRET_COUNT_FLAG: &str = "secret-count";
pub(crate) const TOPIC_COUNT_FLAG: &str = "topic-count";
pub(crate) const TRACING_FLAG: &str = "tracing";
pub(crate) const VPN_FLAG: &str = "vpn";

// TODO Remove conflicts_with_all once PATCH /manage/{manager}/tenant/{tenant}/limit is fixed
pub(crate) fn certificate_count_flag() -> Arg {
  Arg::new(CERTIFICATE_COUNT_FLAG)
    .long(CERTIFICATE_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Limit for number of certificates")
    .long_help(
      "Set the limit for the number of certificates available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
    .conflicts_with_all([
      CONSUMER_RATE_FLAG,
      CPU_FLAG,
      KAFKA_ACL_GROUP_COUNT_FLAG,
      MEM_FLAG,
      PARTITION_COUNT_FLAG,
      PRODUCER_RATE_FLAG,
      REQUEST_RATE_FLAG,
      SECRET_COUNT_FLAG,
      TOPIC_COUNT_FLAG,
    ])
}

pub(crate) fn consumer_rate_flag() -> Arg {
  Arg::new(CONSUMER_RATE_FLAG)
    .long(CONSUMER_RATE_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1048576..=1250000000))
    .value_name("RATE")
    .help("Limit for consumer rate")
    .long_help(
      "Set the limit for the maximum allowed consumer rate (bytes/sec). \
          The value must be greater than or equal to 1048576 \
          and lower than or equal to 1250000000.",
    )
}

pub(crate) fn cpu_flag() -> Arg {
  Arg::new(CPU_FLAG)
    .long(CPU_FLAG)
    .action(ArgAction::Set)
    .value_parser(clap::value_parser!(f64))
    .value_name("CPUS")
    .help("Limit for number of cpus")
    .long_help(
      "Set the limit for the number of cpus to provision for the managed tenant \
          (factions of a vCPU core, 1.0 equals 1 vCPU). \
          The value must be greater than or equal to 0.01 \
          and lower than or equal to 16.0.",
    )
}

pub(crate) fn kafka_acl_group_flag() -> Arg {
  Arg::new(KAFKA_ACL_GROUP_COUNT_FLAG)
    .long(KAFKA_ACL_GROUP_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(0..=50))
    .value_name("COUNT")
    .help("Limit for number of kafka acl groups")
    .long_help(
      "Set the limit for the number of Kafka ACL groups available for the managed tenant. \
          The value must be greater than or equal to 0 and lower than or equal to 50.",
    )
}

pub(crate) fn mem_flag() -> Arg {
  Arg::new(MEM_FLAG)
    .long(MEM_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..=131072))
    .value_name("MEM")
    .help("Limit for amount of memory")
    .long_help(
      "Set the limit for the amount of memory available for the managed tenant (MiB). \
          The value must be greater than or equal to 1 and lower than or equal to 131072.",
    )
}

pub(crate) fn partition_count_flag() -> Arg {
  Arg::new(PARTITION_COUNT_FLAG)
    .long(PARTITION_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Limit for number of partitions")
    .long_help(
      "Set the limit for the number of partitions available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn producer_rate_flag() -> Arg {
  Arg::new(PRODUCER_RATE_FLAG)
    .long(PRODUCER_RATE_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1048576..=1250000000))
    .value_name("RATE")
    .help("Limit for producer rate")
    .long_help(
      "Set the limit for the maximum allowed producer rate (bytes/sec). \
          The value must be greater than or equal to 1048576 \
          and lower than or equal to 1250000000.",
    )
}

pub(crate) fn request_rate_flag() -> Arg {
  Arg::new(REQUEST_RATE_FLAG)
    .long(REQUEST_RATE_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..=100))
    .value_name("RATE")
    .help("Limit for request rate")
    .long_help(
      "Set the limit for the maximum allowed request rate (%). \
          The value must be greater than or equal to 1 and lower than or equal to 100.",
    )
}

pub(crate) fn secret_count_flag() -> Arg {
  Arg::new(SECRET_COUNT_FLAG)
    .long(SECRET_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Limit for number of secrets")
    .long_help(
      "Set the limit for the number of secrets available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn topic_count_flag() -> Arg {
  Arg::new(TOPIC_COUNT_FLAG)
    .long(TOPIC_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<i64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Limit for number of topics")
    .long_help(
      "Set the limit for the number of topics available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn tracing_flag() -> Arg {
  Arg::new(TRACING_FLAG)
    .long(TRACING_FLAG)
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
  Arg::new(VPN_FLAG)
    .long(VPN_FLAG)
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
