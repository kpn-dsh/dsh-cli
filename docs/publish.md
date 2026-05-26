# Publish

[&#x2190; Developers](developers.md)

This page is targeted at developers who wish to publish the `dsh` tool to `crates.io`.
In this explanation it is assumed that all your local code is committed, pushed and
merged to `GitHub`.

Before publishing, you must create a tag in your repository for the
published version. Use the version number without any prefix as the tag name, e.g. `0.10.0`.
This tag is required when creating a `GitHub` release as described on the next page.

## Publish `dsh_api`

Since the `dsh` tool relies on the `dsh_api` crate for most of its capabilities, releasing a new
version of `dsh` often goes hand in hand with releasing a new version of `dsh_api`.
Furthermore, `dsh` cannot be published to `crates.io` while depending on a local
version of `dsh_api`. So the first step for releasing `dsh` is making sure that the
required version of the `dsh_api` crate is published to `crates.io`.

See the [`README.md`](https://github.com/kpn-dsh/dsh-api) file of the `dsh_api`
repository for an explanation on how to publish the `dsh_api` crate.

## Publish `dsh`

First make sure that the `dsh_api` dependency is pointing to the proper version at `crates.io`.
Check the dependency in `Cargo.toml`:

```toml
# When publishing, alway use the crates.io dependency.
dsh_api = { path = "../dsh-api/dsh-api", version = "0.10.0", features = ["generic"] }
# While developing use the relative path dependency to dsh_api and enable the manage feature.
#dsh_api = { path = "../dsh-api/dsh-api", features = ["generic", "manage"] }
```

Now from the root directory of the `dsh` project publish the crate using:

```shell
> cargo publish --all-features --dry-run
> ...
> cargo publish --all-features
```

Once the tool is published to `crates.io`, the next step is to create a binary release.

[Release &#x2192;](release.md)
