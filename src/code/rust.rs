use crate::code::apply_template;
use crate::context::Context;
use crate::proxy_bundles::ProxyCertificateBundleConfig;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

fn rust_example_directory(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> String {
  match context.output_directory() {
    Some(output_directory) => format!("{}/{}-rust-example", output_directory.display(), bundle_configuration.proxy_name),
    None => format!("{}-rust-example", bundle_configuration.proxy_name),
  }
}

pub(crate) fn generate_rust_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<String> {
  let example_directory = rust_example_directory(bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let src_directory = format!("{}/src", &example_directory);
  create_dir_all(&src_directory)?;
  context.print_outcome(format!("created directory '{}'", src_directory));

  let rust_rs = apply_template(MAIN_RS_TEMPLATE, bundle_configuration, bundle_directory)?;
  let rust_rs_filename = format!("{}/src/main.rs", example_directory);
  fs::write(&rust_rs_filename, &rust_rs)?;
  context.print_outcome(format!("created file '{}'", rust_rs_filename));

  let cargo_toml = apply_template(CARGO_TOML_TEMPLATE, bundle_configuration, bundle_directory)?;
  let cargo_toml_filename = format!("{}/Cargo.toml", example_directory);
  fs::write(&cargo_toml_filename, &cargo_toml)?;
  context.print_outcome(format!("created file '{}'", cargo_toml_filename));
  Ok(example_directory)
}

pub(crate) fn rust_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(rust_example_directory(bundle_configuration, context))?)
}

pub(crate) fn delete_rust_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = rust_example_directory(bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted rust example directory '{}'", example_directory));
  Ok(())
}

const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{{proxy-name}}-consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
ctrlc = "3"
rdkafka = { version = "0.39", features = ["ssl-vendored"] }
"#;

const MAIN_RS_TEMPLATE: &str = r#"use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use std::env::args;
use std::time::Duration;
use std::{process, thread};

const PKI_CLIENT_KEY_LOCATION: &str = "{{bundle-directory}}/client.key";
const PKI_CLIENT_CERTIFICATE_LOCATION: &str = "{{bundle-directory}}/client.pem";
const PKI_CA_CERTIFICATE_LOCATION: &str = "{{bundle-directory}}/ca.pem";

const CLIENT_ID: &str = "{{client-id}}";
const GROUP_ID: &str = "{{group-id}}";
const BROKERS: &str =
  "{{brokers}}";

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _ = set_handler(move || {
    eprintln!("interrupted");
    process::exit(0);
  });

  let args: Vec<String> = args().collect();
  let topic = args.get(1).ok_or("missing topic argument")?;

  let mut kafka_client_config = ClientConfig::new();
  kafka_client_config
    .set("auto.offset.reset", "latest")
    .set("bootstrap.servers", BROKERS)
    .set("client.id", CLIENT_ID)
    .set("group.id", GROUP_ID)
    .set("security.protocol", "ssl")
    .set("ssl.ca.location", PKI_CA_CERTIFICATE_LOCATION)
    .set("ssl.certificate.location", PKI_CLIENT_CERTIFICATE_LOCATION)
    .set("ssl.key.location", PKI_CLIENT_KEY_LOCATION);

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
