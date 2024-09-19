# DSH API Client

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
            <ul>
                <li><code>nplz</code>Non production landing zone</li>
                <li><code>poc</code>Proof of concept platform</li>
                <li><code>prod</code>Production landing zone</li>
                <li><code>prodaz</code></li>
                <li><code>prodlz</code></li>
            </ul>
        </td>
    </tr>
    <tr align="top">
        <td><code>DSH_API_TENANT</code></td>
        <td>Tenant id for the target tenant. The target tenant is the tenant whose resources 
            will be managed via the api.</td>
    </tr>
    <tr align="top">
        <td><code>DSH_API_SECRET_[platform]_[tenant]</code></td>
        <td>
            Secret api token for the target tenant. 
            The placeholders <code>[platform]</code> and <code>[tenant]</code> 
            need to be substituted with the platform name and the tenant name in all capitals, 
            with hyphens (<code>-</code>) replaced by underscores (<code>_</code>).
            E.g. if the platform is <code>nplz</code> and the tenant name is 
            <code>greenbox-dev</code>, the environment variable must be
            <code>DSH_API_SECRET_NPLZ_GREENBOX_DEV = "..."</code>.
        </td>
    </tr>
    <tr align="top">
        <td><code>DSH_API_USER_[tenant]</code></td>
        <td>
            Group id and user id for the target tenant.
            The placeholder <code>[tenant]</code> needs to be substituted 
            with the tenant name in all capitals, with hyphens (<code>-</code>) 
            replaced by underscores (<code>_</code>).
            E.g. if the tenant name is <code>greenbox-dev</code>, the environment variable must be
            <code>DSH_API_USER_GREENBOX_DEV = "1903:1903"</code>.
        </td>
    </tr>
</table>

## How to publish to Artifactory

On the KPN Artifactory we have a Cargo repository dedicated
for [DSH-IUC](https://artifacts.kpn.org/ui/repos/tree/General/cargo-dsh-iuc-local).
LDAP Group `dig_dsh_iuc` has write access to this repository and is allowed to publish artifacts.

As in .cargo/config.toml, the default registry points
towards [DSH-IUC](https://artifacts.kpn.org/ui/repos/tree/General/cargo-dsh-iuc-local), you can
publish your crate by running:

Login to Artifactory (one time):

```bash
> make login
```

To publish all crates, run:

```bash
> make publish
```

See make help for more options:

```bash
> make help
Targets Cargo:
  build:                 Build all cargo packages
  login:                 Login to KPN Artifactory for the cargo registry
  publish:               Publish to KPN Artifactory
  publish-allow-dirty:   Publish to KPN Artifactory without checking for uncommited files
  publish-dry-run:       Dry-run the publish to KPN Artifactory
  test:                  Run all cargo tests
  test-<package>:        Run tests for a single cargo package
>
```