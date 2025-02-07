# Developers

This page is targeted a developers who wish to work on the `dsh` tool.

## Local installation and run

First clone the repository to your local machine:

```bash
> git clone git@github.com:kpn-dsh/dsh-cli.git
...
> cd dsh-cli
```

Then you can for example install the tool on your local machine using:

```bash
> cargo install --path .
```

## Development

### Dependencies

The `dsh` tool has a strong dependency on the [`dsh_api`](dsh_api) library,
that provides the client for the DSH resource management API.
This library is continuously being worked on, and is published to `crates.io`.
Hence, at this time `dsh` depends on the [crates.io](https://crates.io/crates/dsh_api)
version of the library.

```toml
# Cargo.toml
dsh_api = "0.5.0"
```

When developing simultaneously on `dsh` and `dsh_api` consider changing the library dependency
to your local copy.

```toml
# Cargo.toml
dsh_api = { path = "../dsh-api/dsh-api", features = "generic" }
```

### Coding guidelines

Before pushing code to github, make sure that you adhere to the code formatting defined in
`rustfmt.toml` and that you have run the `clippy` linter. The following commands should
return without any remarks:

```bash
> cargo +nightly fmt --check
```

```bash
> cargo clippy
```

Consider configuring your IDE to automatically apply the formatting rules when saving a file.
