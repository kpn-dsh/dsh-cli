# `JSON` containing Kafka client properties

We will create a `JSON` file containing some generic Kafka client properties that can be used to
connect to the Kafka proxy, using the following command:

```shell
> dsh proxy code my-proxy json
generating json example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created json configuration file 'my-proxy-configuration.json'
json code for bundle 'my-proxy' generated
```

As is shown in the output, the properties are written to a `JSON` file named
`my-proxy-configuration.json`. Click below to see an example of the generated `JSON` code.

<details>
<summary><code>my-proxy-configuration.json</code></summary>

```json
{
  "client-id": "my-tenant",
  "group-id": "my-tenant_my-proxy_1",
  "bootstrap-servers": [
    "my-proxy-0.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-1.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-2.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091"
  ],
  "bundle-directory": "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/my-proxy",
  "ca-file": "ca.pem",
  "client-certificate-file": "client.pem",
  "client-key-file": "client.key"
}
```

</details>

[&#x2190; Kafka proxy](../proxy.md)
