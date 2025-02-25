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
...
> dsh platform list
...
```

When developing, it is convenient to set an alias:

```bash
> alias dsh-dev="cargo run --package dsh --bin dsh --"
````

You can then easily run the tool without installing it:

```bash
> dsh-dev platform list
...
```

## Development

### Dependencies

The `dsh` tool has a strong dependency on the [`dsh_api`](dsh_api) library,
that provides the client and data types for the DSH resource management API.
This library is published to `crates.io` and your `Cargo.toml` file
should specify the dependency:

```toml
[dependencies]
dsh_api = { version = "0.5.1", features = ["generic"] }
```

The `generic` feature must be enabled. The cli tool has some optional features specified,
which correspond to features of the `dsh_api` crate with the same name:

```toml
[features]
appcatalog = ["dsh_api/appcatalog"]
manage = ["dsh_api/manage"]
robot = ["dsh_api/robot"]
```

Because of the strong dependencies between the tool and the library,
they are often been worked on at the same time.
In that case it is convenient to set the dependency to the local copy of the library:

```toml
dsh_api = { path = "../dsh-api/dsh-api", features = ["generic"] }
```

However, when you publish the tool make sure that you set the dependency
back to the `crates.io` version of the library,

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

## Testing

The `tests` directory contains a number of shell scripts that will run a
fairly large number of command automatically. This is not a full test,
but it will catch many bugs which have to do with the command line part of the application.
The tests need to be run from within the `tests` directory.

```bash
> cd tests
> ./run_all_safe_commands.sh
```

This will run many commands and print the output to the terminal.
Be careful that if you redirect the output to a file,
the default output format will be `json` instead of `table`.
If you want to check the `table` rendering from a file,
you have to explicitly specify the format.

```bash
> ./run_all_safe_commands.sh --output-format table > output-table.txt
```
