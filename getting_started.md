# Getting started

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
