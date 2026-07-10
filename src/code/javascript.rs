use crate::bundle::ProxyCertificateBundleConfig;
use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, EXAMPLE_LIST_TOPICS, EXAMPLE_PRODUCER, LANGUAGE_JAVASCRIPT};
use crate::context::Context;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

pub(crate) fn generate_javascript_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_JAVASCRIPT, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let package_json = apply_template(JAVASCRIPT_TEMPLATE_PACKAGE_JSON, "", bundle_configuration, bundle_directory, "")?;
  let package_json_filename = format!("{}/package.json", example_directory);
  fs::write(&package_json_filename, &package_json)?;
  context.print_outcome(format!("created file '{}'", package_json_filename));

  let javascript_consumer = apply_template(JAVASCRIPT_TEMPLATE_CONSUMER, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "  ")?;
  let javascript_consumer_filename = format!("{}/{}.js", example_directory, EXAMPLE_CONSUMER);
  fs::write(&javascript_consumer_filename, &javascript_consumer)?;
  context.print_outcome(format!("created file '{}'", javascript_consumer_filename));

  let javascript_list_topics = apply_template(JAVASCRIPT_TEMPLATE_LIST_TOPICS, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "  ")?;
  let javascript_list_topics_filename = format!("{}/{}.js", example_directory, EXAMPLE_LIST_TOPICS);
  fs::write(&javascript_list_topics_filename, &javascript_list_topics)?;
  context.print_outcome(format!("created file '{}'", javascript_list_topics_filename));

  let javascript_producer = apply_template(JAVASCRIPT_TEMPLATE_PRODUCER, EXAMPLE_PRODUCER, bundle_configuration, bundle_directory, "  ")?;
  let javascript_producer_filename = format!("{}/{}.js", example_directory, EXAMPLE_PRODUCER);
  fs::write(&javascript_producer_filename, &javascript_producer)?;
  context.print_outcome(format!("created file '{}'", javascript_producer_filename));

  Ok(Some(example_directory))
}

pub(crate) fn javascript_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_JAVASCRIPT, bundle_configuration, context))?)
}

pub(crate) fn delete_javascript_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_JAVASCRIPT, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted javascript example directory '{}'", example_directory));
  Ok(())
}

const JAVASCRIPT_TEMPLATE_PACKAGE_JSON: &str = r#"{
  "dependencies": {
    "@confluentinc/kafka-javascript": "1.9.x",
    "process": "0.11.x"
  }
}
"#;

const JAVASCRIPT_TEMPLATE_CONSUMER: &str = r#"const {Kafka, logLevel} = require('@confluentinc/kafka-javascript').KafkaJS;
const process = require('process');

const pkiDirectory = "{{bundle-directory}}";
const clientId = "{{client-id}}";
const groupId = "{{group-id}}";
const brokers = [
{{brokers}}
];

async function main() {
  // Allow handling of ctrl-c
  process.on('SIGTERM', () => handle_termination('SIGTERM'));
  process.on('SIGINT', () => handle_termination('SIGINT'));

  let topic = process.argv[2];
  if (!topic) {
    console.error("missing topic argument");
  }

  const kafkaConfig = {
    kafkaJS: {
      clientId,
      brokers,
      ssl: true,
      logLevel: logLevel.ERROR
    },
    "ssl.ca.location": pkiDirectory + "/ca.pem",
    "ssl.certificate.location": pkiDirectory + "/client.pem",
    "ssl.key.location": pkiDirectory + "/client.key"
  };

  const kafka = new Kafka(kafkaConfig);
  let consumer = kafka.consumer({
    kafkaJS: { groupId }
  });
  await consumer.connect();
  await consumer.subscribe({topic: topic});

  await consumer.run({
    eachMessage: async ({_, partition, message}) => {
      console.log(partition + ':' + message.offset + ' ' + message.key.toString());
    },
  });
}

function handle_termination(signal) {
  console.log(signal);
  process.exit();
}

if (require.main === module) {
  main().catch(console.error);
}
"#;

const JAVASCRIPT_TEMPLATE_LIST_TOPICS: &str = r#"const {Kafka, logLevel} = require('@confluentinc/kafka-javascript').KafkaJS;
const process = require('process');

const pkiDirectory = "{{bundle-directory}}";
const clientId = "{{client-id}}";
const groupId = "{{group-id}}";
const brokers = [
{{brokers}}
];

async function main() {
  const kafkaConfig = {
    kafkaJS: {
      clientId,
      brokers,
      ssl: true,
      logLevel: logLevel.ERROR
    },
    "ssl.ca.location": pkiDirectory + "/ca.pem",
    "ssl.certificate.location": pkiDirectory + "/client.pem",
    "ssl.key.location": pkiDirectory + "/client.key"
  };

  const kafka = new Kafka(kafkaConfig);
  let consumer = kafka.consumer({
    kafkaJS: { groupId }
  });
  await consumer.connect();
  let admin = consumer.dependentAdmin();
  await admin.connect();
  let topics = await admin.listTopics();
  topics.sort();
  topics.forEach(topic => console.log(topic));
  process.exit();
}

if (require.main === module) {
    main().catch(console.error);
}
"#;

const JAVASCRIPT_TEMPLATE_PRODUCER: &str = r#"const {Kafka, logLevel} = require('@confluentinc/kafka-javascript').KafkaJS;
const process = require('process');

const pkiDirectory = "{{bundle-directory}}";
const clientId = "{{client-id}}";
const brokers = [
{{brokers}}
];

let producerInterval = 0;

async function main() {
  // Allow handling of ctrl-c
  process.on('SIGTERM', () => handle_termination('SIGTERM'));
  process.on('SIGINT', () => handle_termination('SIGINT'));

  let topic = process.argv[2];
  if (!topic) {
    console.error("missing topic argument");
  }

  const kafkaConfig = {
    kafkaJS: {
      clientId,
      brokers,
      ssl: true,
      logLevel: logLevel.ERROR
    },
    "ssl.ca.location": pkiDirectory + "/ca.pem",
    "ssl.certificate.location": pkiDirectory + "/client.pem",
    "ssl.key.location": pkiDirectory + "/client.key"
  };

  const kafka = new Kafka(kafkaConfig);
  let producer = kafka.producer({
    kafkaJS: {}
  });
  await producer.connect();

  producerInterval = setInterval(async () => {
    const key = "{{proxy-name}}-{{example}}-javascript: " + Math.round(new Date().getTime() / 1000);
    const message = {
      key,
      value: "payload",
    };
    await producer.send({
      topic,
      messages: [message],
    });
    console.log(key);
  }, 1000)
}

function handle_termination(signal) {
  console.log(signal);
  if (producerInterval) {
    clearInterval(producerInterval);
  }
  process.exit();
}

if (require.main === module) {
  main().catch(console.error);
}
"#;
