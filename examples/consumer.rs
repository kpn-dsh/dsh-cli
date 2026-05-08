use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use std::env::args;
use std::time::Duration;
use std::{process, thread};

const PKI_CLIENT_KEY_LOCATION: &str = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/vol/client.key";
const PKI_CLIENT_CERTIFICATE_LOCATION: &str = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/vol/client.pem";
const PKI_CA_CERTIFICATE_LOCATION: &str = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev/bundles/vol/ca.pem";

const CLIENT_ID: &str = "greenbox-dev";
const GROUP_ID: &str = "greenbox-dev_vol_0";
const BROKERS: &str =
  "vol-0.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091,vol-1.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091,vol-2.kafka.greenbox-dev.dsh-dev.dsh.np.aws.kpn.org:9091";

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
