# DSH resource management API command line tool

> **NOTE**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`.
> Please include the exact command, the erroneous output and an explanation of the expected output.
> You can also send requests for new features to this e-mail address.

This project provides a tool to call functions on the DSH resource management API from the
command line of your workstation. The following DSH resources can be
listed, queried, searched, created and deleted.

## Local installation and run

The DSH Api Command Line Tool (`dsh`) can be installed on your local machine
(assuming you have the `rust` tool chain installed),
by executing the following command.

```bash
> cargo install dsh
...
```

## Targets

* Platform
* Tenant / gid / uid
* Password for each platform / tenant combination

This method will get the target tenant.
This function will try the potential sources listed below, and returns at the first match.

1. Command line argument `--tenant`.
1. Environment variable `DSH_CLI_TENANT`.
1. Parameter `default-tenant` from settings file, if available.
1. If stdin is a terminal, ask the user to enter the value.
1. Else return with an error.

### Environment variables

In order to run `dsh`, either make sure that the environment variables described below
are properly set. These values can also be set via the settings file
or be provided as command line arguments.

<table>
    <tr valign="top">
        <th align="left">variable</th>
        <th align="left">description</th>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_GUID</code></td>
        <td>
            Group and user id for the target tenant. 
            This value must be provided as a single integer, e.g. <code>1903</code>.
            This environment variable can be overridden via the <code>--guid</code>
            command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_OUTPUT_FORMAT</code></td>
        <td>
            This option specifies the format used when printing the output. 
            If this argument is not provided, the value from the settings file will be used. 
            Else, when stdout is a terminal the default 'table' will be used, if
            stdout is not a terminal the value 'json' will be used.
            <ul>
                <li><code>csv</code> - Output will be formatted as comma separated values</li>
                <li><code>json</code> - Output will be in json format</li>
                <li><code>plain</code> - Output will be formatted as plain text</li>
                <li><code>quiet</code> - No output will be generated</li>
                <li><code>table</code> - Output will be formatted as a table with borders</li>
                <li>
                    <code>table-no-border</code> - Output will be formatted as a table 
                    without borders
                </li>
                <li><code>toml</code> - Output will be in toml format</li>
                <li><code>yaml</code> - Output will be in yaml format</li>
            </ul>
            This environment variable can be overridden via the 
            <code>--output-format</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PASSWORD</code></td>
        <td>
            This environment variable specifies the ecret api token/password for the target tenant. 
            Note that when the environment variable <code>DSH_CLI_PASSWORD_FILE</code> 
            or the argument <code>--password-file</code> command line argument is provided,
            this environment variable will not be used. 
            For better security, consider using one of these two options instead of 
            defining <code>DSH_CLI_PASSWORD</code>
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PASSWORD_FILE</code></td>
        <td>
            This environment variable specifies a file containing the secret api 
            token/password for the target tenant. 
            Note that when the <code>--password-file</code> command line argument is provided,
            this environment variable will not be used. 
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PLATFORM</code></td>
        <td>
            Target platform on which the tenant's environment lives.
            <ul>
                <li><code>np-aws-lz-dsh / nplz</code> - Staging platform for KPN internal tenants</li>
                <li><code>poc-aws-dsh / poc</code> - Staging platform for non KPN tenants</li>
                <li><code>prod-aws-dsh / prod</code> - Production platform for non KPN tenants</li>
                <li><code>prod-aws-lz-dsh / prodlz</code> - Production platform for KPN internal tenants</li>
                <li><code>prod-aws-lz-laas / prodls</code> - Production platform for logstash as a service</li>
                <li><code>prod-azure-dsh / prodaz</code> - Production platform for non KPN tenants</li>
            </ul>
            This environment variable can be overridden via the 
            <code>--platform</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_TENANT</code></td>
        <td>Tenant id for the target tenant. The target tenant is the tenant whose resources 
            will be managed via the api.
            This environment variable can be overridden via the 
            <code>--tenant</code> command line argument.
        </td>
    </tr>
</table>

### Settings file

### Run

When installation completed without any errors and the environment variables are set,
you should be able to start the tool from the command line.

```bash
> dsh
DSH resource management api command line interface.

Usage: dsh [OPTIONS] [SUBJECT/COMMAND]

Subjects/commands:
  api          List and call DSH resource management api.
  app          Show, manage and list apps deployed from the DSH app catalog.
  application  Show, manage and list applications deployed on the DSH.
  bucket       Show, manage and list DSH buckets.
  certificate  Show, manage and list DSH certificates.
  env          Find values used in configurations.
  image        Show image usage.
  manifest     Show App Catalog manifests.
  metric       Show metric exports.
  platform     Show, list and open platform resources.
  proxy        Show, manage and list DSH Kafka proxies.
  secret       Show, manage and list DSH secrets.
  topic        Show, manage and list DSH topics.
  vhost        Show vhost usage.
  volume       Show, manage and list DSH volumes.
  setting      Show, manage and list dsh settings.
  target       Show, manage and list dsh target configurations.
  help         Print this message or the help of the given subcommand(s)

Options:
  -p, --platform <PLATFORM>                 Provide target platform. [possible values: np-aws-lz-dsh, poc-aws-dsh,
                                            prod-aws-dsh, prod-aws-lz-dsh, prod-aws-lz-laas, prod-azure-dsh]
  -t, --tenant <TENANT>                     Provide target tenant.
  -g, --guid <GUID>                         Provide target group and user id.
      --password-file <FILE>                Provide password file name.
  -o, --output-format <FORMAT>              Set output format. [possible values: csv, json, json-compact, plain, quiet,
                                            table, table-no-border, toml, toml-compact, yaml]
  -v, --verbosity <VERBOSITY>               Set verbosity level. [possible values: off, low, medium, high]
      --dry-run                             Execute in dry-run mode.
      --force                               Force changes without confirmation.
      --matching-style <STYLE>              Set styling for matches. [possible values: normal, bold, dim, italic,
                                            underlined, reverse]
      --no-color                            No color.
  -q, --quiet                               Run in quiet mode.
      --show-execution-time                 Show execution time.
      --terminal-width <WIDTH>              Set terminal width.
      --generate-autocomplete-file <SHELL>  Generate autocomplete file [possible values: bash, elvish, fish, powershell,
                                            zsh]
  -h, --help                                Print help (see more with '--help')
  -V, --version                             Print version

For most commands adding an 's' as a postfix will yield the same result as using the 'list' subcommand, e.g. using 'dsh
apps' will be the same as using 'dsh app list'.
```

You can have a more comprehensive explanation by using the `--help` command line option.
Commands also have their own help text.

```
> dsh --help
> dsh secret --help
> dsh secret list --help
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
dsh_api = "0.3.1"
```

When developing simultaneously on `dsh` and `dsh_api` consider changing the library dependency
to your local copy.

```toml
# Cargo.toml
dsh_api = { path = "../dsh-api/dsh-api" }
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
