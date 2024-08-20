# Trifonius Command Line Tool

## Local installation

The Trifonius Command Line Tool (`tcli`) can be installed on your local machine,
by executing the following command from the trifonius Engine project's root directory.

```bash
> cargo install --path cli
```

In order to run `tcli` make sure that the environment variables described below
are properly set.
Since the command line tool is based on the [Trifonius DSH API Client](../dsh_api/README.md),
these environment variables are the same as for the client and described in more detail there.

### Environment variables

<table>
    <tr align="top">
        <th align="left">variable</th>
        <th align="left">description</th>
    </tr>
    <tr align="top">
        <td align="top"><code>TRIFONIUS_CONFIG_DIR</code></td>
        <td>Set the location of the configuration files directory. Defaults to the current working 
            directory.
        </td>
    </tr>
    <tr align="top">
        <td align="top"><code>TRIFONIUS_TARGET_PLATFORM</code></td>
        <td>
            Target platform on which the tenant's environment lives.
        </td>
    </tr>
    <tr align="top">
        <td><code>TRIFONIUS_TARGET_TENANT</code></td>
        <td>
            Tenant id for the target tenant.
        </td>
    </tr>
    <tr align="top">
        <td><code>TRIFONIUS_TARGET_TENANT_[tenant]_SECRET</code></td>
        <td>
            Secret api token for the target tenant. 
        </td>
    </tr>
    <tr align="top">
        <td><code>TRIFONIUS_TARGET_TENANT_[tenant]_USER</code></td>
        <td>
            Group id and user id for the target tenant.
        </td>
    </tr>
</table>
