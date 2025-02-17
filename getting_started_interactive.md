# Getting started - interactive

When installation is complete you should be able to start the tool from the command line.

```bash
> dsh
DSH resource management api command line interface.

Usage: dsh [OPTIONS] [SUBJECT/COMMAND]
       dsh --help
       dsh secret --help
       dsh secret list --help

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
  -p, --platform <PLATFORM>     Provide target platform. [possible values: np-aws-lz-dsh, poc-aws-dsh, prod-aws-dsh,
                                prod-aws-lz-dsh, prod-aws-lz-laas, prod-azure-dsh]
  -t, --tenant <TENANT>         Provide target tenant.
      --password-file <FILE>    Provide target password file name.
  -o, --output-format <FORMAT>  Set output format. [possible values: csv, json, json-compact, plain, quiet, table,
                                table-no-border, toml, toml-compact, yaml]
  -v, --verbosity <VERBOSITY>   Set verbosity level. [possible values: off, low, medium, high]
      --dry-run                 Execute in dry-run mode.
      --force                   Force changes without confirmation.
      --matching-style <STYLE>  Set styling for matches. [possible values: normal, bold, dim, italic, underlined,
                                reverse]
      --no-color                No color.
      --no-headers              No headers.
  -q, --quiet                   Run in quiet mode.
      --log-level <LEVEL>       Set log level. [possible values: off, error, warn, info, debug, trace]
      --log-level-api <LEVEL>   Set log level for the dsh api crate.
      --log-level-sdk <LEVEL>   Set log level for the dsh sdk crate.
      --show-execution-time     Show execution time.
      --terminal-width <WIDTH>  Set terminal width.
  -h, --help                    Print help (see more with '--help')
  -V, --version                 Print version

For most commands adding an 's' as a postfix will yield the same result as using the 'list' subcommand, e.g. using 'dsh
apps' will be the same as using 'dsh app list'.
```

You can have a more comprehensive explanation by using the `--help` command line option.
Subcommands also have their own help text.

```
> dsh --help
> dsh secret --help
> dsh secret list --help
```

## Target platform and tenant

Most functions of the `dsh` tool depend on the DSH resource management api,
which is a rest web service.
In order to be authenticated and authorized for this web service,
three target values are required:

* `platform` - The target platform where the DSH resources and services are deployed.
  This can either be a development or a production environment,
  and platforms will typically serve different kinds of tenants.
* `tenant` - The target tenant that is responsible for the resources and services
  that are deployed on the DSH. Tenants will often have resources and services
  on more than one platform, e.g. a development platform and a production platform.
* `password` - Each combination of a `platform` and a `tenant` is considered
  a separate entity with respect to authentication and authorization,
  and therefor needs a separate `password`.
  This password can be obtained by logging in to the DSH console application
  for the `platform` and `tenant`, and selecting the `Resources > Secrets` menu.
  There it will be listed as `system/rest-api-client`.

Each `dsh` function that needs to access the web service must provide these three values.
Each of these values can be provided to the tool as either:

* a command line argument,
* via an environment variable,
* via the settings/targets files on the file system or
* interactively via a prompt.

The tool will check these possible sources for the target parameters in this order.
Passwords can also be stored in the computer's keyring application
(currently only available on OsX and Windows).

Functions that do not need to access the web service do not require these target parameters.

#### Command line arguments

* `--platform` - Specify the target platform by either the full name
  or the alias (e.g. `np-aws-lz-dsh` or `nplz`).
* `--tenant` - Specify the target tenant name,
* `--password-file` - Specify the name of a file containing the target password.
  This can either be an absolute file name
  or a relative file name from the working directory of your application.

```bash
> dsh --platform nplz --tenant my-tenant --password-file ~/.password secret list
```

Specifying the password directly as a command line argument is a security hazard
and is not supported.

#### Environment variables

> **NOTE**   
> For detailed information about all environment variables,
> see [Environment variables](environment_variables.md).

If you did not specify the target platform and/or target tenant on the command line,
the tool will try to get the values from environment variables:

* `DSH_CLI_PLATFORM` - Specify the target platform by providing either the full name or the alias
  (e.g. `np-aws-lz-dsh` or `nplz`).
