# `JavaScript` code example

It is assumed that you have a recent version of `scala3`, its `sbt` package manager and a `jvm`
installed. The generated code has a dependency on the Apache Kafka client jar.

We will create the example code for the `scala3` programming language with the following
command:

```shell
> dsh proxy code my-proxy scala3
generating scala3 example for bundle 'my-proxy' for 'np-aws-lz-dsh@my-tenant'
created directory 'my-proxy-example-scala3'
created directory 'my-proxy-example-scala3/src/main/scala/org/kpn/dsh/proxy/my_proxy'
created file 'my-proxy-example-scala3/README.md'
created file 'my-proxy-example-scala3/build.sbt'
created file 'my-proxy-example-scala3/src/main/scala/org/kpn/dsh/proxy/my_proxy/package.scala'
created file 'my-proxy-example-scala3/src/main/scala/org/kpn/dsh/proxy/my_proxy/Consumer.scala'
created file 'my-proxy-example-scala3/src/main/scala/org/kpn/dsh/proxy/my_proxy/Producer.scala'
created file 'my-proxy-example-scala3/src/main/scala/org/kpn/dsh/proxy/my_proxy/ListTopics.scala'
created file 'my-proxy-example-scala3/src/main/scala/org/kpn/dsh/proxy/my_proxy/DshProxyProperties.scala'
scala3 code for bundle 'my-proxy' generated in directory 'my-proxy-example-scala3'
```

As is shown in the output, the example is generated in a newly created directory which contains
a `README.md`, a `build.sbt` manifest and a `src` directory. The `src` directory contains 5
`scala`. Click below to see the generated code.

<details>
<summary><code>README.md</code></summary>

```markdown
# `Scala3` code example

It is assumed that you have a recent version of `scala3`, its `sbt` package manager and a `jvm`
installed. The generated code has a dependency on the Apache Kafka client jar.

We run the examples directly via `sbt`. First we will list all Kafka topics that we have access to.

> cd rguc-proxy-example-scala3
> sbt "runMain org.kpn.dsh.proxy.rguc_proxy.ListTopics"
...
scratch.my-topic.rguc
...

Then we will send messages to a Kafka topic. In the example below we use topic
`scratch.my-topic.rguc`. Use `ctrl-c` to stop the program.

> sbt "runMain org.kpn.dsh.proxy.rguc_proxy.Producer scratch.my-topic.rguc"
rguc-proxy-producer-scala3: 1780047895
rguc-proxy-producer-scala3: 1780047896
rguc-proxy-producer-scala3: 1780047897
rguc-proxy-producer-scala3: 1780047898
^C

Next we will consume records from the same topic. Again, use `ctrl-c` to stop the program.

```shell
> sbt "runMain org.kpn.dsh.proxy.rguc_proxy.Consumer scratch.my-topic.rguc"
0:2808 rguc-proxy-producer-scala3: 1780047895
0:2809 rguc-proxy-producer-scala3: 1780047896
0:2810 rguc-proxy-producer-scala3: 1780047897
0:2811 rguc-proxy-producer-scala3: 1780047898
^C
```

</details>

<details>
<summary><code>build.sbt</code></summary>

```scala
ThisBuild / scalaVersion := "3.3.6"

ThisBuild / scalacOptions ++= Seq("-release", "21")

lazy val root = (project in file(".")).settings(name := "rguc-proxy")

libraryDependencies ++= Seq(
  "org.apache.kafka" % "kafka-clients" % "4.4.0",
  "org.slf4j" % "slf4j-nop" % "2.0.20"
)
```

</details>

<details>
<summary><code>src/main/scala/org/kpn/dsh/proxy/rguc_proxy/package.scala</code></summary>

[//]: # (@formatter:off)
```scala
package org.kpn.dsh.proxy

import org.apache.kafka.clients.CommonClientConfigs

import scala.collection.mutable

