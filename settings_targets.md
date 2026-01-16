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
 │    ├── platform1/
 │    │    ├── tenant1/
 │    │    │    └── certificates/
 │    │    │         ├── broker-ca.pem
 │    │    │         ├── broker-client.key
 │    │    │         └── broker-client.pem
 │    │    ├── tenant2/
 │    │    │    ...
 │    │    └── refresh-token.encrypted
 │    └── platform2/
 │         ...
 └── settings.toml
```

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

## Targets

> **WARNING**  
> The target mechanism doesn't work in this version. If you need this, use version 0.8.0 for now.
> The target capability in its current form is no longer necessary, since the single-sign-on
> authentication mechanism works much better. The target implementation will be reconsidered for
> future versions.

The target data (platforms and tenants) is stored in files in the directory
`$HOME/.dsh_cli/targets`.
For each combination of a platform and a tenant there is a separate subdirectory.
E.g., for the platform `np-aws-lz-dsh` and the tenant `greenbox-dev` the target data is stored in
the file `$HOME/.dsh_cli/targets/np-aws-lz-dsh/greenbox-dev`:

When using the `robot` authentication method, each platform/tenant combination also needs a
password. These passwords are not stored in the target files. For security reasons, passwords
are stored in your computers keychain, supported for Mac OsX and Windows. Support for the linux
keychain is available, but not tested yet.

When using the `single-sign-on` authentication method, an encrypted refresh token for the
platform needs to be stored in the platforms directory.

[Platforms specification &#x2192;](platforms-specification.md)
