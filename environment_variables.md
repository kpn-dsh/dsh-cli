# DSH resource management API command line tool

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
                <li><code>json-compact</code> - Output will be in compact json format</li>
                <li><code>plain</code> - Output will be formatted as plain text</li>
                <li><code>quiet</code> - No output will be generated</li>
                <li><code>table</code> - Output will be formatted as a table with borders</li>
                <li>
                    <code>table-no-border</code> - Output will be formatted as a table 
                    without borders
                </li>
                <li><code>toml</code> - Output will be in toml format</li>
                <li><code>toml-compact</code> - Output will be in compact toml format</li>
                <li><code>yaml</code> - Output will be in yaml format</li>
            </ul>
            This environment variable can be overridden via the 
            <code>--output-format</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PASSWORD</code></td>
        <td>
            This environment variable specifies the secret api token/password for the target tenant. 
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
