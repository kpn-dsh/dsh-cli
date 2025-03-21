# DSH resource management API command line tool

> **NOTE**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`.
> Please include the exact command, the erroneous output and an explanation of the expected output.
> You can also send requests for new features to this e-mail address.

This project provides a tool to call functions on the DSH resource management API from the
command line of your workstation or from a script.
Below in an overview of the capabilities of the `dsh` tool:

* Calling all operations exposed in DSH resource management API
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
in the `dsh` tool and what's not. The features are disabled by default.
The following features are defined:

* `manage` - Enables the manage methods.
* `robot` - Enables the robot operation.

## Installation

The DSH Api Command Line Tool (`dsh`) can be installed on your local machine
(assuming you have the `rust` tool-chain installed),
by executing the following command.

```bash
> cargo install dsh
...
```

## Run

When installation completed without any errors,
you should be able to start the `dsh` tool from the command line.

```bash
> dsh
DSH resource management api command line interface.

Usage: dsh [OPTIONS] [SUBJECT/COMMAND]
       dsh [SUBJECT/COMMAND] [SUBCOMMAND] [OPTIONS]
       dsh --help
       dsh [SUBJECT/COMMAND] --help
       dsh [SUBJECT/COMMAND] [SUBCOMMAND] --help

Subjects/commands:
  api          List and call DSH resource management api.
  app          Show, manage and list apps deployed from the DSH app catalog.
  bucket       Show, manage and list DSH buckets.
  certificate  Show, manage and list DSH certificates.
  env          Find values used in configurations.
  image        Show image usage.
  manifest     Show App Catalog manifests.
  metric       Show metric exports.
  platform     Show, list and open platform resources.
  proxy        Show, manage and list DSH Kafka proxies.
  secret       Show, manage and list DSH secrets.
  service      Show, manage and list services deployed on the DSH.
  token        Request DSH tokens.
  topic        Show, manage and list DSH topics.
  vhost        Show vhost usage.
  volume       Show, manage and list DSH volumes.
  setting      Show, manage and list dsh settings.
  target       Show, manage and list dsh target configurations.

Options:
  -p, --platform <PLATFORM>...  Provide target platform [possible values: np-aws-lz-dsh, poc-aws-dsh, prod-aws-dsh,
                                prod-aws-lz-dsh, prod-aws-lz-laas, prod-azure-dsh]
  -t, --tenant <TENANT>...      Provide target tenant
      --password-file <FILE>    Provide target password file name
      --dry-run                 Execute in dry-run mode
      --force                   Force changes without confirmation
  -h, --help                    Print help (see more with '--help')
  -V, --version                 Print version

Output options:
      --log-level <LEVEL>       Set log level [possible values: off, error, warn, info, debug, trace]
      --log-level-api <LEVEL>   Set log level for the dsh api crate
      --log-level-sdk <LEVEL>   Set log level for the dsh sdk crate
      --matching-color <COLOR>  Set color for matches [possible values: normal, red, green, blue]
      --matching-style <STYLE>  Set styling for matches [possible values: normal, bold, dim, italic, underlined,
                                reverse]
      --no-color                No color
      --no-headers              No headers
  -o, --output-format <FORMAT>  Set output format [possible values: csv, json, json-compact, plain, quiet, table,
                                table-no-border, toml, toml-compact, yaml]
  -q, --quiet                   Run in quiet mode
  -v, --verbosity <VERBOSITY>   Set verbosity level [possible values: off, low, medium, high]
      --show-execution-time     Show execution time
      --terminal-width <WIDTH>  Set terminal width
```

You can have a more comprehensive explanation by using the `--help` command line option.
Commands also have their own help text.

```
> dsh --help
> dsh secret --help
> dsh secret list --help
```
