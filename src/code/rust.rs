use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, EXAMPLE_LIST_TOPICS, EXAMPLE_PRODUCER, LANGUAGE_RUST};
use crate::context::Context;
use crate::proxy_bundles::ProxyCertificateBundleConfig;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

pub(crate) fn generate_rust_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_RUST, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let src_directory = format!("{}/src", &example_directory);
  create_dir_all(&src_directory)?;
  context.print_outcome(format!("created directory '{}'", src_directory));

  let bin_directory = format!("{}/src/bin", &example_directory);
  create_dir_all(&bin_directory)?;
  context.print_outcome(format!("created directory '{}'", bin_directory));

  let cargo_toml = apply_template(CARGO_TOML_TEMPLATE, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "")?;
  let cargo_toml_filename = format!("{}/Cargo.toml", example_directory);
  fs::write(&cargo_toml_filename, &cargo_toml)?;
  context.print_outcome(format!("created file '{}'", cargo_toml_filename));

  let main_rs = apply_template(MAIN_RS_TEMPLATE_LIST_TOPICS, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "  ")?;
  let main_rs_filename = format!("{}/src/main.rs", example_directory);
  fs::write(&main_rs_filename, &main_rs)?;
  context.print_outcome(format!("created file '{}'", main_rs_filename));

  let consumer_rs = apply_template(BIN_CONSUMER_RS_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "  ")?;
  let consumer_rs_filename = format!("{}/src/bin/consumer.rs", example_directory);
  fs::write(&consumer_rs_filename, &consumer_rs)?;
  context.print_outcome(format!("created file '{}'", consumer_rs_filename));

  let producer_rs = apply_template(BIN_PRODUCER_RS_TEMPLATE, EXAMPLE_PRODUCER, bundle_configuration, bundle_directory, "  ")?;
  let producer_rs_filename = format!("{}/src/bin/producer.rs", example_directory);
  fs::write(&producer_rs_filename, &producer_rs)?;
  context.print_outcome(format!("created file '{}'", producer_rs_filename));

  Ok(Some(example_directory))
}

pub(crate) fn rust_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_RUST, bundle_configuration, context))?)
}

pub(crate) fn delete_rust_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_RUST, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted rust example directory '{}'", example_directory));
  Ok(())
}

const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{{example}}"
version = "0.1.0"
edition = "2024"

[dependencies]
ctrlc = "3"
rdkafka = { version = "0.39", features = ["ssl-vendored"], default-features = false }
"#;

const BIN_CONSUMER_RS_TEMPLATE: &str = r#"use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use std::env::args;
use std::time::Duration;
use std::{process, thread};

const PKI_DIRECTORY: &str = "{{bundle-directory}}";
const CLIENT_ID: &str = "{{client-id}}";
const GROUP_ID: &str = "{{group-id}}";
const BROKERS: [&str; 3] = [
{{brokers}}
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Allow handling of ctrl-c
  set_handler(|| process::exit(0))?;

  let args: Vec<String> = args().collect();
  let topic = args.get(1).ok_or("missing topic argument")?;

  let mut kafka_client_config = ClientConfig::new();
  kafka_client_config
    .set("auto.offset.reset", "latest")
    .set("bootstrap.servers", BROKERS.join(","))
    .set("client.id", CLIENT_ID)
    .set("group.id", GROUP_ID)
    .set("security.protocol", "ssl")
    .set("ssl.ca.location", format!("{PKI_DIRECTORY}/ca.pem"))
    .set("ssl.certificate.location", format!("{PKI_DIRECTORY}/client.pem"))
    .set("ssl.key.location", format!("{PKI_DIRECTORY}/client.key"));

  let consumer: BaseConsumer = kafka_client_config.create().map_err(|error| format!("failed to create consumer: {error}"))?;

  consumer
    .subscribe(&[topic])
    .map_err(|error| format!("failed to subscribe to topic '{topic}': {error}"))?;

  loop {
    match consumer.poll(Duration::ZERO) {
      Some(Ok(message)) => println!("{}:{} {}", message.partition(), message.offset(), String::from_utf8_lossy(message.key().unwrap())),
      Some(Err(error)) => println!("error: {error}"),
      None => {}
    }
    thread::sleep(Duration::from_millis(10));
  }
}
"#;

const MAIN_RS_TEMPLATE_LIST_TOPICS: &str = r#"use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use std::time::Duration;

const PKI_DIRECTORY: &str = "{{bundle-directory}}";
const CLIENT_ID: &str = "{{client-id}}";
const GROUP_ID: &str = "{{group-id}}";
const BROKERS: [&str; 3] = [
{{brokers}}
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut kafka_client_config = ClientConfig::new();
  kafka_client_config
    .set("auto.offset.reset", "latest")
    .set("bootstrap.servers", BROKERS.join(","))
    .set("client.id", CLIENT_ID)
    .set("group.id", GROUP_ID)
    .set("security.protocol", "ssl")
    .set("ssl.ca.location", format!("{PKI_DIRECTORY}/ca.pem"))
    .set("ssl.certificate.location", format!("{PKI_DIRECTORY}/client.pem"))
    .set("ssl.key.location", format!("{PKI_DIRECTORY}/client.key"));

  let consumer: BaseConsumer = kafka_client_config
    .create()
    .map_err(|error| format!("failed to create consumer: {error}"))?;

  let metadata = consumer.fetch_metadata(None, Duration::from_millis(2000)).unwrap();
  let mut topics = metadata
    .topics()
    .iter()
    .map(|topic| (topic.name(), topic.partitions().len()))
    .collect::<Vec<_>>();

  topics.sort_by(|(topic_a, _), (topic_b, _)| topic_a.to_string().cmp(&topic_b.to_string()));
  for (topic, partitions) in topics {
    println!("{} ({})", topic, partitions);
  }

  consumer.unsubscribe();
  Ok(())
}
"#;

const BIN_PRODUCER_RS_TEMPLATE: &str = r#"use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{BaseProducer, BaseRecord};
use std::env::args;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{process, thread};

const PKI_DIRECTORY: &str = "{{bundle-directory}}";
const CLIENT_ID: &str = "{{client-id}}";
const BROKERS: [&str; 3] = [
{{brokers}}
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Allow handling of ctrl-c
  set_handler(|| process::exit(0))?;

  // Get topic name from command line argument.
  let args: Vec<String> = args().collect();
  let topic = args.get(1).ok_or("missing topic argument")?;

  // Set Kafka client configuration
  let mut kafka_client_config = ClientConfig::new();
  kafka_client_config
    .set("bootstrap.servers", BROKERS.join(","))
    .set("client.id", CLIENT_ID)
    .set("security.protocol", "ssl")
    .set("ssl.ca.location", format!("{PKI_DIRECTORY}/ca.pem"))
    .set("ssl.certificate.location", format!("{PKI_DIRECTORY}/client.pem"))
    .set("ssl.key.location", format!("{PKI_DIRECTORY}/client.key"));

  let producer: BaseProducer = kafka_client_config.create().map_err(|error| format!("failed to create producer: {error}"))?;
  loop {
    let key = format!("{{proxy-name}}-{{example}}-rust: {}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    let record = BaseRecord::to(topic).key(&key).payload("payload");
    match producer.send(record) {
      Ok(_) => println!("{}", key),
      Err((error, _)) => println!("error: {error}"),
    }
    thread::sleep(Duration::from_millis(1000));
  }
}
"#;
