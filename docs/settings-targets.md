# Settings and targets

[&#x2190; Environment variables](environment-variables.md)

The `dsh` tool stores settings, tokens and certificates in a tool directory. This directory is
typically located in a subdirectory of the user's home directory (`$HOME/.dsh_cli`), but if
required this location can be changed by setting the environment variable `DSH_CLI_HOME`.

The directory structure is as follows:

```
$HOME/.dsh_cli/
 ├── latest-release.toml
 ├── settings.toml
 └── targets/
      ├── np-aws-lz-dsh/
      │    ├── my-tenant1/
      │    │    └── bundles/
      │    │         └── my-bundle/
      │    │              ├── bundle.toml
      │    │              ├── ca.key
      │    │              ├── ca.pem
      │    │              ├── client.key
      │    │              ├── client.pem
      │    │              ├── server.key
      │    │              └── server.pem
      │    ├── my-tenant2/
      │    │    ...
      │    └── refresh-token.encrypted
      └── prod-aws-lz-dsh/
           ...
```

## `latest-release.toml`

The file `$HOME/.dsh_cli/latest-release.toml` contains information about the latest GitHub
release of the tool. This file should not be edited or changed.

## `settings.toml`

The settings are stored in the file `$HOME/.dsh_cli/settings.toml`. Some example entries are:

```toml
default-platform = "np-aws-lz-dsh"
default-tenant = "greenbox-dev"
matching-color = "red"
matching-style = "bold"
show-execution-time = false
verbosity = "medium"
```

These settings can be created and managed via the `dsh setting` command in the tool itself,
which is the preferred way. Since the settings are stored in a `toml` file, they can also be
edited (at your own risk) using your favourite editor.

## `targets/`

The `$HOME/.dsh_cli/targets` directory contain a subdirectory for each platform. The
platform directories contain subdirectories for each tenant on the platform.

E.g., for the platform `np-aws-lz-dsh` and the tenant `my-tenant1` the target data is stored in
the directory `$HOME/.dsh_cli/targets/np-aws-lz-dsh/my-tenant1`. The target data currently
consists of data related to proxy bundles (in the `bundles` subdirectory).

When using the `single-sign-on` authentication method, a short-lived encrypted refresh token for
the platform is stored in the file `refresh-token.encrypted` in the platform directory.

Note that all directories are created only when needed, e.g. when the user logged in via
single-sign-on or when a proxy was created.

[Platforms specification &#x2192;](platforms-specification.md)