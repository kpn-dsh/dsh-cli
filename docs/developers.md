# Developers

[&#x2190; Set up autocompletion](autocompletion.md)

This page is targeted a developers who wish to work on the `dsh` tool.

## Local installation and run

First clone the repository to your local machine:

```shell
> git clone git@github.com:kpn-dsh/dsh-cli.git
...
> cd dsh-cli
```

Then you can for example install the `dsh` tool on your local machine using:

```shell
> cargo install --path .
...
> dsh platform list
...
```

When developing, it is convenient to set an alias:

```shell
> alias dshd="cargo run --all-features --package dsh --bin dsh --"
````

You can then easily run the `dsh` tool without installing it:

```shell
> dshd platform list
...
```

## Development

### Dependencies

#### `dsh_api` dependency

The `dsh` tool has a strong dependency on the [`dsh_api`](dsh_api) library that provides the
client and data types for the DSH resource management API. This library is published to
`crates.io` and your `Cargo.toml` file should specify the dependency:

```toml
[dependencies]
dsh_api = { version = "0.10.0", features = ["generic"] }
```

The `generic` feature must be enabled. The `dsh` tool has some optional features specified,
which correspond to features of the `dsh_api` crate with the same name:

```toml
[features]
manage = ["dsh_api/manage"]
robot = ["dsh_api/robot"]
```

Because of the strong dependencies between the `dsh` tool and the `dsh_api` library, they are often
been worked on at the same time. In that case it is convenient to set the dependency to the local
copy of the library crate. Also, you might want to enable the `manage` feature while developing.

```toml
dsh_api = { path = "../dsh-api/dsh-api", features = ["generic", "manage"] }
```

rock_api = { version = "0.1.0", path = "/Users/wilbert/Workspaces/kpn/rock-api-rs",
features = ["log", "rcgen"], optional = true }

#### `rock_api` dependency

When the `rock` feature is enabled, the `dsh` tool also has a dependency on the `rock_api`
library that provides functions to sign certificates for private vhosts. At this time it is not
clear in which repository `rock_api` will land.

For now the dependency is set to point to the GitHub repository for the library crate
[`github.com/kpn-dsh/rock-api-rs`](https://github.com/kpn-dsh/rock-api-rs). However, this cannot be
the final solution since this way it won't be possible to publish the `dsh` tool to `crates.io`.
It will be possible however to build/install the tool if you have access to `github.com/kpn-dsh`.

```toml
rock_api = {
    version = "0.1.0",
    git = "ssh://git@github.com/kpn-dsh/rock-api-rs.git",
    features = ["log", "rcgen"],
    optional = true
}
```

Ordinary users that just want to use the tool can download the pre-build versions for the GitHub
release pages.

### Coding guidelines

Before pushing code to `GitHub`, make sure that you adhere to the code formatting defined in
`rustfmt.toml`, that you have run the `clippy` linter and that you did the license check.
The following commands should return without any remarks:

```shell
> cargo +nightly fmt --check
> cargo clippy
> cargo clippy --all-features
> cargo deny check licenses
```

Consider configuring your IDE to automatically apply the formatting rules when saving a file.

For a more thorough check whether all feature combinations work, use the `cargo-all-features`
subcommand:

```shell
> cargo install cargo-all-features
> cargo all-features build
> cargo all-features test
```

## Unit testing

Be sure to include the `--all-features` flag when you run the unit tests:

```shell
> cargo test --all-features
```

## Integration testing

The `tests` directory contains some shell scripts that will run a large number of commands in
sequence. These are not a full tests, but will catch many bugs which have to do with the
command line part of the program. See [tests/README.md](../tests/README.md) for more information
and a description of the available test scripts.

[Publish &#x2192;](publish.md)
