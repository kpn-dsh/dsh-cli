use crate::code::apply_template;
use crate::context::Context;
use crate::proxy_bundles::ProxyCertificateBundleConfig;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

fn python_example_directory(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> String {
  match context.output_directory() {
    Some(output_directory) => format!("{}/{}-python-example", output_directory.display(), bundle_configuration.proxy_name),
    None => format!("{}-python-example", bundle_configuration.proxy_name),
  }
}

pub(crate) fn generate_python_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<String> {
  let example_directory = python_example_directory(bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let python = apply_template(PYTHON_TEMPLATE, bundle_configuration, bundle_directory)?;
  let python_filename = format!("{}/{}-consumer.py", example_directory, bundle_configuration.proxy_name);
  fs::write(&python_filename, &python)?;
  context.print_outcome(format!("created file '{}'", python_filename));
  Ok(example_directory)
}

pub(crate) fn python_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(python_example_directory(bundle_configuration, context))?)
}

pub(crate) fn delete_python_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = python_example_directory(bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted python example directory '{}'", example_directory));
  Ok(())
}

const PYTHON_TEMPLATE: &str = r#"from confluent_kafka import Consumer
import sys

PKI_DIRECTORY = "{{bundle-directory}}"

CLIENT_ID = "{{client-id}}"
GROUP_ID = "{{group-id}}"
BROKERS = [
{{brokers}}]

def main():
    if len(sys.argv) < 2:
        print("missing topic argument", file=sys.stderr)
        exit(1)
    topic = sys.argv[1]

    kafka_config = {
        "auto.offset.reset": "earliest",
        "bootstrap.servers": ",".join(BROKERS),
        "client.id": CLIENT_ID,
        "group.id": GROUP_ID,
        "security.protocol": "ssl",
        "ssl.ca.location": f"{PKI_DIRECTORY}/ca.pem",
        "ssl.certificate.location": f"{PKI_DIRECTORY}/client.pem",
        "ssl.key.location": f"{PKI_DIRECTORY}/client.key"
    }

    consumer = Consumer(kafka_config)
    consumer.subscribe([topic])

    try:
        while True:
            topic_full = True
            while topic_full:
                msg = consumer.poll(1.0)
                if msg is not None:
                    if msg.key() is not None:
                        print(f"{msg.partition()}:{msg.offset()} {msg.key().decode()}")
                else:
                    topic_full = False
    except KeyboardInterrupt:
        print("interrupted")
    finally:
        if consumer:
            consumer.close()


if __name__ == "__main__":
    main()
"#;
