use crate::code::{EXAMPLE_CONSUMER, EXAMPLE_PRODUCER, EXAMPLE_TOPICS, LANGUAGE_PYTHON, LANGUAGE_RUST};
use crate::context::Context;
use crate::{err, DshCliResult};
use clap::builder::PossibleValue;
use clap::{builder, Arg, ArgAction, ArgMatches};
use dsh_api::platform::VhostZone;
use std::str::FromStr;
use whoami::username;

pub(crate) const CA_COMMON_NAME_OPTION: &str = "ca-common-name-option";

pub(crate) fn ca_common_name_option() -> Arg {
  Arg::new(CA_COMMON_NAME_OPTION)
    .long("ca-common-name")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("NAME")
    .help("Certificate authority common name")
    .long_help("This option specifies the common name used to create certificate authority certificate.")
}

pub(crate) fn get_ca_common_name(matches: &ArgMatches, context: &Context) -> DshCliResult<String> {
  match matches.get_one::<String>(CA_COMMON_NAME_OPTION) {
    Some(ca_common_name) => Ok(ca_common_name.to_string()),
    None => {
      let default_username = username()?;
      let ca_common_name = context.read_single_line(format!("certificate authority common name [{}]", default_username))?;
      if ca_common_name.is_empty() {
        Ok(default_username)
      } else {
        Ok(ca_common_name)
      }
    }
  }
}

pub(crate) const ENABLE_SCHEMA_STORE_OPTION: &str = "enable-schema-store-option";

pub(crate) fn enable_schema_store_option() -> Arg {
  Arg::new(ENABLE_SCHEMA_STORE_OPTION)
    .long("enable-schema-store")
    .action(ArgAction::Set)
    .value_parser(builder::BoolValueParser::new())
    .value_name("BOOL")
    .help("Enable schema store")
    .long_help(
      "If this option is enabled the created certificates will include a dns entry \
    for a schema store.",
    )
}

pub(crate) const NUMBER_OF_DNS_RECORDS_OPTION: &str = "number-of-dns-records-option";

pub(crate) fn number_of_dns_records_option() -> Arg {
  Arg::new(NUMBER_OF_DNS_RECORDS_OPTION)
    .long("number-of-dns-records")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<usize>::new().range(1..11))
    .value_name("NUMBER")
    .help("Number of dns records")
    .long_help(
      "Number of dns records that will be generated in the proxy. Do not use this \
         option unless you know what you are doing.",
    )
}

pub(crate) fn get_number_of_dns_records(matches: &ArgMatches, context: &Context) -> DshCliResult<usize> {
  match matches.get_one::<usize>(NUMBER_OF_DNS_RECORDS_OPTION) {
    Some(number_of_dns_records) if *number_of_dns_records < 10 => {
      context.print_warning("the number of dns records should almost always be set to the default value of 10");
      if context.confirmed(format!("are you sure you want to set the number of dns records to {}?", number_of_dns_records))? {
        Ok(*number_of_dns_records)
      } else {
        err!("cancelled")
      }
    }
    _ => Ok(10),
  }
}

pub(crate) const VHOST_ZONE_OPTION: &str = "vhost-zone-option";

pub(crate) fn vhost_zone_option() -> Arg {
  let possible_values = [PossibleValue::new("private").help("Private vhost"), PossibleValue::new("public").help("Public vhost")];
  Arg::new(VHOST_ZONE_OPTION)
    .long("vhost-zone")
    .action(ArgAction::Set)
    .value_parser(possible_values)
    .value_name("ZONE")
    .help("Vhost zone")
    .long_help("This option indicates whether the certificates will be created for a public or a private vhost.")
}

pub(crate) fn get_vhost_zone(matches: &ArgMatches, context: &Context) -> DshCliResult<VhostZone> {
  match matches.get_one::<String>(VHOST_ZONE_OPTION) {
    Some(vhost_zone) => Ok(VhostZone::from_str(vhost_zone)?),
    None => {
      let vhost_zone_string = context.read_single_line("vhost zone [PRIVATE/public]")?;
      if vhost_zone_string.is_empty() {
        Ok(VhostZone::Private)
      } else {
        Ok(VhostZone::from_str(&vhost_zone_string)?)
      }
    }
  }
}

pub(crate) const LANGUAGE_ARGUMENT: &str = "language-argument";

pub(crate) fn language_argument() -> Arg {
  Arg::new(LANGUAGE_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::PossibleValuesParser::new(vec![
      PossibleValue::new(LANGUAGE_PYTHON),
      PossibleValue::new(LANGUAGE_RUST),
    ]))
    .value_name("LANGUAGE")
    .help("Programming language")
    .long_help("Identifies the programming language used by the code examples.")
}

pub(crate) const EXAMPLE_ARGUMENT: &str = "example-argument";

pub(crate) fn example_argument() -> Arg {
  Arg::new(EXAMPLE_ARGUMENT)
    .action(ArgAction::Set)
    .value_parser(builder::PossibleValuesParser::new(vec![
      PossibleValue::new(EXAMPLE_CONSUMER),
      PossibleValue::new(EXAMPLE_PRODUCER),
      PossibleValue::new(EXAMPLE_TOPICS),
    ]))
    .value_name("EXAMPLE")
    .help("Generated example")
    .long_help("Identifies which example program is generated.")
}
