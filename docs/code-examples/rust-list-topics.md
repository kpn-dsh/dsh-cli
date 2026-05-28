# Code example `rust list-topics`

For this example we will create a `list-topics` example for the `rust` programming language:

```shell
> dsh proxy code my-proxy rust list-topics
generating rust list-topics example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-list-topics-rust-example'
created directory 'my-proxy-list-topics-rust-example/src'
created file 'my-proxy-list-topics-rust-example/src/main.rs'
created file 'my-proxy-list-topics-rust-example/Cargo.toml'
rust code for bundle 'my-proxy' generated in directory 'my-proxy-list-topics-rust-example'
```

As is shown in the output, the example is generated in a newly created directory which contains
a `Cargo.toml` manifest and a `src/main.rs` binary module.

## `Cargo.toml`

```toml
[package]
name = "my-proxy-list-topics"
version = "0.1.0"
edition = "2024"

[dependencies]
rdkafka = { version = "0.39", features = ["ssl-vendored"], default-features = false }
```

## `src/main.rs`

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

To build the example, we first change to the created directory and use `cargo build` to build
the executable. This will take a few minutes due to dependencies on `rdkafka` and (indirect)
`openssl`.

```shell
> cd my-proxy-list-topics-rust-example
> cargo build
```

Now finally we can run the executable to list the Kafka topics that we have read access to.

```shell
> target/debug/my-proxy-list-topics
...
scratch.example.my-tenant (1)
...
```
