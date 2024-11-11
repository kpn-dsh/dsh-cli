# DSH Resource Management Command Line Tool

> **NOTE**  
> This tool is still under development and will most likely contain many bugs.
> If you encounter any of these bugs (and you will), you can report them to `unibox@kpn.com`. 
> Please include the exact command, the erroneous output and an explanation of the expected output.
> 
> You can also send requests for new features to this e-mail address.

This project provides a tool to call functions on the DSH resource management API from the 
command line of your workstation. The following DSH resources can be 
listed, queried, searched, created and deleted.

<table>
    <tr align="top">
        <th align="left">resource</th>
        <th>create</th>
        <th>delete</th>
        <th>diff</th>
        <th>find/usage</th>
        <th>list</th>
        <th>show</th>
        <th>update</th>
    </tr>
    <tr align="top">
        <td align="left">app from the app catalog</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">application / service</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">bucket</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">certificate</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">environment variable</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">image</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">manifest</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">metric</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">proxy</td>
        <td align="center"></td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25CE;</td>
    </tr>
    <tr align="top">
        <td align="left">secret</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">topic</td>
        <td align="center"></td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">vhost</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr align="top">
        <td align="left">volume</td>
        <td align="center">&#x25CE;</td>
        <td align="center">&#x25CE;</td>
        <td align="center"></td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center">&#x25C9;</td>
        <td align="center"></td>
    </tr>
</table>

Meaning of the dots:

<table>
    <tr>
        <td>&#x25C9;</td>
        <td>Capability is supported for this resource.</td>
    </tr>
    <tr>
        <td>&#x25CE;</td>
        <td>Capability for this resource is still experimental, incomplete or untested. 
            Use at your own risk.</td>
    </tr>
</table>

## Local installation and run

The DSH Api Command Line Tool (`dcli`) can be installed on your local machine
(assuming you have `git`, `rust` and `cargo` installed),
by executing the following command in a suitable directory.

```bash
> git clone git@github.com:kpn-dsh/dcli.git
...
> cd dcli
> cargo install --path dcli
```

### Environment variables

In order to run `dcli` make sure that the environment variables described below
are properly set.
Since the command line tool is based on the [DSH API Client](../dsh_api/README.md),
these environment variables are the same as for the client.

<table>
    <tr valign="top">
        <th align="left">variable</th>
        <th align="left">description</th>
    </tr>
    <tr valign="top">
        <td><code>DSH_API_PLATFORM</code></td>
        <td>
            Target platform on which the tenant's environment lives.
            <ul>
                <li><code>nplz</code> - Non production landing zone</li>
                <li><code>poc</code> - Proof of concept platform</li>
                <li><code>prod</code> - Production landing zone</li>
                <li><code>prodaz</code></li>
                <li><code>prodlz</code></li>
            </ul>
        </td>
    </tr>
    <tr valign="top">
        <td><code>DSH_API_TENANT</code></td>
        <td>Tenant id for the target tenant. The target tenant is the tenant whose resources 
            will be managed via the api.</td>
    </tr>
    <tr valign="top">
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
    <tr valign="top">
        <td><code>DSH_API_GUID_[tenant]</code></td>
        <td>
            Group id and user id for the target tenant.
            The placeholder <code>[tenant]</code> needs to be substituted 
            with the tenant name in all capitals, with hyphens (<code>-</code>) 
            replaced by underscores (<code>_</code>).
            E.g. if the tenant name is <code>greenbox-dev</code>, the environment variable must be
            <code>DSH_API_GUID_GREENBOX_DEV = "1903:1903"</code>.
        </td>
    </tr>
</table>

### Run

When installation completed without any errors and the environment variables are set, 
you should be able to start the tool from the command line.

```bash
> dcli
DSH api command line interface

Usage: dcli [OPTIONS] [COMMAND]

Commands:
  app          Show, manage and list apps deployed from the DSH app catalog.
  application  Show, manage and list applications deployed on the DSH.
  bucket       Show, manage and list DSH buckets.
  certificate  Show, manage and list DSH certificates.
  env          Find values used in configurations.
  image        Show image usage.
  manifest     Show App Catalog manifests.
  metric       Show metric exports.
  proxy        Show, manage and list DSH Kafka proxies.
  secret       Show, manage and list DSH secrets.
  topic        Show, manage and list DSH topics.
  vhost        Show vhost usage.
  volume       Show, manage and list DSH volumes.

Options:
      --no-border              Omit output border
  -p, --platform <PLATFORM>    Target platform
      --verbosity <VERBOSITY>  Verbosity level
  -t, --tenant <TENANT>        Target tenant
  -v...                        Verbosity level
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version

For most commands adding an 's' as a postfix will yield the same result as using
the 'list' subcommand, e.g. using 'dcli apps' will be the same as using 'dcli
app list'.
```
You can have a more comprehensive explanation by adding the `--help` command line option, 
and all available commands also have their own help text.

```bash
> dcli --help
> dcli secret --help
> dcli secret list --help
```

## DSH Api Client

The command line tool is based on [DSH Api Client](dsh_api).

## Coding guidelines

Before pushing code to github, make sure that you adhere to the code formatting defined in 
`rustfmt.toml`. The following command shoud return without any remarks:

```bash
> cargo +nightly fmt --check
```

Consider configuring your IDE to automatically apply the formatting rules when saving a file. 