package object my_proxy {
  val BundleDir: String = "/Users/username/.dsh_cli/targets/np-aws-lz-dsh/my-tenant/bundles/proxy/my-proxy"
  val ClientId: String = "my-tenant"
  val GroupId: String = "my-tenant_my-proxy_1"
  val Brokers: List[String] = List(
    "my-proxy-0.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-1.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091",
    "my-proxy-2.kafka.my-tenant.dsh-dev.dsh.np.aws.kpn.org:9091"
  )

  def commonClientProperties: mutable.Map[String, Any] =
    val properties = DshProxyProperties.fromBundleDir(BundleDir).get
    properties.put(CommonClientConfigs.CLIENT_ID_CONFIG, ClientId)
    properties.put(CommonClientConfigs.BOOTSTRAP_SERVERS_CONFIG, Brokers.mkString(","))
    properties
}
```

</details>

<details>
<summary><code>src/main/scala/org/kpn/dsh/proxy/rguc_proxy/Consumer.scala</code></summary>

[//]: # (@formatter:off)
```scala
package org.kpn.dsh.proxy.my_proxy

import org.apache.kafka.clients.consumer.{ConsumerConfig, KafkaConsumer}
import org.kpn.dsh.proxy.my_proxy

import java.time.Duration
import scala.jdk.CollectionConverters.*

object Consumer:

  def main(args: Array[String]): Unit =
    if (args.length < 1)
      println("missing topic argument")
      System.exit(1)
    val topic = args(0)

    val properties = commonClientProperties
    properties.put(ConsumerConfig.GROUP_ID_CONFIG, GroupId)
    properties.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.StringDeserializer")
    properties.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.StringDeserializer")

    val consumer = new KafkaConsumer[String, String](properties.asJava)

    consumer.subscribe(List(topic).asJava)

    while true
    do
      consumer
        .poll(Duration.ofSeconds(5))
        .records(topic)
        .asScala
        .foreach(record => println(record.key.trim))
```

</details>

<details>
<summary><code>src/main/scala/org/kpn/dsh/proxy/rguc_proxy/DshProxyProperties.scala</code></summary>

[//]: # (@formatter:off)
```scala
package org.kpn.dsh.proxy.my_proxy

import org.apache.kafka.clients.CommonClientConfigs
import org.apache.kafka.common.config.SslConfigs
import org.kpn.dsh.proxy.my_proxy

import java.io.{ByteArrayInputStream, File, FileOutputStream}
import java.security.cert.{Certificate, CertificateFactory}
import java.security.spec.PKCS8EncodedKeySpec
import java.security.{KeyFactory, KeyStore, PrivateKey, SecureRandom}
import java.util
import java.util.stream.Collectors
import java.util.{Base64, Random}
import scala.collection.mutable
import scala.util.{Failure, Success, Try}

