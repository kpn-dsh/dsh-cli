# Code example `python consumer`

For this example we will create a `consumer` example for the `rust` programming language:

```shell
> dsh proxy code my-proxy rust consumer
generating rust consumer example for bundle 'my-proxy' for 'np-aws-lz-dsh@greenbox-dev'
created directory 'my-proxy-consumer-rust-example'
created directory 'my-proxy-consumer-rust-example/src'
created file 'my-proxy-consumer-rust-example/src/main.rs'
created file 'my-proxy-consumer-rust-example/Cargo.toml'
rust code for bundle 'my-proxy' generated in directory 'my-proxy-consumer-rust-example'
```

As is shown in the output, the example is generated in a newly created directory which contains
a `Cargo.toml` manifest and a `src/main.rs` binary module.

```shell
> ls -lR my-proxy-consumer-rust-example
total 8
-rw-r--r--  1 username  staff  186 28 mei  18:07 Cargo.toml
drwxr-xr-x  3 username  staff   96 28 mei  18:07 src

my-proxy-consumer-rust-example/src:
total 8
-rw-r--r--  1 username  staff  1908 28 mei  18:07 main.rs
```

To build the example, we first change to the created directory and use `cargo build` to build
the executable. This will take a few minutes due to dependencies on `rdkafka` and (indirect)
`openssl`.

```shell
> cd my-proxy-consumer-rust-example
> cargo build
```

Now finally we can run the executable and receive messages from a Kafka topic. For the example we
assume that your platform/tenant has read access to a Kafka topic that can provide a realtime
stream of messages. In the example below we use `realtime-topic` for this.

```shell
> target/debug/my-proxy-consumer scratch.realtime-topic.my-tenant
0:64236241 created:6178587
0:64236242 updated:0130269727
0:64236243 updated:20144956071
0:64236244 cancelled:20196925708
0:64236245 updated:20144909534
^C
```

Since the example code has dependencies on `rdkafka` and (indirect) to `openssl`, the build
process will take a few minutes.

called
`my-proxy-consumer-rust-example`.
