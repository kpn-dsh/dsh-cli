# Certificate signing

[&#x2190; README](../README.md)

DSH supports Public Key Infrastructure (PKI) certificates for `vhosts` and `proxys`. To guarantee
trust and security it is required to use certificates with all services and proxies
running on the DSH. This is also a requirement in the KSP.

### Self-signed certificates

Self-signed certificates do not provide trust, but they do provide security (TLS and mTLS).
For some `proxy` use cases it might be sufficient to use self-signed certificates.
Self-signed certificates are not very useful for `vhosts`.

### Internal KPN certificates (_RoCK_)

Internal KPN domains can be signed by _RoCK_. The `dsh` tool can create certificate signing
requests and have the _RoCK API_ sign them. The ca chain used by _RoCK_ consists of the following
certificates:

* `KPN N.V. Private Root CA G3` - Root certificate authority
  ([link](https://artifacts.kpn.org/kpn-pki/kpn.private.ca.g3.crt)).
* `KPN TB Private CA G1` - Intermediate certificate authority
  ([link](https://artifacts.kpn.org/kpn-pki/intermediate.kpn.private.nv.tb-g1.crt)).

Laptops or MacBooks managed by KPN come with the root certificate installed, and some will also
have the intermediate certificate available. If you want to ensure that all browsers work properly
you can add the certificate chain to the installed certificates.

In test and development cases a `dsh proxy` can be used with a self-signed certificate. Although
this is secure, it does not provide any trust. By adding a signed certificate to a `proxy` the
client can be certain that he is connecting to a trustworthy endpoint.

### Onboarding with _RoCK API_

In order to be able to let _RoCK_ sign certificates for your tenant (`my-tenant`), you have to take
a few steps.

First you as a user need to be a member with `IAAS Administrator` privileges in the LDAP group you
will be using (`my_ldap_group`). See [MyTechnium](https://my.technium.kpn.org) for this.

Second, you need to onboard your tenant's domain with _RoCK_, by sending an e-mail to the TechBase
CTS Team at e-mail address <a href="mailto:infraplatform@kpn.com">infraplatform@kpn.com</a>. For
details on the onboarding process, see the
[_RoCK_ Confluence pages](https://kpn.atlassian.net/wiki/spaces/EX/pages/140518941/RoCK+Onboarding).
For the DSH platform, tenant name `my-tenant` and LDAP group `my_ldap_group`, you need to include
the following data in your e-mail:

<table>
  <tr valign="top">
    <th align="left">certificate type</th>
    <td><code>subdomain certificate</code></td>
  </tr>
  <tr valign="top">
    <th align="left">requested domain(s)</th>
    <td>
      <code>*.my-tenant.dsh-dev.dsh.np.aws.kpn.org</code><br/>
      <code>*.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org</code><br/>
      <code>*.my-tenant.dsh-prod.dsh.prod.aws.kpn.org</code><br/>
      <code>*.kafka.my-tenant.dsh-prod.dsh.prod.aws.kpn.org</code>
    </td>
  </tr>
  <tr valign="top">
    <th align="left">ldap group</th>
    <td><code>my_ldap_group</code></td>
  </tr>
  <tr valign="top">
    <th align="left">service now group</th>
    <td><code>KPN-DATA-Data Services Hub / Klarrio / id: GROUP200334</code></td>
  </tr>
</table>

### Authenticating with _RoCK API_

Once you and your tenant's domain are onboarded with _RoCK_, you need to authenticate yourself
before you will be able to use the `dsh` tool to access the _RoCK API_. The easiest method is to
install the `rock-client`on your machine
([link](https://kpn.atlassian.net/wiki/spaces/EX/pages/140516946/RoCK+Rock+Client))
and then enter:

```shell
> rock-client get_auth_token
```

This will open a web page where you can authenticate yourself, after which a token will be
installed on your machine. The `dsh` tool can use this token to access the _RoCK API_.

You can check whether you are properly authenticated and authorized by listing the _RoCK_
certificates and domains that you are entitled to:

```shell
> dsh certificate list --rock
> dsh certificate list --rock-domains
````

## Adding a `vhost` certificate

Although it is possible to access a `vhost` without a properly signed certificate, the user's
browser will warn against the missing certificates, which is not very convenient. On some
platforms or browsers you can ignore these warnings, but others will not let you proceed. All this
can be solved by installing a properly signed certificate with the `vhost`.

To add a signed certificate to an already existing `vhost`, use the command:

```shell
> dsh vhost add eavesdropper
````

You will be asked whether your domain is private or public, and which Certificate Authority you
want to use. For both questions use the default answer (just hit `enter`). If there already exists
a certificate for your domain in the _RoCK_ database you will be asked to confirm that _RoCK_
can create a new certificate for the domain, overwriting the old certificate. Re-using an existing
certificate is not yet supported so if you do not confirm, the operation will be canceled.

## Adding a signed Kafka `proxy` certificate

Adding certificates is a mandatory step for creating and deploying a `proxy`. The page about
[Kafka proxy](proxy.md) describes the process for self-signed certificates.

The process for creating signed certificates is very similar to the self-signed case. You just have
to omit the `--self-signed` option. You will be asked similar questions as with the `vhost` and the
self-signed case:

```shell
> dsh proxy create my-proxy
create proxy certificates bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
vhost zone [PRIVATE/public]:
certificate authority [KPN-CA/kpn-digic-rsdv]:
attach ca chain to certificates? [y/N]enable acl groups? [y/N]
enable schema store? [y/N]
vhost zone [PRIVATE/public]:
...
```

The deployment step, generating code examples and using `acl-groups` is exactly the same as for
the self-signed case.

[README &#x2192;](../README.md)