object DshProxyProperties:
  private val CaChainFilename = "ca.pem"
  private val ClientCertFilename = "client.pem"
  private val ClientKeyFilename = "client.key"

  /** Generate authentication properties for DSH Kafka proxy from proxy bundle directory
   *
   * Loads the ca chain certificate, the mTLS client certificate and the client key from the proxy bundle directory.
   * The expected files names are `ca.pem`, `client.pem` and `client.key` respectively. Then these values are stored
   * in newly created trust and key stores, which are written to the file system in temporary files. The locations
   * and passwords of these trust and key stores are returned as Kafka properties in a mutable scala`Map`.
   * The following properties are set:
   *
   * - `security.protocol` - Protocol used to communicate with brokers, fixed at `ssl`.
   * - `ssl.truststore.location` - Location of the trust store file.
   * - `ssl.truststore.password` - Password for the trust store file.
   * - `ssl.keystore.location` - Location of the key store file.
   * - `ssl.keystore.password` - Password for the key store file.
   * - `ssl.key.password` - Password of the private key in the key store file or the PEM key specified in
   *   'ssl.keystore.key'.
   *
   * @param bundleDir Directory that contains the proxy bundle, as generated by the DSH cli tool.
   * @return Mutable `Map` containing the authentication properties for DSH Kafka proxy.
   */
  def fromBundleDir(bundleDir: String): Try[mutable.Map[String, Any]] =
    try
      val caChainSource = scala.io.Source.fromFile(bundleDir + "/" + CaChainFilename)
      val caChain = try caChainSource.mkString finally caChainSource.close()
      val clientCertSource = scala.io.Source.fromFile(bundleDir + "/" + ClientCertFilename)
      val clientCert = try clientCertSource.mkString finally clientCertSource.close()
      val clientKeySource = scala.io.Source.fromFile(bundleDir + "/" + ClientKeyFilename)
      val clientKey = try clientKeySource.mkString finally clientKeySource.close()
      create(caChain, clientCert, clientKey)
    catch case e: Exception => Failure(e)

  /** Create authentication properties for DSH Kafka proxy
   *
   * Stores the ca chain certificate, the mTLS client certificate and the client key in newly created trust and key
   * stores, which are written to the file system in temporary files. The locations and passwords of these trust and
   * key stores are returned as Kafka properties in a mutable scala `Map`. The following properties are set:
   *
   * - `security.protocol` - Protocol used to communicate with brokers, fixed at `ssl`.
   * - `ssl.truststore.location` - Location of the trust store file.
   * - `ssl.truststore.password` - Password for the trust store file.
   * - `ssl.keystore.location` - Location of the key store file.
   * - `ssl.keystore.password` - Password for the key store file.
   * - `ssl.key.password` - Password of the private key in the key store file or the PEM key specified in
   *   'ssl.keystore.key'.
   *
   * @param caChain Ca chain certificate.
   * @param clientCert Client certificate (mTLS).
   * @param clientKey Client key.
   * @return Mutable `Map` containing the authentication properties for DSH Kafka proxy.
   */
  def create(caChain: String, clientCert: String, clientKey: String): Try[mutable.Map[String, Any]] =
    try
      val password = generatePassword
      val truststorePath = saveToFile(createTruststore(caChain, password), password)
      val clientCertificate = certificateFromPEM(clientCert)
      val keystorePath = saveToFile(createKeystore(clientKey, clientCertificate, password), password)

      val properties = mutable.Map.empty[String, Any]
      properties.put(CommonClientConfigs.SECURITY_PROTOCOL_CONFIG, "ssl")
      properties.put(SslConfigs.SSL_TRUSTSTORE_LOCATION_CONFIG, truststorePath.toString)
      properties.put(SslConfigs.SSL_TRUSTSTORE_PASSWORD_CONFIG, password)
      properties.put(SslConfigs.SSL_KEYSTORE_LOCATION_CONFIG, keystorePath.toString)
      properties.put(SslConfigs.SSL_KEYSTORE_PASSWORD_CONFIG, password)
      properties.put(SslConfigs.SSL_KEY_PASSWORD_CONFIG, password)
      Success(properties)
    catch case e: Exception => Failure(e)

  private def createKeystore(clientKey: String, clientCert: Certificate, password: String): KeyStore =
    val jksStore = KeyStore.getInstance("JKS")
    jksStore.load(null, password.toCharArray)
    jksStore.setCertificateEntry("cert", clientCert)
    val key = convertPemToPrivateKey(clientKey)
    jksStore.setKeyEntry("1", key, password.toCharArray, Array(clientCert))
    jksStore

  private def convertPemToPrivateKey(key: String): PrivateKey =
    val clientKeyDer = key
      .replace("-----BEGIN PRIVATE KEY-----", "")
      .replace("-----END PRIVATE KEY-----", "")
      .replace("\n", "")
    val keyBytes = Base64.getMimeDecoder.decode(clientKeyDer)
    val encodedPKCS8Key = new PKCS8EncodedKeySpec(keyBytes)
    val keyFactory = KeyFactory.getInstance("RSA")
    keyFactory.generatePrivate(encodedPKCS8Key)

  private def createTruststore(caChain: String, password: String): KeyStore =
    val trustStore = KeyStore.getInstance("jks")
    trustStore.load(null, password.toCharArray)
    val caChainArray = convertPemCaChainToCaChainArray(caChain)
    for ((caCert, index) <- caChainArray.zipWithIndex)
      trustStore.setCertificateEntry(s"ca${index}", caCert)
    trustStore

  private def convertPemCaChainToCaChainArray(caChain: String): Array[Certificate] =
    val splitCertificateOn = "-----END CERTIFICATE-----"
    caChain
      .strip
      .split(splitCertificateOn)
      .map(_ + splitCertificateOn)
      .map(certificateFromPEM)

  private val passwordCharacters: String = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"

  private def generatePassword: String = {
    val random: Random = new SecureRandom
    util.Arrays.stream(new Array[Character](32)).map((c: Character) => passwordCharacters.charAt(random.nextInt(passwordCharacters.length))).map(String.valueOf).collect(Collectors.joining)
  }

  private def certificateFromPEM(pem: String) = CertificateFactory.getInstance("X.509").generateCertificate(new ByteArrayInputStream(rawContentFromPEM(pem)))

  private def rawContentFromPEM(pem: String) = {
    val PEM_START = "-----BEGIN CERTIFICATE-----"
    val PEM_END = "-----END CERTIFICATE-----"
    try {
      var tokens = pem.split(PEM_START)
      tokens = tokens(1).split(PEM_END)
      val buf = Base64.getMimeDecoder.decode(tokens(0))
      buf
    } catch {
      case e: Exception =>
        throw new IllegalArgumentException("Invalid PEM file content", e)
    }
  }

  private def saveToFile(store: KeyStore, password: String): File = {
    val file = File.createTempFile("__secret_store__", "." + store.getType.toLowerCase)
    val fos = new FileOutputStream(file)
    try store.store(fos, password.toCharArray)
    finally if (fos != null) fos.close()
    file
  }
