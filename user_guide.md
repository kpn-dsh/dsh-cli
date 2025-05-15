# User guide

[&#x2190; Quick start](quick_start.md)

When installation is complete you should be able to start the `dsh` tool from the command line.

```bash
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
  -p, --platform <PLATFORM>   Provide target platform [possible values: np-aws-lz-dsh, poc-aws-dsh, prod-aws-dsh,
                              prod-aws-lz-dsh, prod-aws-lz-laas, prod-azure-dsh]
  -t, --tenant <TENANT>       Provide target tenant
      --password-file <FILE>  Provide target password file name
      --dry-run               Execute in dry-run mode
      --force                 Force changes without confirmation
  -h, --help                  Print help (see more with '--help')
  -V, --version               Print version

Output options:
  -o, --output-format <FORMAT>  Set output format [possible values: csv, json, json-compact, plain, quiet, table,
                                table-no-border, toml, toml-compact, yaml]
  -q, --quiet                   Run in quiet mode
  -v, --verbosity <VERBOSITY>   Set verbosity level [possible values: off, low, medium, high]

Settings:
  default platform  np-aws-lz-dsh / nplz
  default tenant    greenbox
  settings file     /Users/wilbert/Workspaces/dsh/dcli/.dsh_cli/settings.toml

Environment variables:
  DSH_CLI_HOME           /Users/wilbert/Workspaces/dsh/dcli/.dsh_cli
  DSH_CLI_PASSWORD_FILE  /Users/wilbert/Workspaces/dsh/dcli/np-aws-lz-dsh.greenbox.pwd
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
  To get a list of all supported platforms, use the command `dsh platform list`.
* `tenant` - The target tenant that is responsible for the resources and services
  that are deployed on the DSH. Tenants will often have resources and services
  on more than one platform, e.g. a development platform and a production platform.
* `password` - Each combination of a `platform` and a `tenant` is considered
  a separate entity with respect to authentication and authorization,
  and therefor needs a separate `password`.
  This password can be obtained by logging in to the DSH console web application
  for the `platform` and `tenant`, and selecting the `Resources > Secrets` menu.
  There it will be listed as `system/rest-api-client`.

Each `dsh` function that needs to access the web service must provide these three values.
The values can be provided as follows:

1. If the command line argument is given, use its value,
1. else if the environment variable is specified, use its value,
1. else if the value is defined in the settings/targets files in the `dsh` tool directory,
   use that value,
1. else if the `dsh` tool is used from a terminal, the user will be prompted for the value.

The `dsh` tool will check these possible sources for the target parameters in this order.
Passwords can also be stored in the computer's keyring
(currently only available on OsX and Windows).

Functions that do not need to access the web service do not require these target parameters.

### Command line arguments

* `--platform` or `-p` - Specify the target platform by either the full name
  or the alias (e.g. `np-aws-lz-dsh` or `nplz`).
* `--tenant` or `-t` - Specify the target tenant name.
* `--password-file` - Specify the name of a file containing the target password.
  This can either be an absolute file name
  or a file name relative to the working directory.

```bash
> dsh --platform nplz --tenant my-tenant --password-file ~/.password secret list
list all secret ids
┌─────────────────────────────────────────┐
│ secret ids (1)                          │
├─────────────────────────────────────────┤
│ api-key                                 │
│ ...                                     │
└─────────────────────────────────────────┘
```

Specifying the password directly as a command line argument is a security hazard
and is not supported.

### Environment variables

If you did not specify the target platform and/or target tenant on the command line,
the `dsh` tool will try to get the values from environment variables:

* `DSH_CLI_PLATFORM` - Specify the target platform by providing either the full name or the alias
  (e.g. `np-aws-lz-dsh` or `nplz`).
* `DSH_CLI_TENANT` - Specify the tenant name.
* `DSH_CLI_PASSWORD_FILE` - Specify the name of a file containing the target password.
  This can either be an absolute file name
  or a relative file name from the working directory.
* `DSH_CLI_PASSWORD` - The password can also be specified directly from an environment variable.

```bash
> export DSH_CLI_PLATFORM=np-aws-lz-dsh
> export DSH_CLI_TENANT=my-tenant
> export DSH_CLI_PASSWORD="..."
> dsh secret list
...
```

For detailed information about all environment variables,
see [Environment variables](environment_variables.md).

### Settings and targets

If you did not specify the target platform and/or target tenant on the command line
and the environment variables are not defined,
the `dsh` tool will try the `default-platform` and `default-tenant` values in the settings file
and the password from the targets files. See [Settings and targets](settings_targets.md)
for more information.

### Prompt

If you did not specify the target platform and/or target tenant on the command line,
the environment variables are not defined and the `default-platform` and `default-tenant`
values are not set in the settings and target files,
the user will be prompted to provide the required values from the terminal.

```bash
> dsh secret list
target platform: nplz
target tenant: my-tenant
password for tenant my-tenant@np-aws-lz-dsh: ********
...
```

In non-interactive use, e.g. in a script, a terminal is not available and an error message
will be shown.

[Environment variables &#x2192;](environment_variables.md)
