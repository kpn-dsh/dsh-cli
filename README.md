# DSH resource management API command line tool

> **NOTE**  
> If you encounter any bugs or issues you can report them to `unibox@kpn.com`.
> Please include the exact command, the erroneous output and an explanation of the expected output.
> You can also send requests for new features to this e-mail address.

This project provides a tool to call functions on the DSH resource management API from the
command line of your workstation or from a script.
Some of the capabilities of the `dsh` tool are:

* Calling all operations exposed in the DSH resource management API
  from either the command line or from a script.
* Many additional and easier to use functions are provided.
* Extensive help information on each level using the `--help` and `-h` flags,
  including listings of all available operations.
* Configuring platform and tenant credentials interactively via the `dsh` tool.
  Sensitive passwords are stored in your computer's keyring, if available.
* Retrieving information about the available platforms.
* Opening web applications (e.g. the console, the swagger ui or the vhost of your app or service)
  from the command line.
* Reversed lookup functions. For example find all services that use a given secret or volume,
  or contain an environment value with a given value in their configuration.

## Features

By enabling/disabling the features described below you have some control over what's included
in the `dsh` tool and what's not. The features are disabled by default.
The following features are defined:

* `manage` - Enables the manage methods. Enabling this feature is only useful
  if your tenant is authorized for management capabilities.
* `robot` - Enables the robot operation.

## Installation

### Cargo install

If you have the Rust tool-chain installed, the `dsh` tool can be installed on your local machine
directly from `crates.io`, by executing the following command:

```bash
> cargo install dsh --all-features
```

This will install the `dsh` tool with all features enabled in `$HOME/.cargo/bin/dsh`.

### Pre-built binaries

For some platforms a pre-built binary is available at the `GitHub` release page:
[`https://github.com/kpn-dsh/dsh-cli/releases`](https://github.com/kpn-dsh/dsh-cli/releases).
You can download this file to your workstation. Make sure that the directory where you store
the executable is included in your `PATH` variable and that you set the execute flag:

```bash
> chmod $DIR/dsh u+x
```

## Next steps

* [Quick start](docs/quick_start.md)
* [User guide](docs/user_guide.md)
* [Authentication and authorization](docs/authentication_authorization.md)
* [Environment variables](docs/environment_variables.md)
* [Settings and targets](docs/settings_targets.md)
* [Platforms specification](docs/platforms-specification.md)
* [Set up autocompletion](docs/autocompletion.md)
* [Developers](docs/developers.md)
* [Publish](docs/publish.md)
* [Release](docs/release.md)
* [Codesign and notarize for macOS](docs/code-signing-macos.md)
