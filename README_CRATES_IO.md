# DSH resource management API command line tool

> **NOTE**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`.
> Please include the exact command, the erroneous output and an explanation of the expected output.
> You can also send requests for new features to this e-mail address.

This project provides a tool to call functions on the DSH resource management API from the
command line of your workstation. The following DSH resources can be
listed, queried, searched, created and deleted.

* api
* app from the app catalog
* application / service
* bucket
* certificate
* environment variable
* image
* manifest
* metric
* platform
* proxy
* secret
* topic
* vhost
* volume

See the `README` file in the repository for more information.

## Installation

The DSH Api Command Line Tool (`dsh`) can be installed on your local machine
(assuming you have the `rust` tool chain installed),
by executing the following command.

```bash
> cargo install dsh
...
```

## Run

When installation completed without any errors,
you should be able to start the tool from the command line.

```bash
> dsh
DSH resource management api command line interface.

Usage: dsh [OPTIONS] [SUBJECT/COMMAND]

Subjects/commands:
  api          List and call DSH resource management api.
  app          Show, manage and list apps deployed from the DSH app
               catalog.
  application  Show, manage and list applications deployed on the
               DSH.
  bucket       Show, manage and list DSH buckets.
  certificate  Show, manage and list DSH certificates.
  env          Find values used in configurations.
  image        Show image usage.
  manifest     Show App Catalog manifests.
  metric       Show metric exports.
  platform     Show, list and open platform resources.
  proxy        Show, manage and list DSH Kafka proxies.
  secret       Show, manage and list DSH secrets.
  token        Request DSH tokens.
  topic        Show, manage and list DSH topics.
  vhost        Show vhost usage.
  volume       Show, manage and list DSH volumes.
  setting      Show, manage and list dsh settings.
  target       Show, manage and list dsh target configurations.
  help         Print this message or the help of the given
               subcommand(s)

Options:
  -p, --platform <PLATFORM>
          Provide target platform. [possible values: np-aws-lz-dsh,
          prod-aws-lz-dsh]
  -t, --tenant <TENANT>
          Provide target tenant.
  -g, --guid <GUID>
          Provide target group and user id.
      --password-file <FILE>
          Provide password file name.
  -o, --output-format <FORMAT>
          Set output format. [possible values: csv, json,
          json-compact, plain, quiet, table, table-no-border, toml,
          toml-compact, yaml]
  -v, --verbosity <VERBOSITY>
          Set verbosity level. [possible values: off, low, medium,
          high]
      --dry-run
          Execute in dry-run mode.
      --force
          Force changes without confirmation.
      --matching-style <STYLE>
          Set styling for matches. [possible values: normal, bold,
          dim, italic, underlined, reverse]
      --no-color
          No color.
  -q, --quiet
          Run in quiet mode.
      --show-execution-time
          Show execution time.
      --terminal-width <WIDTH>
          Set terminal width.
      --generate-autocomplete-file <SHELL>
          Generate autocomplete file [possible values: bash,
          elvish, fish, powershell, zsh]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version

For most commands adding an 's' as a postfix will yield the same
result as using the 'list' subcommand, e.g. using 'dsh apps' will
be the same as using 'dsh app list'.
```

You can have a more comprehensive explanation by using the `--help` command line option.
Commands also have their own help text.

```
> dsh --help
> dsh secret --help
> dsh secret list --help
```
