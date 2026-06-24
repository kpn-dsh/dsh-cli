# Release to `GitHub`

[&#x2190; Publish](publish.md)

Once the `dsh` tool is published to `crates.io` it can easily be installed by typing:

```shell
> cargo install dsh
```

But this requires that you have a Rust toolchain installed. For people who are not familiar
with Rust or do not have the toolchain available, it is convenient to have a binary release
available as well.

This page describes the steps required to release the `dsh` tool to `GitHub` releases.
In this explanation it is assumed that your local copy is in sync with the version on `GitHub`
and that it is tagged with the version number (currently `0.10.0`) of the release, without
any prefixes. This tag is required when creating the `GitHub` release.

## Build release binaries

The `dsh` tool needs to be build separately for each required feature set and for each
supported platform. The build steps are described assuming a macOS platform. For Linux or
Windows the steps are similar, but the details may vary.

It is assumed that a recent Rust toolchain is installed.

### macOS

Since we're building for the native platform (macOS, `aarch64-apple-darwin`), the build
commands are simple:

```shell
> cargo build --release
...
> mv target/release/dsh dsh-v0.10.0-aarch64-apple-darwin
> cargo build --release --all-features
...
> mv target/release/dsh dsh-manage-v0.10.0-aarch64-apple-darwin
```

This will result in two executables in your project directory, one with `robot`, `stream` and
`tenant` features enabled, and one without:

```shell
> ls -al
total 87072
...
-rwxr-xr-x   1 username  staff  23506384  6 mrt. 09:51 dsh-manage-v0.10.0-aarch64-apple-darwin
-rwxr-xr-x   1 username  staff  21444176  6 mrt. 09:51 dsh-v0.10.0-aarch64-apple-darwin
...
```

For the binaries to be able to run on other machines than the machine it was build on,
the binaries need to be codesigned and notarized. See
[Codesign and notarize for macOS](code-signing-macos.md) for the required steps.

### Linux

Since we're building for the native platform (macOS, `aarch64-apple-darwin`), the build
commands are simple:

```shell
> rustup target add x86_64-unknown-linux-gnu
```

```shell
> cargo build --target x86_64-unknown-linux-gnu --release
...
> mv target/release/dsh dsh-v0.10.0-x86_64-unknown-linux-gnu
> cargo build --target x86_64-unknown-linux-gnu --release --all-features
...
> mv target/release/dsh dsh-manage-v0.10.0-x86_64-unknown-linux-gnu
```

This will result in two executables in your project directory, one with `robot`, `stream` and
`tenant` features enabled, and one without:

```shell
> ls -al
total 87072
...
-rwxr-xr-x   1 username  staff  23506384  6 mrt. 09:51 dsh-manage-v0.10.0-x86_64-unknown-linux-gnu
-rwxr-xr-x   1 username  staff  21444176  6 mrt. 09:51 dsh-v0.10.0-x86_64-unknown-linux-gnu
...
```

### Windows

To be done...

## Create `GitHub` release

The section describes how to create a binary release at `GitHub` releases.

Prerequisites for creating a release are:

* All macOS binaries are available, properly named, codesigned and notarized:
    * `dsh-manage-v0.10.0-aarch64-apple-darwin`
    * `dsh-v0.10.0-aarch64-apple-darwin`
* If applicable, all Linux binaries are available and properly named.
* If applicable, all Windows binaries are available, properly named and codesigned.
* The release git branch is tagged with the release version number, without prefixes
  (`0.10.0`).

Open the `GitHub` releases page:
[https://github.com/kpn-dsh/dsh-cli/releases](https://github.com/kpn-dsh/dsh-cli/releases).

Click `Draft a new release`.

On the `New release` page:

* Select `0.10.0` from the `Select tag` dropdown menu.
* Enter `0.10.0` for the release title.
* Add the release notes. Use the release notes from the previous version as a starting point
  and keep the styling the same.
* Drag all binaries to the page, or click the button and select the binaries from the
  filesystem.
* Select `Set as a pre-release` and/or `Set as the latest release` as desired.
* Click `Publish release` to publish or `Save draft` to save your work.

[README &#x2192;](../README.md)
