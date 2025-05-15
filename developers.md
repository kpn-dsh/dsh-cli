# Developers

[&#x2190; Set up autocompletion](autocompletion.md)

This page is targeted a developers who wish to work on the `dsh` tool.

## Local installation and run

First clone the repository to your local machine:

```bash
> git clone git@github.com:kpn-dsh/dsh-cli.git
...
> cd dsh-cli
```

Then you can for example install the `dsh` tool on your local machine using:

```bash
> cargo install --path .
...
> dsh platform list
...
```

When developing, it is convenient to set some aliases:

```bash
> alias dsh-dev="cargo run --package dsh --bin dsh --"
> alias dsh-deva="cargo run --all-features --package dsh --bin dsh --"
````

You can then easily run the `dsh` tool without installing it:

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
dsh_api = { version = "0.7.1", features = ["generic"] }
```

The `generic` feature must be enabled. The `dsh` tool has some optional features specified,
which correspond to features of the `dsh_api` crate with the same name:

```toml
[features]
manage = ["dsh_api/manage"]
robot = ["dsh_api/robot"]
```

Because of the strong dependencies between the `dsh` tool and the library,
they are often been worked on at the same time.
In that case it is convenient to set the dependency to the local copy of the library.
Also you might want to enable the `manage` feature while developing.

```toml
dsh_api = { path = "../dsh-api/dsh-api", features = ["generic", "manage"] }
```

However, when you publish the `dsh` tool make sure that you set the dependency
back to the `crates.io` version of the library,

### Coding guidelines

Before pushing code to github, make sure that you adhere to the code formatting defined in
`rustfmt.toml` and that you have run the `clippy` linter. The following commands should
return without any remarks:

```bash
> cargo +nightly fmt --check
> cargo clippy
> cargo clippy --all-features
```

Consider configuring your IDE to automatically apply the formatting rules when saving a file.

## Unit testing

Be sure to include the `--all-features` flag when you run the unit tests:

```bash
> cargo test --all-features
```

## Integration testing

The `tests` directory contains some shell scripts that will run a
large number of commands in sequence. This is not a full test,
but it will catch many bugs which have to do with the command line part of the program.
The tests need to be run from within the `tests` directory.

### `test_commands.sh`

This script will run many commands that all must succeed and print output to `stdout`
(and possibly `stderr`). Commands that would result in changes on the DSH platform
all have the `--dry-run` argument included, so that no actual changes will be made.
Be careful that if you redirect the output to a file,
the default output format will be `json` instead of `table`.
If you want to check the `table` rendering from a file,
you have to explicitly change the output format in the script file.

### `test_erroneous_commands.sh`

This script will run many erroneous commands and print the error message to `stderr`.
All commands must produce a controlled error message and never terminate in panic.

### `test_manage_commands.sh`

This script will run commands that are only available when the `manage` feature
was enabled when the tool was built or installed.
All commands must succeed and print output to `stdout` (and possibly `stderr`).
Commands that would result in changes on the DSH platform
all have the `--dry-run` argument included, so that no actual changes will be made.

### `test_platform_open_commands.sh`

This script will run some `dsh platform open` commands which will try to open DSH
resources and web applications. If you run this script, make sure that you are
already logged in to the Console for the proper platform and tenant.
If successful, you will have some open tabs in your browser.

### `test_styling.sh`

This script will run some commands with different color and styling settings.
This allows for a visual check of the proper rendering of tables and textual console output
to `stdout` and `stderr`.

### `test_targets.sh`

This script will run the `dsh service list` command with different ways of providing the
target platform, tenant and password.

[README &#x2192;](README.md)
