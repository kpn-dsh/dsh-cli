# Make

[&#x2190; Release](release.md)

The `Makefile` in the root of this project defines rules and targets to help with the necessary
steps for building, signing, publishing and releasing the `dsh` tool.

## Prerequisites

### Toolchain

<details>
<summary><b>Tool chain</b></summary>

* rustup target add x86_64-unknown-linux-musl
* brew install filosottile/musl-cross/musl-cross

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

</details>

<details>
<summary><b>KPN Developer Certificate and Team</b></summary>

If you want to codesign and notarize the `dsh` tool as a genuine KPN-application you need:

* the KPN Developer Certificate installed on your development machine and
* an Apple developer account that is a member of the KPN Apple Development Team that owns the
  certificate (team id `B86ZND72C8`).

The certificate as well as the development team are owned/maintained by the KPN App Center.
Ask them for the certificate and to invite you to become a member of their development team .
The KPN App Center can be contacted via e-mail at `appcenter@kpn.com`.

For the remainder of this explanation it is assumed that the developer account id is
`your.name@kpn.com`.

Once you received the certificate from the App Center, install it on the machine that you will use
to codesign the `dsh` tool. You can use the macOS `Keychain Access` tool to check whether the
certificate is properly installed. It should be visible on the `Certificates` tab as

`Developer ID Application: KPN B.V. (B86ZND72C8)`.

Since this certificate is issued by the Apple Developer ID Certification Authority,
you should also have their CA certificate installed (`Developer ID Certification Authority`) in
order for the KPN certificate to be trusted. If you don't have it already, you can download and
install it from [DeveloperIDG2CA](https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer).

You can check whether the KPN Developer Certificate is properly installed using the command:

```shell
> security find-identity -v -p codesigning
```

If everything is ok this command should return something like:

```shell
1) FA675DE5B91B034C50FF28367713EE347A5A5C0C "Developer ID Application: KPN B.V. (B86ZND72C8)"
   1 valid identities found
```

</details>

### Coding guidelines

Check Cargo.toml for proper dsh_api dependencies.

Before pushing code to `GitHub`, make sure that you adhere to the code formatting defined in
`rustfmt.toml`, that you have run the `clippy` linter and that you did the license check.
The following commands should return without any remarks:

```shell
> cargo +nightly fmt --check
> cargo clippy
> cargo deny check licenses
```

## Steps

Build the tool with the following command:

```shell
> make check
> make build
```

The build step will result in four binary files in the `releases` directory.

### Code signing

Next step is signing the binary with the KPN developers certificate.
Code signing is executed by the `codesign` command:

```shell
> make codesign
```

This command will most likely ask four your Keychain password. You can check the result by:

```shell
> make codesign-check
```

### Create app-specific password

The notarise step requires that you specify an app-specific password for the application.

* Log in to https://appleid.apple.com with your developer account.
* Select menu `Sign-In and Security`.
* Click `App-Specific Passwords`.
* Click `+` button.
* Enter `dsh` as the app-name.
* Click `Create`.
* When requested, enter your apple-id password.
* Copy the generated password (looks like `abcd-efgh-ijkl-mnop`).

For the rest of this explanation `abcd-efgh-ijkl-mnop` will be used for the password.

### Notarize

In order to notarize the `dsh` tool it must first be packed in a zip file:

```shell
> zip dsh.zip target/release/dsh
```

This creates the file `dsh.zip` containing only the `dsh` binary.
This zip file can then be submitted to be notarised by the following command:

```shell
> make notarize password=abcd-efgh-ijkl-mnop
Conducting pre-submission checks for releases/dsh-v0.10.0-mac-binaries.zip and initiating connection to the Apple notary service...
Submission ID received
  id: abcdef01-2345-6789-abcd-ef0123456789
Upload progress: 100,00% (5,39 MB of 5,39 MB)
Successfully uploaded file
  id: abcdef01-2345-6789-abcd-ef0123456789
  path: releases/dsh-v0.10.0-mac-binaries.zip
```

Be sure to save the provided id (in the example `abcdef01-2345-6789-abcd-ef0123456789`), since you
might need it to check the status of the process.

Notarization usually takes less than 5 minutes, but in some cases it can take quite a bit longer.
In order to poll the status of the process you can use the following command:

```shell
> make notary-poll password=abcd-efgh-ijkl-mnop session=abcdef01-2345-6789-abcd-ef0123456789
{
  "logFormatVersion": 1,
  "jobId": "abcdef01-2345-6789-abcd-ef0123456789",
  "status": "Accepted",
  "statusSummary": "Ready for distribution",
  "statusCode": 0,
  "archiveFilename": "dsh-v0.10.0-mac-binaries.zip",
  "uploadDate": "2025-07-15T11:02:34.611Z",
  "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
  "ticketContents": [
    {
      "path": "dsh-v0.10.0-mac-binaries.zip/dsh",
      "digestAlgorithm": "SHA-256",
      "cdhash": "abcdef0123456789abcdef0123456789abcdef01",
      "arch": "x86_64"
    }
  ],
  "issues": null
}
```

When the notarize process is finished, you can check whether it was successful using the
`codesign` command:

```shell
> make notarize-check
target/release/dsh: valid on disk
target/release/dsh: satisfies its Designated Requirement
target/release/dsh: explicit requirement satisfied
```

## Tool

A useful tool is [What' s Your Sign](https://objective-see.org/products/whatsyoursign.html).
It allows you to check the status by right-clicking the `dsh` binary in the Finder and select the
`Signing Info` context menu. When codesigning and notarising are both completed the tool will show
this as follows:

<img src="images/whats-your-sign.png" width="600" />

[Release &#x2192;](release.md)
