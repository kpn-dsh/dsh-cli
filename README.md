# Trifonius engine project

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
        <td><code>TRIFONIUS_TARGET_TENANT</code></td>
        <td>Tenant id for the target tenant. The target tenant is the tenant whose resources 
            will be managed by Trifonius.</td>
    </tr>
    <tr align="top">
        <td><code>TRIFONIUS_TARGET_TENANT_[tenant]_SECRET</code></td>
        <td>
            Secret api token for the target tenant. 
            Note that the placeholder <code>[tenant]</code> needs to be substituted 
            with the tenant name in all capitals, and hyphens (<code>-</code>) 
            replaced with underscores (<code>_</code>).
            E.g. if the tenant name is <code>greenbox-dev</code>, the environment variable must be
            <code>TRIFONIUS_TARGET_TENANT_GREENBOX_DEV_SECRET = "..."</code>.
        </td>
    </tr>
    <tr align="top">
        <td><code>TRIFONIUS_TARGET_TENANT_[tenant]_USER</code></td>
        <td>
            Group id and user id for the target tenant.
            Note that the placeholder <code>[tenant]</code> needs to be substituted 
            with the tenant name in all capitals, and hyphens (<code>-</code>) 
            replaced with underscores (<code>_</code>).
            E.g. if the tenant name is <code>greenbox-dev</code>, the environment variable must be
            <code>TRIFONIUS_TARGET_TENANT_GREENBOX_DEV_USER = "1903:1903"</code>.
        </td>
    </tr>
</table>

This allows the creation of the default token fetcher, as is shown in the following code fragment:

```rust
use trifonius_engine::resource::resource_descriptor::ResourceType;
use trifonius_engine::resource::resource_registry::ResourceRegistry;
use trifonius_engine::DEFAULT_TARGET_CLIENT_FACTOR;

#[tokio::main]
async fn main() {
    let resource_registry = ResourceRegistry::create(&DEFAULT_TARGET_CLIENT_FACTOR).unwrap();
    let topic_descriptors = resource_registry.resource_descriptors_by_type(ResourceType::Topic).unwrap();
    for topic_descriptor in topic_descriptors {
        println!("{}", topic_descriptor);
    }
}

```

## How to publish to Artifactory

On the KPN Artifactory we have a Cargo repository dedicated
for [DSH-IUC](https://artifacts.kpn.org/ui/repos/tree/General/cargo-dsh-iuc-local).
LDAP Group `dig_dsh_iuc` has write access to this repository and is allowed to publish artifacts.

As in .cargo/config.toml, the default registry points
towards [DSH-IUC](https://artifacts.kpn.org/ui/repos/tree/General/cargo-dsh-iuc-local), you can
publish your crate by running:

Login to Artifactory (one time):

```bash
make login
```

To publish all crates, run:

```bash
make publish
```

See make help for more options:

```bash
make help
```