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

pub(crate) fn certificate_count_flag() -> Arg {
  Arg::new(CERTIFICATE_COUNT_FLAG)
    .long(CERTIFICATE_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Number of certificates")
    .long_help(
      "The number of certificates available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn consumer_rate_flag() -> Arg {
  Arg::new(CONSUMER_RATE_FLAG)
    .long(CONSUMER_RATE_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1048576..=1250000000))
    .value_name("RATE")
    .help("Consumer rate")
    .long_help(
      "The maximum allowed consumer rate (bytes/sec). \
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
    .help("Number of cpus")
    .long_help(
      "The number of cpus to provision for the managed tenant \
          (factions of a vCPU core, 1.0 equals 1 vCPU). \
          The value must be greater than or equal to 0.01 \
          and lower than or equal to 16.0, and a multiple of 0.01.",
    )
}

// TODO Not available in generated code
// pub(crate) fn kafka_acl_group_flag() -> Arg {
//   Arg::new(KAFKA_ACL_GROUP_COUNT_FLAG)
//     .long(KAFKA_ACL_GROUP_COUNT_FLAG)
//     .action(ArgAction::Set)
//     .value_parser(builder::RangedU64ValueParser::<u64>::new().range(0..=50))
//     .value_name("COUNT")
//     .help("Number of kafka acl groups")
//     .long_help(
//       "The number of Kafka ACL groups available for the managed tenant. \
//        The value must be greater than or equal to 0 and lower than or equal to 50.")
// }

pub(crate) fn mem_flag() -> Arg {
  Arg::new(MEM_FLAG)
    .long(MEM_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=131072))
    .value_name("MEM")
    .help("Amount of memory")
    .long_help(
      "The amount of memory available for the managed tenant (MiB). \
          The value must be greater than or equal to 1 and lower than or equal to 131072.",
    )
}

pub(crate) fn partition_count_flag() -> Arg {
  Arg::new(PARTITION_COUNT_FLAG)
    .long(PARTITION_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Number of partitions")
    .long_help(
      "The number of partitions available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn producer_rate_flag() -> Arg {
  Arg::new(PRODUCER_RATE_FLAG)
    .long(PRODUCER_RATE_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1048576..=1250000000))
    .value_name("RATE")
    .help("Producer rate")
    .long_help(
      "The maximum allowed producer rate (bytes/sec). \
          The value must be greater than or equal to 1048576 \
          and lower than or equal to 1250000000.",
    )
}

pub(crate) fn request_rate_flag() -> Arg {
  Arg::new(REQUEST_RATE_FLAG)
    .long(REQUEST_RATE_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=100))
    .value_name("RATE")
    .help("Request rate")
    .long_help(
      "The maximum allowed request rate (%). \
          The value must be greater than or equal to 1 and lower than or equal to 100.",
    )
}

pub(crate) fn secret_count_flag() -> Arg {
  Arg::new(SECRET_COUNT_FLAG)
    .long(SECRET_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Number of secrets")
    .long_help(
      "The number of secrets available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}

pub(crate) fn topic_count_flag() -> Arg {
  Arg::new(TOPIC_COUNT_FLAG)
    .long(TOPIC_COUNT_FLAG)
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<u64>::new().range(1..=40))
    .value_name("COUNT")
    .help("Number of topics")
    .long_help(
      "The number of topics available for the managed tenant. \
          The value must be greater than or equal to 1 and lower than or equal to 40.",
    )
}
