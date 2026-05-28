# Code example `rust producer`

For this example we will create a `producer` example for the `rust` programming language:

```shell
> dsh proxy code my-proxy rust producer
generating rust producer example for bundle 'my-proxy' for 'np-aws-lz-dsh@greenbox-dev'
created directory 'my-proxy-producer-rust-example'
created directory 'my-proxy-producer-rust-example/src'
created file 'my-proxy-producer-rust-example/src/main.rs'
created file 'my-proxy-producer-rust-example/Cargo.toml'
rust code for bundle 'my-proxy' generated in directory 'my-proxy-producer-rust-example'
```

As is shown in the output, the example is generated in a newly created directory which contains
a `Cargo.toml` manifest and a `src/main.rs` binary module.

```shell
> ls -lR my-proxy-producer-rust-example
total 8
-rw-r--r--  1 username  staff  186 28 mei  18:07 Cargo.toml
drwxr-xr-x  3 username  staff   96 28 mei  18:07 src

my-proxy-producer-rust-example/src:
total 8
-rw-r--r--  1 username  staff  1908 28 mei  18:07 main.rs
```

To build the example, we first change to the created directory and use `cargo build` to build
the executable. This will take a few minutes due to dependencies on `rdkafka` and (indirect)
`openssl`.

```shell
> cd my-proxy-producer-rust-example
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
