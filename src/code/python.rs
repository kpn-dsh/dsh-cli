use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, EXAMPLE_LIST_TOPICS, EXAMPLE_PRODUCER, LANGUAGE_PYTHON};
use crate::context::Context;
use crate::proxy_bundles::ProxyCertificateBundleConfig;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

pub(crate) fn generate_python_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_PYTHON, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let python_consumer = apply_template(PYTHON_TEMPLATE_CONSUMER, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "    ")?;
  let python_consumer_filename = format!("{}/{}.py", example_directory, EXAMPLE_CONSUMER);
  fs::write(&python_consumer_filename, &python_consumer)?;
  context.print_outcome(format!("created file '{}'", python_consumer_filename));

  let python_list_topics = apply_template(PYTHON_TEMPLATE_LIST_TOPICS, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "    ")?;
  let python_list_topics_filename = format!("{}/{}.py", example_directory, EXAMPLE_LIST_TOPICS);
  fs::write(&python_list_topics_filename, &python_list_topics)?;
  context.print_outcome(format!("created file '{}'", python_list_topics_filename));

  let python_producer = apply_template(PYTHON_TEMPLATE_PRODUCER, EXAMPLE_PRODUCER, bundle_configuration, bundle_directory, "    ")?;
  let python_producer_filename = format!("{}/{}.py", example_directory, EXAMPLE_PRODUCER);
  fs::write(&python_producer_filename, &python_producer)?;
  context.print_outcome(format!("created file '{}'", python_producer_filename));

  Ok(Some(example_directory))
}

pub(crate) fn python_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_PYTHON, bundle_configuration, context))?)
}

pub(crate) fn delete_python_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_PYTHON, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted python example directory '{}'", example_directory));
  Ok(())
}

const PYTHON_TEMPLATE_CONSUMER: &str = r#"from confluent_kafka import Consumer
import sys

PKI_DIRECTORY = "{{bundle-directory}}"
CLIENT_ID = "{{client-id}}"
GROUP_ID = "{{group-id}}"
BROKERS = [
{{brokers}}
]


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

const PYTHON_TEMPLATE_LIST_TOPICS: &str = r#"from confluent_kafka.admin import AdminClient

PKI_DIRECTORY = "{{bundle-directory}}"
CLIENT_ID = "{{client-id}}"
BROKERS = [
{{brokers}}
]


def main():
    kafka_config = {
        "bootstrap.servers": ",".join(BROKERS),
        "client.id": CLIENT_ID,
        "security.protocol": "ssl",
        "ssl.ca.location": f"{PKI_DIRECTORY}/ca.pem",
        "ssl.certificate.location": f"{PKI_DIRECTORY}/client.pem",
        "ssl.key.location": f"{PKI_DIRECTORY}/client.key"
    }

    kafka_admin = AdminClient(kafka_config)
    topics = sorted(kafka_admin.list_topics().topics.values(), key=lambda kv: kv.topic)
    for topic in topics:
        print(f"{topic.topic} ({len(topic.partitions)})")


if __name__ == "__main__":
    main()
"#;

const PYTHON_TEMPLATE_PRODUCER: &str = r#"from confluent_kafka import Producer
from datetime import datetime
import math
import sys
import time

PKI_DIRECTORY = "{{bundle-directory}}"
CLIENT_ID = "{{client-id}}"
BROKERS = [
{{brokers}}
]


def main():
    if len(sys.argv) < 2:
        print("missing topic argument", file=sys.stderr)
        exit(1)
    topic = sys.argv[1]

    kafka_config = {
        "bootstrap.servers": ",".join(BROKERS),
        "client.id": CLIENT_ID,
        "security.protocol": "ssl",
        "ssl.ca.location": f"{PKI_DIRECTORY}/ca.pem",
        "ssl.certificate.location": f"{PKI_DIRECTORY}/client.pem",
        "ssl.key.location": f"{PKI_DIRECTORY}/client.key"
    }

    producer = Producer(kafka_config)

    try:
        while True:
            key = f"{{proxy-name}}-{{example}}-python: {math.floor(datetime.now().timestamp())}"
            producer.produce(topic=topic, key=key)
            producer.flush()
            print(key)
            time.sleep(1)
    except KeyboardInterrupt:
        print("interrupted")
    finally:
        producer.flush()


if __name__ == "__main__":
    main()
"#;