```

</details>

<details>
<summary><code>src/main/scala/org/kpn/dsh/proxy/rguc_proxy/ListTopics.scala</code></summary>

[//]: # (@formatter:off)
```scala
package org.kpn.dsh.proxy.my_proxy

import org.apache.kafka.clients.admin.AdminClient
import org.kpn.dsh.proxy.my_proxy

import scala.jdk.CollectionConverters.*

object ListTopics:

  def main(args: Array[String]): Unit =
    val properties = commonClientProperties

    val admin_client = AdminClient.create(properties.asJava)

    admin_client
      .listTopics
      .names
      .get
      .asScala
      .toList
      .sorted
      .foreach(println)
```

</details>

<details>
<summary><code>src/main/scala/org/kpn/dsh/proxy/rguc_proxy/Producer.scala</code></summary>

[//]: # (@formatter:off)
```scala
package org.kpn.dsh.proxy.my_proxy

import org.apache.kafka.clients.producer.{KafkaProducer, ProducerConfig, ProducerRecord}
import org.kpn.dsh.proxy.my_proxy

import scala.jdk.CollectionConverters.*

object Producer:

  def main(args: Array[String]): Unit =
    if (args.length < 1)
      println("missing topic argument")
      System.exit(1)
    val topic = args(0)

    val properties = commonClientProperties
    properties.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.StringSerializer")
    properties.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.StringSerializer")

    val producer = new KafkaProducer[String, String](properties.asJava)

    while true
    do
      val now = System.currentTimeMillis / 1000
      val key = s"rguc-proxy-producer-scala3: {$now}"
      val record = new ProducerRecord[String, String](topic, key, "payload")
      println(key)
      producer.send(record)
      Thread.sleep(1000)
```

</details>

The `scala` file `DshProxyProperties.scala` contains the code to generate the properties
needed to connect to the Kafka proxy.

We run the examples directly via `sbt`. First we will list all Kafka topics that we have access to.

```shell
> cd my-proxy-example-scala3
> sbt "runMain org.kpn.dsh.proxy.my_proxy.ListTopics"
...
scratch.my-topic.my-tenant
...
```

Then we will send messages to a Kafka topic. In the example below we use topic
`scratch.my-topic.my-tenant`. Use `ctrl-c` to stop the program.

```shell
> sbt "runMain org.kpn.dsh.proxy.my_proxy.Producer scratch.my-topic.my-tenant"
my-proxy-producer-scala3: 1780047895
my-proxy-producer-scala3: 1780047896
my-proxy-producer-scala3: 1780047897
my-proxy-producer-scala3: 1780047898
^C
```

Next we will consume records from the same topic. Again, use `ctrl-c` to stop the program.

```shell
> sbt "runMain org.kpn.dsh.proxy.my_proxy.Consumer scratch.my-topic.my-tenant"
0:2808 my-proxy-producer-scala3: 1780047895
0:2809 my-proxy-producer-scala3: 1780047896
0:2810 my-proxy-producer-scala3: 1780047897
0:2811 my-proxy-producer-scala3: 1780047898
^C
```

[&#x2190; Kafka proxy](../proxy.md)
