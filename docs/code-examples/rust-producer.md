# Code example `rust producer`

For this example we will create a `producer` example for the `rust` programming language:

```shell
> dsh proxy code my-proxy rust producer
generating rust producer example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-producer-example-rust'
created directory 'my-proxy-producer-example-rust/src'
created file 'my-proxy-producer-example-rust/src/main.rs'
created file 'my-proxy-producer-example-rust/Cargo.toml'
rust code for bundle 'my-proxy' generated in directory 'my-proxy-producer-example-rust'
```

As is shown in the output of the command, the example is generated in a newly created directory
which contains a `Cargo.toml` manifest and a `src/main.rs` binary module.

## `Cargo.toml`

```toml
[package]
name = "my-proxy-producer"
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
  // Listen to ctrl-c.
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
    let key = format!("timestamp: {}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    let record = BaseRecord::to(topic).key(&key).payload("payload");
    match producer.send(record) {
      Ok(_) => println!("{}", key),
      Err((error, _)) => println!("error: {error}"),
    }
    thread::sleep(Duration::from_millis(1000));
  }
}
```

To build the example, we first change to the created directory and use `cargo build` to build
the executable. This will take a few minutes due to dependencies on `rdkafka` and (indirect)
`openssl`.

```shell
> cd my-proxy-producer-example-rust
> cargo build
```

Now finally we can run the executable and send messages to a Kafka topic. In the example below
we use the topic `scratch.example.my-tenant` for this. Use `ctrl-c` to stop the program.

```shell
> target/debug/my-proxy-producer scratch.example.my-tenant
timestamp: 1779992760
timestamp: 1779992761
timestamp: 1779992762
timestamp: 1779992763
^C
```
