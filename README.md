# DSH resource management API command line tool

> **NOTE**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`.
> Please include the exact command, the erroneous output and an explanation of the expected output.
> You can also send requests for new features to this e-mail address.

This project provides a tool to call functions on the DSH resource management API from the
command line of your workstation. The following DSH resources can be
listed, queried, searched, created and deleted.

<table>
    <tr align="top">
        <th align="left">resource</th>
        <th>create</th>
        <th>delete</th>
        <th>diff</th>
        <th>find/usage</th>
        <th>list</th>
        <th>show</th>
        <th>update</th>
    </tr>
    <tr align="top">
        <td align="left">api</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">app from the app catalog</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">application / service</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">bucket</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">certificate</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">environment variable</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">image</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">manifest</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">metric</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">platform</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">proxy</td>
        <td align="center"></td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25CE;</td>
    </tr>
    <tr align="top">
        <td align="left">secret</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">topic</td>
        <td align="center"></td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">vhost</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">volume</td>
        <td align="center">&#x25CE;</td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
</table>

Meaning of the dots:

<table>
    <tr>
        <td>&#x25C9;</td>
        <td>Capability is supported for this resource.</td>
    </tr>
    <tr>
        <td>&#x25CE;</td>
        <td>Capability for this resource is still experimental, incomplete or untested. 
            Use at your own risk.</td>
    </tr>
</table>

## Local installation and run

The DSH Api Command Line Tool (`dsh`) can be installed on your local machine
(assuming you have the `rust` tool chain installed),
by executing the following command.

```bash
> cargo install dsh
...
```

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
        <td><code>DSH_CLI_CSV_QUOTE</code></td>
        <td>
            This environment variable specifies the quote character that will be used 
            when printing csv data. If this variable is not provided, the value from the 
            settings file will be used. The default setting is not to use any quote characters.
            Note that the tool will fail when the generated output already contains 
            the quote character.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_CSV_SEPARATOR</code></td>
        <td>
            This environment variable specifies the separator string that will be used 
            when printing csv data. If this variable is not provided, the value from the 
            settings file will be used. The default separator is <code>","</code> (comma).
            Note that the tool will fail when the generated output already contains 
            the csv separator string.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_DRY_RUN</code></td>
        <td>
            If this environment variable is set (to any value) the tool will run in quiet mode, 
            meaning that no output will be produced to the terminal (stdout and stderr).
            The same effect can be accomplished via the <code>--dry-run</code>
            command line argument.
        </td>
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
        <td><code>DSH_CLI_MATCHING_STYLE</code></td>
        <td>
            This environment variable specifies the styling to be used when printing matching 
            results for the find functions, e.q. when matching regular expressions. 
            If this argument is not provided, the value from the settings file will be used.
            Else the default 'bold' will be used.
            <ul>
              <li><code>normal</code> - Matches will be displayed in normal font</li>
              <li><code>bold</code> - Matches will be displayed bold</li>
              <li><code>dim</code> - Matches will be displayed dimmed</li>
              <li><code>italic</code> - Matches will be displayed in italics</li>
              <li><code>underlined</code> - Matches will be displayed underlined</li>
              <li><code>reverse</code> - Mathces will be displayed reversed</li>
            </ul>
            This environment variable can be overridden via the 
            <code>--matching-style</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_NO_ESCAPE</code><br/><code>NO_COLOR</code></td>
        <td>
            When either of these environment variables is set (to any value) 
            the output will not contain any color or other escape sequences.
            This environment variable can be overridden via the 
            <code>--no-color</code> or <code>--no-ansi</code> command line argument.
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
                <li><code>nplz</code> - Non production landing zone</li>
                <li><code>poc</code> - Proof of concept platform</li>
                <li><code>prod</code> - Production landing zone</li>
                <li><code>prodaz</code></li>
                <li><code>prodlz</code></li>
            </ul>
            This environment variable can be overridden via the 
            <code>--platform</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_QUIET</code></td>
        <td>
            When this environment variable is set (to any value) the tool will run in quiet mode, 
            meaning that no output will be produced to the terminal (stdout and stderr).
            This environment variable can be overridden via the 
            <code>--quit</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_SHOW_EXECUTION_TIME</code></td>
        <td>
            When this environment variable is set (to any value) the execution time of the 
            executed function will be shown, in milliseconds.
            The execution time will also be shown when the verbosity level is set to <code>high</code>.
            This environment variable can be overridden via the 
            <code>--show-execution-time</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_TERMINAL_WIDTH</code></td>
        <td>
            When this environment variable is set it will define the maximum terminal width.
            This environment variable can be overridden via the 
            <code>--terminal-width</code> command line argument.
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
    <tr valign="top">
        <td><code>DSH_CLI_VERBOSITY</code></td>
        <td>
            If this option is provided, it will set the verbosity level. 
            The default verbosity setting is <code>low</code>.
            <ul>
                <li><code>off</code> - No logging will be printed</li>
                <li>
                    <code>low</code> - Lowest verbosity level, 
                    only error messages will be printed
                </li>
                <li><code>medium</code> - Medium verbosity level, some info will be printed</li>
                <li>
                    <code>high</code> - Highest verbosity level, all info will be printed, 
                    including the execution time
                </li>
            </ul>
            This environment variable can be overridden via the 
            <code>--verbosity</code> command line argument.
            Also, when the environment variable <code>DSH_CLI_QUIET</code> is set
            or the command line argument <code>--quiet</code> is provided, nothing will be printed.
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

The `dsh` tool has a strong dependency on the [`dsh-api`](dsh_api) library,
that provides the client for the DSH resource management API.
This library is still being worked on, and it is not (yet) published to `crates.io`.
Hence, at this time `dsh` depends on the [github](https://github.com/kpn-dsh/dsh-api)
version of the library.

```toml
# Cargo.toml
dsh-api = { git = "ssh://git@github.com/kpn-dsh/dsh-api.git", branch = "0.2.0" }
```

When developing simultaneously on `dsh` and `dsh-api` consider changing the library dependency
to your local copy.

```toml
# Cargo.toml
dsh-api = { path = "../dsh-api/dsh-api" }
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
