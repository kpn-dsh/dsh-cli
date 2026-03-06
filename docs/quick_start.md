# Quick start

[&#x2190; README](../README.md)

### Prompts

When the `dsh` tool is installed properly, you can run it by simply typing a command and
the tool will prompt you for the required parameters.
To get a list of the configured secrets for `my-tenant` on platform `nplz`, just type:

```bash
> dsh secret list
target platform: nplz
target tenant: my-tenant
please log in to platform np-aws-lz-dsh using the 'dsh login' command
user is not authenticated
```

To login, just type the following command:

```bash
dcli> dsh login
target platform: nplz
login to platform np-aws-lz-dsh
opening login page for platform np-aws-lz-dsh
```

Your browser will open automatically, and you are requested to login using the 2 fase
authentication process. When login succeeded, you will be notified on the console and the
tenants that you have access to on this platform will be listed.

```bash
dcli> dsh login
target platform: nplz
login to platform np-aws-lz-dsh
opening login page for platform np-aws-lz-dsh
you are logged in as schel104
authorized tenants: my-tenant, ...
```

No try again, to finally see the results.

```bash
> dsh secret list
target platform: nplz
target tenant: my-tenant
target my-tenant@np-aws-lz-dsh
list all services with their parameters
┌─────────────────────────────────────────┐
│ secret ids (1)                          │
├─────────────────────────────────────────┤
│ api-key                                 │
│ ...                                     │
└─────────────────────────────────────────┘
```

### Command line arguments

In most cases, especially when the `dsh` tool is not run from a terminal,
it is more convenient to provide the required parameters explicitly via the command line.

You can get the above list of all secrets for tenant `my-tenant`
on platform `np-aws-lz-dsh` by typing the following command:

```bash
> dsh secret list --platform np-aws-lz-dsh --tenant my-tenant
...
```

For the most used arguments there are shortcuts defined. The above command can
also be run as follows:

```bash
> dsh secret list -p np-aws-lz-dsh -t my-tenant
...
```

### Environment variables

Even more convenient is providing the required parameters via environment variables:

```bash
> export DSH_CLI_PLATFORM=np-aws-lz-dsh
> export DSH_CLI_TENANT=my-tenant
```

Now you can get the same list of secrets by just typing:

```bash
> dsh secret list
...
```

### Settings and targets

If you work with more than one platform and/or tenant,
providing the parameters and passwords via prompts, command line arguments or
environment variables quickly becomes tedious.
Luckily, you can also set a default platform and default tenant:

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

[User guide &#x2192;](user_guide.md)