* `DSH_CLI_TENANT` - Specify the tenant name.
* `DSH_CLI_PASSWORD_FILE` - Specify the name of a file containing the target password.
  This can either be an absolute file name
  or a relative file name from the working directory of your application.
* `DSH_CLI_PASSWORD` - The password can also be specified directly from an environment variable.

```bash
> export DSH_CLI_PLATFORM=np-aws-lz-dsh
> export DSH_CLI_TENANT=my-tenant
> export DSH_CLI_PASSWORD="..."
```

#### Settings and targets

> **NOTE**   
> For detailed information about all settings and targets,
> see [Settings and targets](settings_and_targets.md).

The `dsh` tool stores its settings and default values in a settings file,
which is stored in the `dsh` program directory.
This program directory is by default located in the user's home directory (`$HOME/.dsh_cli`),
but the location can be overridden via an environment variable (`DSH_CLI_HOME`).

If you did not specify the target platform and/or target tenant on the command line
and the environment variables are not defined,
the tool will try the `default-platform` and `default-tenant` values in the settings file.

```bash
> cat ~/.dsh_cli/settings.toml
default-platform = "np-aws-lz-dsh"
default-tenant = "my-tenant"
verbosity = "medium"
```

The settings file is a `toml` file and can be edited manually,
but the easier and preferred way is to use the `setting set` or `setting unset` commands:

```bash
> dsh setting set default-platform prodlz
default platform set to prod-aws-lz-dsh
> dsh setting set default-tenant other-tenant
default tenant set to other-tenant
> cat ~/.dsh_cli/settings.toml
default-platform = "prod-aws-lz-dsh"
default-tenant = "other-tenant"
verbosity = "medium"
```

When the `default-platform` and `default-tenant` settings are present in the settings file,
these values do not have to be specified via the command line or environment variables.
If you do anyway, the command line and environment variable values will take precedence.

#### Interactively via a prompt

* as command line arguments per request
* stored in environment variables
* stored in the file system
* entered by the user on request

```bash
> dsh setting unset default-platform
default platform unset
> dsh setting unset default-tenant
default tenant unset
> cat ~/.dsh_cli/settings.toml
verbosity = "medium"
> unset DSH_CLI_PLATFORM
> unset DSH_CLI_TENANT
> unset DSH_CLI_PASSWORD
> unset DSH_CLI_PASSWORD_FILE
> dsh secret list
target platform: nplz
target tenant: my-tenant
password for tenant my-tenant@np-aws-lz-dsh: ...
list all secret ids
┌─────────────────────────────────────────┐
│ secret ids (1)                          │
├─────────────────────────────────────────┤
│ api-key                                 │
└─────────────────────────────────────────┘
```

## `> dsh target`

```shell
> dsh target list
```

```shell
> dsh target new
create new target configuration
enter platform: nplz
enter tenant: my-tenant
enter password:
target my-tenant@np-aws-lz-dsh created
>
```

```shell
> dsh target delete
```

```shell
> dsh target default
```

```shell
> dsh target unset-default
```

## Use in scripts

Trying to use the dsh-cli for creating everything.

```bash
dsh --version
# expected output
# dsh version: 0.4.0
# dsh-api library version: 0.4.0
# dsh rest api version: 1.9.0
```

First verify the connection:

```bash
dsh apps
```

Manual config:

```text
platform: prod-aws-lz-dsh
tenant: kpnbm-e2e-01
groupid: 2061
password: see secrets
```

Create a target config with this data:

```bash
touch ~/.dsh_cli/targets/prod-aws-lz-dsh.kpnbm-e2e-01.toml
cat >> ~/.dsh_cli/targets/prod-aws-lz-dsh.kpnbm-e2e-01.toml<< EOF
platform = "prod-aws-lz-dsh"
tenant = "kpnbm-e2e-01"
group-user-id = 2061
EOF
```

Now I am still getting the request for providing details. So I will also add a config.

```bash
touch ~/.dsh_cli/settings.toml
cat >> ~/.dsh_cli/settings.toml<< EOF
default-platform = "prod-aws-lz-dsh"
default-tenant = "kpnbm-e2e-01"
matching-style = "bold"
show-execution-time = false
verbosity = "medium"
EOF
```

Now, `dsh apps` only requires a password.

## Installation

If you have the `rust` tool chain installed, the DSH Api Command Line Tool (`dsh`) can be
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
