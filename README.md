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
* `rock` - Enable certificate signing capabilities via the KPN _RoCK API_ service.

## Installation

### Pre-built binaries

For macOS and Linux pre-built binaries are available via the projects `GitHub`
[release page](https://github.com/kpn-dsh/dsh-cli/releases).
The installation for macOS and Linux is the same:

* Download the binary for your OS.
* Rename the downloaded binary to `dsh`.
* Make the `dsh` binary executable (`chmod u+x dsh`).
* Move the binary to a directory that is included in your `PATH` variable (`/usr/local/bin` or
  `/usr/bin` or similar).

### Cargo install

If you have the Rust tool-chain installed, the `dsh` tool can be installed on your local machine
directly from `crates.io`, by executing the following command:

```shell
> cargo install dsh --all-features --locked
```

This will install the `dsh` tool with all features enabled in `$HOME/.cargo/bin/dsh`.

## Next steps

* [Quick start](docs/quick-start.md)
* [User guide](docs/user-guide.md)
* [Authentication and authorization](docs/authentication-authorization.md)
* [Environment variables](docs/environment-variables.md)
* [Settings and targets](docs/settings-targets.md)
* [Kafka proxy](docs/proxy.md)
* [Platforms specification](docs/platforms-specification.md)
* [Set up autocompletion](docs/autocompletion.md)
* [Developers](docs/developers.md)
* [Publish](docs/publish.md)
* [Release](docs/release.md)
* [Codesign and notarize for macOS](docs/code-signing-macos.md)
