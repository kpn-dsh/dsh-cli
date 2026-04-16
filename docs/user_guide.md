# User guide

[&#x2190; Quick start](quick_start.md)

To see an overview of the most important commands that the `dsh` tool provides, just type
the tool name without any command, subcommand, options or arguments:

```shell
> dsh
DSH resource management api command line interface.

Usage: dsh [OPTIONS] [SUBJECT/COMMAND]

Subjects/commands:
  api          List and call DSH resource management api.
  app          Show, manage and list apps deployed from the DSH app catalog.
  bucket       Show, manage and list DSH buckets.
  certificate  Show, manage and list DSH certificates.
  env          Find values used in configurations.
  image        Show image usage.
  login        Login via single sign on
  logout       Logout from single sign on
  manifest     Show App Catalog manifests.
  metric       Show metric exports.
  nodepool     Show node pool resources.
  platform     Show, list and open platform resources.
  proxy        Show, manage and list DSH Kafka proxies.
  robot        Manage and store robot secrets.
  secret       Show, manage and list DSH secrets.
  service      Show, manage and list services deployed on the DSH.
  setting      Show, manage and list dsh settings.
  stream       Show, manage and list internal and public managed streams.
  tenant       Show and manage tenants on the DSH.
  token        Request DSH tokens.
  topic        Show, manage and list DSH scratch topics.
  vhost        Show vhost usage.
  volume       Show, manage and list DSH volumes.

Options:
      --dry-run  Execute in dry-run mode
      --force    Force changes without confirmation
  -h, --help     Print help (see more with '--help')

Output options:
  -o, --output-format <FORMAT>  Set output format [possible values: csv, json, json-compact, 
                                plain, quiet, table, table-no-border, toml, toml-compact, yaml]
  -q, --quiet                   Run in quiet mode
  -v, --verbosity <VERBOSITY>   Set verbosity level [possible values: off, low, medium, high]


Settings:
  file-name  /Users/me/.dsh_cli/settings.toml
```

You can have a complete list of all command and a more comprehensive explanation by using the
`--help` command line option. Subcommands also have their own help text.

```shell
> dsh --help
> dsh secret
> dsh secret --help
> dsh secret list --help
```

## Target platform and tenant

Most functions of the `dsh` tool depend on the DSH resource management api,
which is a rest web service. Each request to the api is targeted at one platform and one
tenant only. This means that most commands of the `dsh` tool require a platform and tenant
to be specified.

Functions that do not need to access the web service do not require these target parameters.

### Platform

The target platform specifies where the DSH resources and services are living.
This can either be a development or a production environment,
and platforms will typically serve different kinds of tenants.
To get a list of all supported platforms, use the command `dsh platform list`.

When invoking a command, the platform can be specified in a number of different ways
listed below (sorted by decreasing precedence).

1. `--platform` or `-p` command line option
2. `DSH_CLI_PLATFORM` environment variable
3. `default-platform` parameter in settings (not available for robot authentication method)
4. If stdin is a terminal, the user is prompted for the platform

### Tenant

The target tenant is responsible for the resources and services
that are deployed on the DSH. Tenants will often have resources and services
on more than one platform, e.g. a development platform and a production platform.
The tenant name can be specified in different ways (again sorted by decreasing
precedence):

1. `--all-tenants` command line option (not available in robot authentication method)
2. `--tenants` command line option (not available in robot authentication method)
3. `--tenant` or `-t` command line option
4. `DSH_CLI_TENANT` environment variable
5. `default-tenant` parameter in settings (not available in robot authentication method)
6. If stdin is a terminal, the user is prompted for the tenant name

### Command line options

Next to the command line options to specify the platform and tenant there are many more
options to change the default behavior and settings of the `dsh` tool. Some important options are:

* `--dry-run` - Don't actually execute the command
* `--log-level` - Set the log level of the tool (`off`, `error`, `warn`, `info`, `debug` or `trace`)
* `--output-format` or `-o` - Change the default output format (`csv`, `json`, `json-compact`,    
  `plain`, `quiet`, `table`, `table-no-border`, `toml`, `toml-compact`, `yaml`)
* `--quiet` - Don't print any output to the console
* `--verbosity` - Set the amount of metadata generated (`off`, `low`, `medium`, `high`)

For a complete list and detailed information about all command line options use `dsh --help`.

### Environment variables

As with the command line options, there are many environment variables used to change the
default behavior and settings of the `dsh` tool. Some important variables are:

* `DSH_CLI_DRY_RUN` - Don't actually execute the command
* `DSH_CLI_LOG_LEVEL` - Set the log level of the tool
* `DSH_CLI_OUTPUT_FORMAT` - Change the default output format
* `DSH_CLI_QUIET` - Don't print any output to the console
* `DSH_CLI_VERBOSITY` - Set the amount of metadata generated

Note that command line options take precedence over environment variables for the same
configuration item.

For a complete list and detailed information about all environment variables,
see [Environment variables](environment_variables.md), or use the following command:

```shell
> dsh --env-vars
```

For an explanation of a single environment variable (and the value if it is set), use the
`--env-var` option followed by a (part of) the name of the environment variable:

```shell
> dsh --env-var tenant
┌──────────────────────┬────────────────────────────────────────────────────────────────────────────────────────────┐
│ environment variable │ DSH_CLI_TENANT                                                                             │
├──────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────┤
│ value                │ me                                                                                         │
│ secret               │ no                                                                                         │
│ override             │ allowed                                                                                    │
│ default value        │                                                                                            │
│ explanation          │ Target tenant for which commands/capabilities will be executed. This environment variable  │
│                      │ can be overridden via the --tenant command line argument.                                  │
└──────────────────────┴────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Settings

Many settings with regard to the configuration of the `dsh` tool can also be made in the tool
itself, using the `setting` command:

```shell
> dsh setting set default-platform nplz
> dsh setting set default-tenant my-tenant
> dsh setting set dry-run true
> dsh setting set log-level debug
> dsh setting set output-format json
> dsh setting set quiet true
> dsh setting set verbosity
```

To see all settings (with their current values), use:

```shell
> dsh settings
┌────────────────────┬──────────────────────────────────┐
│ setting            │ value                            │
├────────────────────┼──────────────────────────────────┤
│ authentication     │ robot                            │
│ ...                │                                  │
│ settings file name │ /Users/me/.dsh_cli/settings.toml │
│ ...                │                                  │
└────────────────────┴──────────────────────────────────┘
```

Settings are stored in the tool directory (`$HOME/.dsh_cli/settings.toml`).

To reset a setting to its default value, use:

```shell
> dsh setting unset log-level
```

Note that command line options and environment variables take precedence over settings
for the same configuration item.

For a complete list and information about all available settings, use `dsh setting set -h`.

### Prompt

If you did not specify the target platform and/or target tenant (or any other mandatory values)
on the command line, the environment variables or in the settings, the user will be prompted
to provide the required values.

```shell
> dsh secret list
target platform: nplz
target tenant: my-tenant
...
```

In non-interactive use (e.g. in a script), a terminal is not available and an error message
will be shown if mandatory parameters are not provided.

[Authentication and authorization &#x2192;](authentication_authorization.md)
