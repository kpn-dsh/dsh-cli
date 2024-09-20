# DSH Api Command Line Tool

## Local installation

The DSH Api Command Line Tool (`dcli`) can be installed on your local machine,
by executing the following command from the project's root directory.

```bash
> cargo install --path dcli
```

In order to run `dcli` make sure that the environment variables described below
are properly set.
Since the command line tool is based on the [DSH API Client](../dsh_api/README.md),
these environment variables are the same as for the client and described in more detail there.

### Environment variables

<table>
    <tr align="top">
        <th align="left">variable</th>
        <th align="left">description</th>
    </tr>
    <tr align="top">
        <td align="top"><code>TRIFONIUS_CONFIG_DIR</code></td>
        <td>
            Set the location of the configuration files directory. 
            Defaults to the current working directory.
            This variable is only used when the <code>trifonius</code> feature is enabled.
        </td>
    </tr>
    <tr align="top">
        <td align="top"><code>DSH_API_PLATFORM</code></td>
        <td>
            Target platform on which the tenant's environment lives.
        </td>
    </tr>
    <tr align="top">
        <td><code>DSH_API_TENANT</code></td>
        <td>
            Tenant id for the target tenant.
        </td>
    </tr>
    <tr align="top">
        <td><code>DSH_API_SECRET_[platform]_[tenant]</code></td>
        <td>
            Secret api token for the target tenant. 
        </td>
    </tr>
    <tr align="top">
        <td><code>DSH_API_USER_[tenant]</code></td>
        <td>
            Group id and user id for the target tenant.
        </td>
    </tr>
</table>
