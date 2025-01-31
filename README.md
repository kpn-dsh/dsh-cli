# DSH resource management API command line tool

> **WARNING**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`.
> Please include the exact command, the erroneous output and an explanation of the expected output.
> You can also send requests for new features to this e-mail address.

* [Environment variables](environment_variables.md)
* [Set up autocompletion](autocompletion.md)
* [Developers](developers.md)

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

## Installation

If you have the `rust` tool chain installed, the DSH Api Command Line Tool (`dsh`) can be
installed on your local machine directly from `crates.io` by executing the following command.

```bash
> cargo install dsh
...
```

## Run

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
  token        Request DSH tokens.
  topic        Show, manage and list DSH topics.
  vhost        Show vhost usage.
  volume       Show, manage and list DSH volumes.
  setting      Show, manage and list dsh settings.
  target       Show, manage and list dsh target configurations.
  help         Print this message or the help of the given subcommand(s)

Options:
  -p, --platform <PLATFORM>
          Provide target platform. [possible values: np-aws-lz-dsh, poc-aws-dsh, prod-aws-dsh,
          prod-aws-lz-dsh, prod-aws-lz-laas, prod-azure-dsh]
  -t, --tenant <TENANT>
          Provide target tenant.
  -g, --guid <GUID>
          Provide target group and user id.
      --password-file <FILE>
          Provide password file name.
  -o, --output-format <FORMAT>
          Set output format. [possible values: csv, json, json-compact, plain, quiet, table,
          table-no-border, toml, toml-compact, yaml]
  -v, --verbosity <VERBOSITY>
          Set verbosity level. [possible values: off, low, medium, high]
      --dry-run
          Execute in dry-run mode.
      --force
          Force changes without confirmation.
      --matching-style <STYLE>
          Set styling for matches. [possible values: normal, bold, dim, italic, underlined,
          reverse]
      --no-color
          No color.
  -q, --quiet
          Run in quiet mode.
      --show-execution-time
          Show execution time.
      --terminal-width <WIDTH>
          Set terminal width.
      --generate-autocomplete-file <SHELL>
          Generate autocomplete file [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version

For most commands adding an 's' as a postfix will yield the same result as using the 'list'
subcommand, e.g. using 'dsh apps' will be the same as using 'dsh app list'.
```

You can have a more comprehensive explanation by using the `--help` command line option.
Commands also have their own help text.

```
> dsh --help
> dsh secret --help
> dsh secret list --help
```

## Configuration

The `dsh` tool can be run entirely from the command line and
all configurations and parameters can be specified via command line arguments.
However, especially when using `dsh` interactively,
it is much more convenient to make some settings persistent.

### Environment variables

Almost all configuration and parameters can be configured via environment variables.
The table below describes some variables to get you started.
Detailed information about all environments variables can be found in
[`environment_variables.md`](environment_variables.md).

<table>
    <tr valign="top">
        <th align="left">variable</th>
        <th align="left">description</th>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_OUTPUT_FORMAT</code></td>
        <td>
            This option specifies the format used when printing the output. 
            Some possible values are
            <code>csv</code>, <code>json</code>, <code>plain</code>, <code>table</code>, 
            <code>toml</code> or <code>yaml</code>.
            Note that not all output formats can be used with all subjects and capabilities.
            The output format can also be set via the 
            <code>--output-format</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PASSWORD</code></td>
        <td>
            This environment variable specifies the secret api token/password for the target tenant.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PASSWORD_FILE</code></td>
        <td>
            This environment variable specifies a file containing the secret api 
            token/password for the target tenant. 
            Note that when the <code>--password-file</code> command line argument is provided,
            this will override the environment variable. 
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PLATFORM</code></td>
        <td>
            Target platform on which the tenant's environment lives. 
            The platform can be specified by its full name (e.g. <code>np-aws-lz-dsh</code>) 
            or by its alias (e.g. <code>nplz</code>).
            The platform can also be specified via the 
            <code>--platform</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_TENANT</code></td>
        <td>
            Tenant id for the target tenant. The target tenant is the tenant whose resources 
            will be managed via the api.
            The tenant can also be set via the 
            <code>--tenant</code> command line argument.
        </td>
    </tr>
</table>

### Settings

The `dsh` tool is also able to store settings and configuration in settings files.
The settings files can be created and managed via the tool itself, which is the preferred way.
The settings are typically stored in a subdirectory of the user's home directory
(`$HOME/.dsh_cli`).
This location can be changed by setting the environment variable `DSH_CLI_HOME`.

The settings are stored in the file `$HOME/.dsh_cli/settings.toml`:

```toml
default-platform = "np-aws-lz-dsh"
default-tenant = "greenbox-dev"
matching-style = "bold"
show-execution-time = false
verbosity = "medium"
```

The target data is stored in files in the directory `$HOME/.dsh_cli/targets`.
For each combination of a platform and a tenant there is a separate file.
E.g., for the platform `np-aws-lz-dsh` and the tenant `greenbox-dev` the target data is stored in
the file `$HOME/.dsh_cli/targets/np-aws-lz-dsh.greenbox-dev.toml`:

```toml
platform = "np-aws-lz-dsh"
tenant = "greenbox-dev"
group-user-id = 1903
```

The target's password is not stored in these files.
For security reasins.passwords are stored in your computers keychain (for Mac Osx and Windows). 
