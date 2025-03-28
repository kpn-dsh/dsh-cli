# Environment variables

[&#x2190; User guide](user_guide.md)

The `dsh` tool can be run entirely from the command line and
all configurations and parameters can be specified via command line arguments.
However, especially when using `dsh` interactively,
it is much more convenient to make some settings persistent via environment variables.

<table>
    <tr valign="top">
        <th align="left">variable</th>
        <th align="left">description</th>
    </tr>
    <tr valign="top">
        <td><code>DSH_API_PLATFORMS_FILE</code></td>
        <td>
            Set this environment variable to override the default list of available platforms.
            The value of the environment variable must be the name 
            of the alternative platforms file. It can either be an absolute file name, 
            or a relative file name from the working directory. 
            When this environment variable is set, the normal list of default platforms 
            will <em>not</em> be included. If you need these too, make sure that you also 
            include the default platforms in your platforms file.
            See the bottom of this page for more information.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_CSV_QUOTE</code></td>
        <td>
            This environment variable specifies the quote character that will be used 
            when printing csv data. If this variable is not provided, the value from the 
            settings file will be used. The default setting is not to use any quote characters.
            Note that the <code>dsh</code> tool will fail when the generated output already 
            contains the quote character.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_CSV_SEPARATOR</code></td>
        <td>
            This environment variable specifies the separator string that will be used 
            when printing csv data. If this variable is not provided, the value from the 
            settings file will be used. The default separator is <code>","</code> (comma).
            Note that the <code>dsh</code> tool will fail when the generated output already 
            contains the csv separator string.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_DRY_RUN</code></td>
        <td>
            If this environment variable is set (to any value) the <code>dsh</code> tool will not 
            call any api operations that could potentially make changes, like delete, create or 
            change. The input parameters will be validated and checked.
            The same effect can be accomplished via the <code>--dry-run</code>
            command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_ERROR_COLOR</code></td>
        <td>
            This environment variable specifies the color to be used when printing error messages. 
            If this variable is not set, the settings file will be checked for the 
            <code>error-color</code> entry. Else the default color <code>red</code> will be used.
            The supported colors are:
            <ul>
              <li>
                <code>normal</code> - matches will be displayed in the terminals default 
                foreground color,
              </li>
              <li><code>red</code> - matches will be displayed in red,</li>
              <li><code>green</code> - matches will be displayed in green,</li>
              <li><code>blue</code> - matches will be displayed in blue.</li>
            </ul>
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_ERROR_STYLE</code></td>
        <td>
            This environment variable specifies the styling to be used when printing error 
            messages. If this variable is not set, the settings file will be checked for the 
            <code>error-style</code> entry. Else the default value <code>bold</code> will be used.
            The supported styles are:
            <ul>
              <li><code>normal</code> - no styling will be applied,</li>
              <li><code>bold</code> - matches will be displayed in a bold typeface,</li>
              <li><code>dim</code> - matches will be displayed dimmed,</li>
              <li><code>italic</code> - matches will be displayed in an italics typeface,</li>
              <li><code>underlined</code> - matches will be displayed underlined,</li>
              <li><code>reverse</code> - matches will be displayed reversed.</li>
            </ul>
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_HOME</code></td>
        <td> 
            Use this environment variable to change the location where <code>dsh</code> 
            stores its settings and targets information. 
            The default location is <code>$HOME/.dsh_cli</code>.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_LOG_LEVEL</code></td>
        <td> 
            Use this environment variable to set the log level of the <code>dsh</code> tool.
            The available log levels are:
            <ul>
              <li><code>off</code> - logging is off,</li>
              <li><code>error</code> - only errors will be logged,</li>
              <li><code>warn</code> - warnings and errors will be logged,</li>
              <li><code>info</code> - high level info, warnings and errors will be logged,</li>
              <li><code>debug</code> - debug info, high level info, warnings and errors 
                will be logged,</li>
              <li><code>trace</code> - tracing info, debug info, high level info, warnings 
                and errors will be logged.</li>
            </ul>
            If this argument is not provided, the settings file will be checked. 
            When the <code>--log-level</code> command line argument is provided this will override
            this environment variable or the value in the settings file.
            The default log level is <code>error</code>.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_LOG_LEVEL_API</code></td>
        <td> 
            Use this environment variable to set the log level for the functions 
            in the library crate <code>dsh_api</code>, that supports the <code>dsh</code> tool.
            For the available log levels see the description of the 
            <code>DSH_CLI_LOG_LEVEL</code> environment variable.<br/>
            If this argument is not provided, the settings file will be checked. 
            When the <code>--log-level-api</code> command line argument is provided this will 
            override this environment variable or the value in the settings file.
            The default log level is <code>error</code>.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_LOG_LEVEL_SDK</code></td>
        <td> 
            Use this environment variable to set the log level for the functions 
            in the library crate <code>dsh_sdk</code>, that supports the <code>dsh</code> tool.
            For the available log levels see the description of the 
            <code>DSH_CLI_LOG_LEVEL</code> environment variable.<br/>
            If this argument is not provided, the settings file will be checked. 
            When the <code>--log-level-sdk</code> command line argument is provided this will 
            override this environment variable or the value in the settings file.
            The default log level is <code>error</code>.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_MATCHING_COLOR</code></td>
        <td>
            This environment variable specifies the color to be used when printing matching 
            results for the find functions, e.q. when matching regular expressions. 
            If this variable is not set, the settings file will be checked for the 
            <code>matching-color</code> entry. 
            Else the default color <code>green</code> will be used.
            See environment variable <code>DSH_CLI_ERROR_COLOR</code> for the supported colors.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_MATCHING_STYLE</code></td>
        <td>
            This environment variable specifies the styling to be used when printing matching 
            results for the find functions, e.q. when matching regular expressions. 
            If this variable is not set, the settings file will be checked for the 
            <code>matching-style</code> entry. 
            Else the default value <code>bold</code> will be used.
            See environment variable <code>DSH_CLI_ERROR_STYLE</code> for the supported styles.
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
        <td><code>DSH_CLI_NO_HEADERS</code></td>
        <td>
            When this environment variables is set (to any value) 
            the output will not contain headers.
            This environment variable can be overridden via the 
            <code>--no-headers</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_OUTPUT_FORMAT</code></td>
        <td>
            This option specifies the format used when printing the output. 
            If this argument is not provided, the value from the settings file will be used. 
            Else, when <code>stdout</code> is a terminal the default 
            <code>table</code> will be used, or if <code>stdout</code> is not a terminal 
            the value <code>json</code> will be used.
            The supported values are:
            <ul>
                <li><code>csv</code> - output will be formatted as comma separated values,</li>
                <li><code>json</code> - output will be in json format,</li>
                <li><code>json-compact</code> - output will be in compact json format,</li>
                <li><code>plain</code> - output will be formatted as plain text,</li>
                <li><code>quiet</code> - no output will be generated,</li>
                <li><code>table</code> - output will be formatted as a table with borders,</li>
                <li>
                    <code>table-no-border</code> - output will be formatted as a table 
                    without borders,
                </li>
                <li><code>toml</code> - output will be in toml format,</li>
                <li><code>toml-compact</code> - output will be in compact toml format,</li>
                <li><code>yaml</code> - output will be in yaml format.</li>
            </ul>
            This environment variable can be overridden via the 
            <code>--output-format</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_PASSWORD</code></td>
        <td>
            This environment variable specifies the secret api token/password for the target 
            tenant. Note that when the environment variable <code>DSH_CLI_PASSWORD_FILE</code> 
            or the argument <code>--password-file</code> command line argument is provided,
            this environment variable will never be used. 
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
            Target platform on which the tenants environment lives.
            The supported platforms are:
            <ul>
                <li>
                    <code>np-aws-lz-dsh / nplz</code> - 
                    staging platform for KPN internal tenants,
                </li>
                <li>
                    <code>poc-aws-dsh / poc</code> - 
                    staging platform for non KPN tenants,
                </li>
                <li>
                    <code>prod-aws-dsh / prod</code> - 
                    production platform for non KPN tenants,
                </li>
                <li>
                    <code>prod-aws-lz-dsh / prodlz</code> - 
                    production platform for KPN internal tenants,
                </li>
                <li>
                    <code>prod-aws-lz-laas / prodls</code> - 
                    production platform for logstash as a service,
                </li>
                <li>
                    <code>prod-azure-dsh / prodaz</code> - 
                    production platform for non KPN tenants.
                </li>
            </ul>
            This environment variable can be overridden via the 
            <code>--platform</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_QUIET</code></td>
        <td>
            When this environment variable is set (to any value) the <code>dsh</code> tool 
            will run in quiet mode, meaning that no output will be produced to the terminal 
            (<code>stdout</code> and <code>stderr</code>).
            This environment variable can be overridden via the 
            <code>--quit</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_SHOW_EXECUTION_TIME</code></td>
        <td>
            When this environment variable is set (to any value) the execution time of the 
            executed function will be shown, in milliseconds.
            The execution time will also be shown when the verbosity level is set to 
            <code>high</code>. This environment variable can be overridden via the 
            <code>--show-execution-time</code> command line argument.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_STDERR_COLOR</code></td>
        <td>
            This environment variable specifies the color to be used when printing explanations 
            and metadata. 
            If this variable is not set, the settings file will be checked for the 
            <code>stderr-color</code> entry. Else the default color for the terminal will be used.
            See environment variable <code>DSH_CLI_ERROR_COLOR</code> for the supported colors.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_STDERR_STYLE</code></td>
        <td>
            This environment variable specifies the styling to be used when printing explanations 
            and metadata. 
            If this variable is not set, the settings file will be checked for the 
            <code>stderr-style</code> entry. Else the default value <code>dim</code> will be used.
            See environment variable <code>DSH_CLI_ERROR_STYLE</code> for the supported styles.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_STDOUT_COLOR</code></td>
        <td>
            This environment variable specifies the color to be used when printing results. 
            If this variable is not set, the settings file will be checked for the 
            <code>stdout-color</code> entry. Else the default color for the terminal will be used.
            See environment variable <code>DSH_CLI_ERROR_COLOR</code> for the supported colors.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_STDOUT_STYLE</code></td>
        <td>
            This environment variable specifies the styling to be used when printing results. 
            If this variable is not set, the settings file will be checked for the 
            <code>stdout-style</code> entry. Else the default value <code>normal</code> 
            (no styling) will be used.
            See environment variable <code>DSH_CLI_ERROR_STYLE</code> for the supported styles.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_SUPPRESS_EXIT_STATUS</code></td>
        <td>
            If this environment variable is set (to any value) the <code>dsh</code> tool will  
            always return exit status 0, even when an error has occurred. 
            This can be useful in scripting environments.
            The same effect can be accomplished via the <code>--suppress-exit-code</code>
            command line argument or the <code>suppress-exit-status</code> setting.
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
            The supported verbosity levels are:
            <ul>
                <li><code>off</code> - no logging will be printed,</li> 
                <li><code>low</code> - only error messages will be printed,</li>
                <li><code>medium</code> - some info will be printed,</li>
                <li><code>high</code> - all info will be printed.</li>
            </ul>
            This environment variable can be overridden via the 
            <code>--verbosity</code> command line argument.
            Also, when the environment variable <code>DSH_CLI_QUIET</code> is set
            or the command line argument <code>--quiet</code> is provided, nothing will be printed.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_WARNING_COLOR</code></td>
        <td>
            This environment variable specifies the color to be used when printing warnings. 
            If this variable is not set, the settings file will be checked for the 
            <code>warning-color</code> entry. Else the default color <code>blue</code> will be 
            used. See environment variable <code>DSH_CLI_ERROR_COLOR</code> for the supported 
            colors.
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_CLI_WARNING_STYLE</code></td>
        <td>
            This environment variable specifies the styling to be used when printing warnings. 
            If this variable is not set, the settings file will be checked for the 
            <code>warning-style</code> entry. Else the default value <code>bold</code> will be 
            used. See environment variable <code>DSH_CLI_ERROR_STYLE</code> for the supported 
            styles.
        </td>
    </tr>

</table>

[Settings and targets &#x2192;](settings_targets.md)
