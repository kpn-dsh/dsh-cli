# DSH resource management API command line tool

> **NOTE**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`.
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
  Sensitive passwords are stored in your computer's keychain, if available.
* Retrieving information about the available platforms.
* Opening web applications (e.g. the console, the swagger ui or the vhost of your app or service)
  from the command line.
* Reversed lookup functions. For example find all services that use a given secret or volume,
  or have an environment value with a given value in their configuration.

## Features

By enabling/disabling the features described below you have some control over what's included
in the `dsh` tool and what's not. The features are all disabled by default.
The following features are defined:

* `appcatalog` - Enables the app catalog methods.
* `manage` - Enables the manage methods.
* `robot` - Enables the robot operation.

## Installation

If you have the `rust` tool-chain installed, the DSH Api Command Line Tool (`dsh`) can be
installed on your local machine directly from `crates.io` by executing the following command.

```bash
> cargo install dsh
```

This will install the `dsh` tool without any features.

You can add one or more features by providing the appropriate flags.
For example, to install the `manage` and the `robot` features, use:

```bash
> cargo install dsh --features manage,robot
```

You can also use the `--all-features` flag to install all features.

## Next steps

* [Quick start](quick_start.md)
* [User guide](user_guide.md)
* [Environment variables](environment_variables.md)
* [Settings and targets](settings_targets.md)
* [Platforms specification](platforms-specification.md)
* [Set up autocompletion](autocompletion.md)
* [Developers](developers.md)
