# `Rust` code example

> It is assumed that you have a recent version of `Rust` and its `cargo` package manager
> installed.  
> The generated code has a dependency on the [rust-rdkafka](https://crates.io/crates/rdkafka) crate.

For this example we will create example code for the `Rust` programming language:

```shell
> dsh proxy code my-proxy rust
generating rust example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-example-rust'
created directory 'my-proxy-example-rust/src'
created directory 'my-proxy-example-rust/src/bin'
created file 'my-proxy-example-rust/Cargo.toml'
created file 'my-proxy-example-rust/src/main.rs'
created file 'my-proxy-example-rust/src/bin/consumer.rs'
created file 'my-proxy-example-rust/src/bin/producer.rs'
rust code for bundle 'my-proxy' generated in directory 'my-proxy-example-rust'
```

As is shown in the output of the command, the example is generated in a newly created directory
which contains a `Cargo.toml` manifest, a `src/main.rs` binary module and two additional binaries
`src/bin/consumer.rs` and `src/bin/producer.rs`. Click below to see the generated code.

<details>
<summary><code>Cargo.toml</code></summary>

```toml
[package]
name = "my-proxy-list-topic"
version = "0.1.0"
edition = "2024"

[dependencies]
ctrlc = "3"
rdkafka = { version = "0.39", features = ["ssl-vendored"], default-features = false }
```

</details>

<details>
<summary><code>src/main.rs</code></summary>

```rust
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use std::time::Duration;

const PKI_DIRECTORY: &str = "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy";
const CLIENT_ID: &str = "my-tenant";
const GROUP_ID: &str = "my-tenant_my-proxy_1";
const BROKERS: [&str; 3] = [
  "my-proxy-0.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
  "my-proxy-1.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
  "my-proxy-2.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
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
```

</details>

<details>
<summary><code>src/bin/consumer.rs</code></summary>

```rust
use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::message::Message;
use std::env::args;
use std::time::Duration;
use std::{process, thread};

const PKI_DIRECTORY: &str = "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy";
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

  let consumer: BaseConsumer =
    kafka_client_config
      .create()
      .map_err(|error| format!("failed to create consumer: {error}"))?;

  consumer
    .subscribe(&[topic])
    .map_err(|error| format!("failed to subscribe to topic '{topic}': {error}"))?;

  loop {
    match consumer.poll(Duration::ZERO) {
      Some(Ok(message)) => println!(
        "{}:{} {}",
        message.partition(),
        message.offset(),
        String::from_utf8_lossy(message.key().unwrap())
      ),
      Some(Err(error)) => println!("error: {error}"),
      None => {}
    }
    thread::sleep(Duration::from_millis(10));
  }
}
```

</details>

<details>
<summary><code>src/bin/producer.rs</code></summary>

```rust
use ctrlc::set_handler;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{BaseProducer, BaseRecord};
use std::env::args;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{process, thread};

const PKI_DIRECTORY: &str = "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy";
const CLIENT_ID: &str = "my-tenant";
const BROKERS: [&str; 3] = [
  "my-proxy-0.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
  "my-proxy-1.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
  "my-proxy-2.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
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

  let producer: BaseProducer =
    kafka_client_config
      .create()
      .map_err(|error| format!("failed to create producer: {error}"))?;

  loop {
    let key = format!("my-proxy-producer-rust: {}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    let record = BaseRecord::to(topic).key(&key).payload("payload");
    match producer.send(record) {
      Ok(_) => println!("{}", key),
      Err((error, _)) => println!("error: {error}"),
    }
    thread::sleep(Duration::from_millis(1000));
  }
}
```

</details>

To build the example, we first change to the created directory and then use `cargo build` to build
the executables. This will take a few minutes due to the dependency on `rdkafka`, which depends on
`openssl`. The executables will be written to the `target/debug` directory.

```shell
> cd my-proxy-consumer-example-rust
> cargo build
```

Now we are able to run the executables. First we will list all Kafka topics that we have
read access to.

```shell
> target/debug/my-proxy-list-topic
...
scratch.my-topic.my-tenant
...
```

Then we will send messages to a Kafka topic. In the example below we use topic
`scratch.my-topic.my-tenant`. Use `ctrl-c` to stop the program.

```shell
> target/debug/producer scratch.my-topic.my-tenant
my-proxy-producer-rust: 1780047895
my-proxy-producer-rust: 1780047896
my-proxy-producer-rust: 1780047897
my-proxy-producer-rust: 1780047898
^C
```

Next we will consume records from the same topic. Again, use `ctrl-c` to stop the program.

```shell
> target/debug/consumer scratch.my-topic.my-tenant
0:2808 my-proxy-producer-rust: 1780047895
0:2809 my-proxy-producer-rust: 1780047896
0:2810 my-proxy-producer-rust: 1780047897
0:2811 my-proxy-producer-rust: 1780047898
^C
```

[&#x2190; Kafka proxy](../proxy.md)
