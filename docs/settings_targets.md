# Settings and targets

[&#x2190; Environment variables](environment_variables.md)

The `dsh` tool stores settings, tokens and certificates in a directory. The settings can be
created and managed via the tool itself, which is the preferred way, but since they are stored
in a `toml` file, they can also be edited (at your own risk) using your favourite text editor.

The settings and targets are typically stored in a subdirectory of the user's home directory
(`$HOME/.dsh_cli`).
This location can be changed by setting the environment variable `DSH_CLI_HOME`.

```
$HOME/.dsh_cli/
 ├── targets/
 │    ├── np-aws-lz-dsh/
 │    │    ├── my-tenant1/
 │    │    │    └── certificates/
 │    │    │         ├── broker-ca.pem
 │    │    │         ├── broker-client.key
 │    │    │         └── broker-client.pem
 │    │    ├── my-tenant2/
 │    │    │    ...
 │    │    └── refresh-token.encrypted
 │    └── prod-aws-lz-dsh/
 │         ...
 └── settings.toml
```

## Targets

The target data (platforms and tenants) is stored in files in the directory
`$HOME/.dsh_cli/targets`.
E.g., for the platform `np-aws-lz-dsh` and the tenant `my-tenant1` the target data is stored in
the file `$HOME/.dsh_cli/targets/np-aws-lz-dsh/my-tenant1`:

When using the `robot` authentication method, each platform/tenant combination also needs a
password. These passwords are not stored in the target files. For security reasons, passwords
are stored in your computers keyring, supported for Mac OsX and Windows. Support for the linux
keyring is available, but not tested yet.

When using the `single-sign-on` authentication method, an encrypted refresh token for the
platform is stored in the file `refresh-token.encrypted` in the platform's directory.

## Settings

The settings are stored in the file `$HOME/.dsh_cli/settings.toml`:

```toml
default-platform = "np-aws-lz-dsh"
default-tenant = "greenbox-dev"
matching-color = "red"
matching-style = "bold"
show-execution-time = false
verbosity = "medium"
```

[Platforms specification &#x2192;](platforms-specification.md)