# Certificate signing

[&#x2190; Release](release.md)

DSH supports certificates for vhosts and proxys, according to the Public Key Infrastructure (PKI).
To guarantee trust and security it is required to use certificates with all services and proxies
running on the DSH. This is also a requirement in the KSP.

## Certificates

For the DSH there are two kinds of certificates

### KPN certificates (_RoCK_)

* `KPN N.V. Private Root CA G3` - Root certificate authority
  ([link](https://artifacts.kpn.org/kpn-pki/kpn.private.ca.g3.crt))

* `KPN TB Private CA G1` - Intermediate certificate authority
  ([link](https://artifacts.kpn.org/kpn-pki/intermediate.kpn.private.nv.tb-g1.crt))

Laptops or MacBooks managed by KPN come with the root certificate installed, and some will also
have the intermediate certificate available. If you want to ensure that all browsers work properly
you can add the certificate chain to the installed certificates.

### Self-signed certificates

Self-signed certificates do not provide trust, but they do provide security (TLS and mTLS).
For some `proxy` use cases it might be sufficient to use self-signed certificates.
Self-signed certificates are not relevant for vhosts.

## Vhost certificates

Although it is possible to access a vhost without a properly signed certificate, the user's
browser/client will warn about the missing certificates. In some browsers, you can ignore these
warnings, but this is not very convenient. This can be solved by installing a properly signed
certificate with the vhost.

## Kafka proxy certificates

### Onboarding with _RoCK API_

TechBase CTS Team (<a href="mailto:infraplatform@kpn.com">infraplatform@kpn.com</a>)

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

[Release &#x2192;](release.md)
