# Settings and targets

The `dsh` tool stores settings and configuration in settings files,
and platform/tenant credentials in target files.
The settings and target files can be created and managed via the tool itself,
which is the preferred way,
but since they are `toml` files, they can also be edited (at your own risk)
using your favourite text editor.

The settings and targets are typically stored in a subdirectory of the user's home directory
(`$HOME/.dsh_cli`).
This location can be changed by setting the environment variable `DSH_CLI_HOME`.

## Settings

The settings are stored in the file `$HOME/.dsh_cli/settings.toml`:

```toml
default-platform = "np-aws-lz-dsh"
default-tenant = "greenbox-dev"
matching-style = "bold"
show-execution-time = false
verbosity = "medium"
```

## Targets

The target data (platforms and tenants) is stored in files in the directory
`$HOME/.dsh_cli/targets`.
For each combination of a platform and a tenant there is a separate file.
E.g., for the platform `np-aws-lz-dsh` and the tenant `greenbox-dev` the target data is stored in
the file `$HOME/.dsh_cli/targets/np-aws-lz-dsh.greenbox-dev.toml`:

```toml
platform = "np-aws-lz-dsh"
tenant = "greenbox-dev"
```

Each platform/tenant combination also needs a password.
The passwords are not stored in the target files.
For security reasons, passwords are stored in your computers keychain,
supported for Mac OsX and Windows.
Support for the linux keychain is available, but not tested yet.
