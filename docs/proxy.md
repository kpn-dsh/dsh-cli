# Proxy

[&#x2190; Settings and targets](settings-targets.md)

This page describes how the `dsh` tool can be used to connect to a Kafka topic on the DSH platform
via a Kafka proxy. Kafka proxies are a powerful feature of the DSH platform, but they are hard
to set up properly. Using the `dsh` tool this becomes must easier.

Detailed information about the Kafka proxy can be found at the DSH documentation pages:
[https://docs.kpn-dsh.com](https://docs.kpn-dsh.com/platform-services/kafka-proxy).

Steps:

* Create a proxy certificate bundle
* Deploy the proxy
* Generate a code example

Creating proxy bundles, deploying proxies and generating and running code examples all require
the platform and tenant to be specified. For the examples below we will use `np-aws-lz-dsh` for
the platform and `my-tenant` for the tenant. In order to make the example commands easier to read
throughout the examples it is assumed that the platform and tenant are specified via environment
variables:

```shell
dcli> export DSH_CLI_PLATFORM=np-aws-lz-dsh
dcli> export DSH_CLI_TENANT=my-tenant
```

## Create a proxy certificate bundle

The first step is to create a so-called proxy certificate bundle, which contains all the settings
and the certificates and public/private key pairs. In a real situation the certificates
should be signed as an Organization Validated (OV) certificate or an Extended Validation (EV)
certificate, but the current version of the `dsh` tool does not support this yet.

For the example we will use a self-signed ca certificate and the following settings:

* proxy name: `my-proxy`
* ACL groups: not enabled
* certificate authority common name: `username`
* schema store: not enabled
* vhost zone: `private`

You can provide these parameters as command line arguments, but in the example we will not use
this and let the tool prompt us for them. Since we will use all default values, you can just
press the enter-key after each prompt.

```shell
> dsh proxy create my-proxy
create proxy certificates bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
enable acl groups? [y/N]
certificate authority common name [username]:
enable schema store? [y/N]
vhost zone [PRIVATE/public]:
┌────────────────┬─────────────────────────┐
│ bundle         │ my-proxy                │
├────────────────┼─────────────────────────┤
│ platform       │ np-aws-lz-dsh           │
│ tenant         │ my-tenant               │
│ proxy name     │ my-proxy                │
│ group id       │ my-tenant_my-proxy_1 │
│ ca common name │ username                │
│ schema store   │ disabled                │
│ vhost zone     │ private                 │
│ records        │ 10                      │
└────────────────┴─────────────────────────┘
...
proxy certificates bundle 'my-proxy' stored in directory '/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy'
```

After a few seconds an overview of the created configuration, certificates and keys will be
listed and the bundle will be ready. You can list the available proxy certificate bundles via the
`proxy list --bundle` command:

```shell
> dsh proxy list --bundle
list all local proxy certificate bundles for 'np-aws-lz-dsh@my-tenant'
┌──────────┬────────────────┬──────────────┬────────────┬─────────┬────────────────┬───────────────────────────────────────────────────────────────────────────┐
│ bundle   │ ca common name │ schema store │ vhost zone │ records │ acl group name │ directory                                                                 │
├──────────┼────────────────┼──────────────┼────────────┼─────────┼────────────────┼───────────────────────────────────────────────────────────────────────────┤
│ my-proxy │ username       │ disabled     │ private    │ 10      │                │ /Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy │
└──────────┴────────────────┴──────────────┴────────────┴─────────┴────────────────┴───────────────────────────────────────────────────────────────────────────┘
```

You can also list the files in the directory where the bundle is stored:

```shell
> ls -l /Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy
total 56
-rw-------  1 username  staff   180 28 mei  17:07 bundle.toml
-rw-------  1 username  staff  3272 28 mei  17:07 ca.key
-rw-------  1 username  staff  2130 28 mei  17:07 ca.pem
-rw-------  1 username  staff  3272 28 mei  17:07 client.key
-rw-------  1 username  staff  1984 28 mei  17:07 client.pem
-rw-------  1 username  staff  3272 28 mei  17:07 server.key
-rw-------  1 username  staff  2862 28 mei  17:07 server.pem
```

You should not touch or alter these files yourself.

## Deploy the proxy

So far we only created the proxy certificate bundle, which is stored in a directory on our
local computer. In this step we will be deploying the proxy to the DSH platform using the
following settings:

* number of cpus: `0.1`
* number of instances: `1`
* memory: `256`

Again these are the default values but since this is an example we will provide them as command
line arguments instead of letting the tool prompt us for the values:

```shell
> dsh proxy deploy my-proxy --cpus 0.1 --instances 1 --mem 256
[overview of the proxy certificate bundle]
deploy proxy 'my-proxy'? [y/N]
server certificate secret 'my-proxy-server-certificate' created
private key secret 'my-proxy-private-key' created
ca certificate secret 'my-proxy-ca-certificate' created
certificate 'my-proxy-certificate' created
proxy 'my-proxy' deployed
```

The tool will list all proxy certificate bundle settings again and will ask for confirmation.
After typing `y` the proxy will be deployed and the resources that are transferred to the DSH
platform will be listed. You can check whether the proxy is deployed by using the `proxy list`
command (or its shortcut `proxys`). Note that it might take around 10 seconds before everything
is deployed.

```shell
> dsh proxys
list all proxies with parameters
┌──────────┬──────────────────────┬──────┬────────┬─────────┬──────────────┬────────────┐
│ proxy id │ certificate          │ cpus │ memory │ zone    │ schema store │ acl groups │
├──────────┼──────────────────────┼──────┼────────┼─────────┼──────────────┼────────────┤
│ my-proxy │ my-proxy-certificate │ 0.1  │    256 │ private │ disabled     │ disabled   │
└──────────┴──────────────────────┴──────┴────────┴─────────┴──────────────┴────────────┘
```

If you are curious, you can also check the installed certificate and secrets:

```shell
> dsh certificate show my-proxy-certificate
> dsh secret show my-proxy-ca-certificate
> dsh secret show my-proxy-private-key
> dsh secret show my-proxy-server-certificate
```

## Generate a code example

Now that we have a running proxy, we want to use it to connect to a Kafka cluster on the DSH
platform. The easiest way to do this is to let the `dsh` tool create a code example and run it.
At this time code examples can be generated for the `Python` and `Rust` programming languages.
The next version of the tool will support more languages (`Go`, `Java`, `Javascript/Typescript`
and `Scala` are on the roadmap).

For each supported programming language there are (will be) three different examples generated:

<dl>
  <dt><code>consumer</code></dt>
  <dd>Will create a client that connects to the Kafka cluster as a consumer, subscribes
  to a topic and prints the keys of the records that it receives from the topic.</dd>
  <dt><code>list-topic</code></dt>
  <dd>Will will create a client that connects to the Kafka cluster as an admin or consumer and 
  lists all topics that the client can read from.</dd>
  <dt><code>producer</code></dt>
  <dd>Will create a client that connects to the Kafka cluster as a producer and sends a
  record to a topic each second, with the timestamp as record key.</dd>
</dl>

At this time, only a limited set of code examples are supported. Click on the link to see the
explanation.

* [Python code example](code-examples/python.md)
* [Rust code example](code-examples/rust.md)

## Proxy with ACL groups

Although proxy with ACL groups enabled are supported by the `dsh` tool, the documentation is not
yet complete. In short there are two extra steps.

### Enable ACL groups

To enable ACL groups, you have to answer `y` when asked for this in the first step described above.
Then you will be prompted for the ACL group name:

```shell
> dsh proxy create my-proxy
create proxy certificates bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
enable acl groups? [y/N]y
acl group name: my-aclgroup
...
```

We entered `my-aclgroup` as the ACL group name. Deploying the proxy and generating example
code are the same as for the case without ACL groups, but before you can run the code examples you
first have to create and configurae the ACL group.

### Create and configure ACL group

[Platforms specification &#x2192;](platforms-specification.md)
