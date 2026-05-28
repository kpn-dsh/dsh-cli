# Code example `rust consumer`

For this example we will create a `consumer` example for the `rust` programming language:

```shell
> dsh proxy code my-proxy rust consumer
generating rust consumer example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-consumer-rust-example'
created directory 'my-proxy-consumer-rust-example/src'
created file 'my-proxy-consumer-rust-example/src/main.rs'
created file 'my-proxy-consumer-rust-example/Cargo.toml'
rust code for bundle 'my-proxy' generated in directory 'my-proxy-consumer-rust-example'
```

As is shown in the output of the command, the example is generated in a newly created directory
which contains a `Cargo.toml` manifest and a `src/main.rs` binary module.

## `Cargo.toml`

```toml
[package]
name = "my-proxy-consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
ctrlc = "3"
rdkafka = { version = "0.39", features = ["ssl-vendored"], default-features = false }
```

## `src/main.rs`

```rust
use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use std::env::args;
use std::time::Duration;
use std::{process, thread};

const PKI_DIRECTORY: &str = "/Users/wilbert/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy";
const CLIENT_ID: &str = "my-tenant";
const GROUP_ID: &str = "my-tenant_my-proxy_1";
const BROKERS: [&str; 3] = [
  "my-proxy-0.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
  "my-proxy-1.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
  "my-proxy-2.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
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
```

To build the example, we first change to the created directory and use `cargo build` to build
the executable. This will take a few minutes due to dependencies on `rdkafka` and (indirect)
`openssl`.

```shell
> cd my-proxy-consumer-rust-example
> cargo build
```

Now finally we can run the executable and receive messages from a Kafka topic. In the example below
we use the same topic as for the [`rust producer`](rust-producer.md) example
(`scratch.example.my-tenant`). Use `ctrl-c` to stop the program.

```shell
> target/debug/my-proxy-consumer scratch.example.my-tenant
0:119 timestamp: 1779992878
0:120 timestamp: 1779992879
0:121 timestamp: 1779992880
0:122 timestamp: 1779992881
^C
```
