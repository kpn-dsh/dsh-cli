use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::{BorrowedMessage, Message};
use rdkafka::util::get_rdkafka_version;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;

#[derive(Debug)]
struct Config {
  pki_key: String,
  pki_cert: String,
  pki_ca: String,
  tenant: String,
  topic: String,
  broker_name: String,
  client_id: String,
  group_id: String,
  brokers: Vec<String>,
}

impl Config {
  fn new(bundle_directory: &str) -> Self {
    let path = Path::new(bundle_directory);

    let mut pki_key = path.to_path_buf();
    pki_key.push("client.key");

    let mut pki_cert = path.to_path_buf();
    pki_cert.push("client.pem");

    let mut pki_ca = path.to_path_buf();
    pki_ca.push("ca.pem");

    Config {
      pki_key: pki_key.display().to_string(),
      pki_cert: pki_cert.display().to_string(),
      pki_ca: pki_ca.display().to_string(),
      tenant: "greenbox-dev".to_string(),
      topic: "scratch.reference-implementation-protobuf.greenbox-dev".to_string(),
      broker_name: "volmacht".to_string(),
      client_id: "greenbox-dev".to_string(),
      group_id: "greenbox-dev_volmacht_0".to_string(),
      brokers: vec![
        "volmacht-0.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
        "volmacht-1.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
        "volmacht-2.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091",
      ]
      .iter()
      .map(|s| s.to_string())
      .collect(),
    }
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let config = Config::new("/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/volmacht");
  // let config = Config::new("/Users/wilbert/dsh/self-signed-certs/NPLZ/greenbox-dev-NPLZ/2026-03-10");
  let (version_num, version_str) = get_rdkafka_version();
  println!("rdkafka version: {}.{}", version_num, version_str);

  let running = Arc::new(AtomicBool::new(true));
  let running_clone = running.clone();

  tokio::spawn(async move {
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => {
            println!("received sigterm, exiting...");
        }
        _ = sigint.recv() => {
            println!("received sigint, exiting...");
        }
    }
    running_clone.store(false, Ordering::SeqCst);
  });

  let kafka_config = get_kafka_config(&config);

  println!("{:#?}", kafka_config);

  let consumer: StreamConsumer = kafka_config.create().map_err(|error| format!("failed to create consumer: {}", error))?;

  consumer
    .subscribe(&[&config.topic])
    .map_err(|error| format!("failed to subscribe to topic: {}", error))?;

  while running.load(Ordering::SeqCst) {
    consume_messages(&consumer).await;
    sleep(Duration::from_secs(1)).await;
  }

  println!("shutting down gracefully...");
  Ok(())
}

fn get_kafka_config(config: &Config) -> ClientConfig {
  let mut client_config = ClientConfig::new();
  client_config
    .set("bootstrap.servers", config.brokers.join(","))
    .set("client.id", &config.client_id)
    .set("security.protocol", "ssl")
    .set("ssl.key.location", &config.pki_key)
    .set("ssl.certificate.location", &config.pki_cert)
    .set("ssl.ca.location", &config.pki_ca);
  client_config.set("group.id", &config.group_id).set("auto.offset.reset", "earliest");
  client_config
}

async fn consume_messages(consumer: &StreamConsumer) {
  match consumer.recv().await {
    Ok(message) => {
      process_message(&message);
      if let Err(e) = consumer.commit_message(&message, CommitMode::Async) {
        eprintln!("failed to commit message: {}", e);
      }
    }
    Err(e) => {
      eprintln!("error receiving message: {:#?}", e);
    }
  }
}

fn process_message(message: &BorrowedMessage) {
  println!("received message: {:?}", String::from_utf8_lossy(message.key().unwrap()));
  // match message.payload_view::<str>() {
  //   Some(Ok(payload)) => {
  //     println!("received message: {}", payload);
  //   }
  //   Some(Err(e)) => {
  //     eprintln!("failed to decode message payload: {}", e);
  //   }
  //   None => {
  //     println!("received message with empty payload");
  //   }
  // }
}
