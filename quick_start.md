# Quick start

### Prompts

When the `dsh` tool is installed properly, you can run it by simply typing a command and
the tool will prompt you for the required parameters.
To get a list of the configured secrets for `my-tenant` on `my-platform`, just type:

```bash
> dsh secret list
target platform: np-aws-lz-dsh
target tenant: my-tenant
password for tenant my-tenant@np-aws-lz-dsh: ********
┌─────────────────────────────────────────┐
│ secret ids (1)                          │
├─────────────────────────────────────────┤
│ api-key                                 │
│ ...                                     │
└─────────────────────────────────────────┘
```

The password can be obtained by logging in to the DSH console web application for `np-aws-lz-dsh`
and selecting `my-tenant`. Then go to the `Resources > Secrets` menu.
The password will be listed as `system/rest-api-client`.

### Command line arguments

In most cases, especially when the `dsh` tool is not run from a terminal,
it is more convenient to provide the required parameters explicitly via the command line.
For security reasons, the password cannot be provided directly from the command line,
therefor you have to create a password file containing the password.

```bash
> touch .password
> edit .password
```

Now you can get the list of all secrets for tenant `my-tenant`
on platform `np-aws-lz-dsh` by typing the following command:

```bash
> dsh secret list --platform np-aws-lz-dsh --tenant my-tenant --password-file .password
...
```

### Environment variables

Even more convenient is providing the required parameters via environment variables:

```bash
> export DSH_CLI_PLATFORM=np-aws-lz-dsh
> export DSH_CLI_TENANT=my-tenant
> export DSH_CLI_PASSWORD="..."
```

Now you can get the same list of secrets by just typing:

```bash
> dsh secret list
...
```

In this case the password can be provided by an environment variable directly,
but you can also provide it in a file as with the command line arguments:

```bash
> export DSH_CLI_PASSWORD_FILE=.password
```

### Settings and targets

If you work with more than one platform and/or tenant,
providing the parameters and passwords via prompts, command line arguments or
environment variables quickly becomes tedious.
It might be easier to use the `dsh` tool's capabilities to manage target platforms and tenants
and to define default settings.

To create a new target, type:

```bash
> dsh target new np-aws-lz-dsh my-tenant
create new target configuration
enter password:
target my-tenant@np-aws-lz-dsh created
```

This will store the password in your platform's keyring.
Once it is stored there, you don't have to provide it with each invocation:

```bash
> dsh secret list --platform np-aws-lz-dsh --tenant my-tenant
...
```

Finally, you can also set a default platform and default tenant:

```bash
> dsh setting set default-platform np-aws-lz-dsh
default platform set to np-aws-lz-dsh
> dsh setting set default-tenant my-tenant
default tenant set to my-tenant
```

Now if you don't provide the platform or tenant via the command line arguments or environment
variables, the default settings will be used:

```bash
> dsh secret list
...
```
