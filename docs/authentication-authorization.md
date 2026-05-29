# Authentication and authorization

[&#x2190; User guide](user-guide.md)

There are two method to authenticate and authorize for the `cli` tool.

* `single-sign-on` - Interactive use with personal credentials
* `robot` - Non-interactive use with tenant credentials

## Single sign on

The single-sign-on authentication method is intended for interactive use in a command shell on
your workstation. You can log in via KPN Grip and Oauth2, in the same way and with the same
credentials as you log in to the DSH Console web application. These credentials are associated
with an individual user. This allows that all activity is logged and available for accountability
and audits, which is required by KPN policies and similar certifications.

The single-sign-on authentication method is the default for interactive use (stdin is a terminal),
but can also be specified explicitly:

* Via command line option `--authentication sso`.
* via environment variable `DSH_CLI_AUTHENTICATION=sso`.

You can start a session with the command:

```shell
> dsh login nplz
opening login page for platform np-aws-lz-dsh
```

This will direct you to a web page where you can authenticate with your username/password or any
other means available. When logging in was successful you will see the tenants that
you are authorized for.

```shell
you are logged in
authorized tenants: my-tenant1, my-tenant2, my-tenant3, my-tenant4
```

You can be logged in on multiple platforms at the same time, each with its own list of
authenticated tenants. Your session will expire after 30 minutes without any activity,
after which you will have to log in again. You can check for which tenants you are authorized
by running the `dhs` tool without any arguments. This will give you the concise help information,
followed by your current authorizations.

```shell
> dsh
...
Authentications:
  np-aws-lz-dsh    my-tenant1, my-tenant2, my-tenant3, my-tenant4
  prod-aws-lz-dsh  my-tenant1, my-tenant2, my-tenant5
```

Once you're logged in, you can run commands without providing any further credentials:

```shell
> dsh buckets -t my-tenant
┌───────────┬───────────┬─────────────┬────────────────────────────────┬────────────┐
│ bucket id │ versioned │ provisioned │ name                           │ dependants │
├───────────┼───────────┼─────────────┼────────────────────────────────┼────────────┤
│ my-bucket │ false     │ true        │ dev-lz-dsh-my-tenant-my-bucket │            │
│ ...       │           │             │                                │            │
└───────────┴───────────┴─────────────┴────────────────────────────────┴────────────┘
```

With the single-sign-on authentication method it is also possible to run a command for more
than one tenant, or even for all authorized tenants:

```shell
> dsh env find debug --ignore-case --all-tenants
# my-tenant1@np-aws-lz-dsh
┌─────────────┬───┬──────────────────────┬───────┐
│ service id  │ # │ environment variable │ value │
├─────────────┼───┼──────────────────────┼───────┤
│ my-service1 │ 0 │ RUST_LOG             │ debug │
│ my-service3 │ 1 │ LOG_LEVEL            │ DEBUG │
└─────────────┴───┴──────────────────────┴───────┘
# my-tenant2@np-aws-lz-dsh
┌─────────────┬───┬──────────────────────┬───────┐
│ service id  │ # │ environment variable │ value │
├─────────────┼───┼──────────────────────┼───────┤
│ my-service2 │ 0 │ LOGGING              │ debug │
└─────────────┴───┴──────────────────────┴───────┘
```

If needed, you can log out using the command:

```shell
> dsh logout nplz
```

This will direct you to a web page where you can click the `Logout` button.

## Robot password

The robot password authentication method is intended to be used in scripting or CI/CD environments.
With this method you do not log in, but you have to provide the credentials with every command.
The credentials for this method are not associated with a user, but with a tenant. Activities
can therefor not be traced to an individual user, which does not comply with KPN policies.

Each combination of a `platform` and a `tenant` is considered a separate entity with respect
to authentication and authorization, and therefor needs a separate password, called the `robot`
password. This robot password is located in the DSH platform secret store and is named
`system/rest-api-client`. You can get it by logging into the console and navigating to
`Resources > Secrets`. You can also obtain the robot password using the `dsh` tool itself,
by logging in using single-sign-on and running one of the following commands:

```shell
> dsh secret copy system/rest-api-client
> dsh secret show system/rest-api-client --value
```

The robot authentication method is the default for non-interactive use (stdin is not a terminal),
but can also be specified explicitly:

* Via command line option `--authentication robot`.
* via environment variable `DSH_CLI_AUTHENTICATION=robot`.

When running the `dsh` tool using the robot authentication method, you have to somehow provide
the robot password (next to the platform and tenant names). This can be done in a number of
ways, listed below (sorted by decreasing precedence).

### Robot password

1. `--password-file` command line option.
2. `DSH_CLI_PASSWORD_FILE` environment variable.
3. `DSH_CLI_PASSWORD` environment variable.
4. Check entry from the keyring, if available. This can result in a pop-up where the user
   must authenticate for the keyring.
5. If stdin is a terminal, the user is prompted for the password.

### Keyring

When your computer supports it, robot passwords can safely be stored in your computer's keyring.
See `dsh robot -h` for more information. The easiest and safest way to copy a robot password
from the DSH secret store to your local keyring is by using the following command:

```shell
> dsh robot import nplz my-tenant3
```

This will get the robot password from the secret store and copy it to your local keyring
without showing it. To list the available robot passwords in your local keyring, use:

```shell
> dsh robots
┌───────────────┬────────────┐
│ platform      │ tenant     │
├───────────────┼────────────┤
│ np-aws-lz-dsh │ my-tenant1 │
│               │ my-tenant3 │
└───────────────┴────────────┘
```

[Environment variables &#x2192;](environment-variables.md)
